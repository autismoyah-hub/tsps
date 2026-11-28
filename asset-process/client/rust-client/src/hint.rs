//! Persistent hint-arrow state from kb.ao (packet 50).
use crate::{buffer::Reader, Error, Result};
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Hint {
    pub kind: u8,
    pub index: u16,
    pub tile: [u16; 2],
    pub offset: [i32; 2],
    pub height: u32,
    pub plane: i32,
}
impl Hint {
    /// Unknown wire kinds retain the previous kind and decode its payload, as in the client.
    /// The fixed-size network packet can contain padding after the consumed fields.
    pub fn apply(&mut self, raw: &[u8], plane: i32) -> Result<usize> {
        if raw.len() != 6 {
            return Err(Error::Invalid("hint packet length"));
        }
        let mut next = self.clone();
        next.plane = plane;
        let mut r = Reader::new(raw);
        match r.u8()? {
            1 => next.kind = 1,
            n @ 2..=6 => {
                next.kind = 2;
                next.offset = match n {
                    3 => [0, 64],
                    4 => [128, 64],
                    5 => [64, 0],
                    6 => [64, 128],
                    _ => [64, 64],
                };
            }
            10 => next.kind = 3,
            11 => next.kind = 4,
            _ => {}
        }
        match next.kind {
            0 => {}
            1 | 3 => next.index = r.u16()?,
            2 => {
                next.tile = [r.u16()?, r.u16()?];
                next.height = r.u8()? as u32;
            }
            4 => {
                next.index = r.u16()?;
                next.height = r.u24()?;
            }
            _ => return Err(Error::Invalid("hint kind")),
        }
        *self = next;
        Ok(raw.len() - r.remaining())
    }
}

impl Hint {
    pub fn flashes(&self, cycle: i32) -> bool {
        self.kind != 0 && cycle % 20 < 10
    }
    /// GameClient.ti/jl: both actor kinds search registry order, tile hints use
    /// containing child views, and entity hints use the root entity pose.
    pub fn minimap_point(
        &self,
        views: &[crate::projectile_world::View],
        entities: Option<&crate::world_entity::State>,
        mut actor: impl FnMut(u16, bool, u16) -> Option<[i32; 2]>,
    ) -> Option<[i32; 2]> {
        let root = views.iter().find(|v| v.id == 0)?;
        let (view, fine) = match self.kind {
            1 | 3 => views
                .iter()
                .find_map(|v| actor(v.id, self.kind == 1, self.index).map(|p| (v, p)))?,
            2 => {
                let view = views
                    .iter()
                    .find(|v| {
                        v.id != 0
                            && (0..2).all(|i| {
                                self.tile[i] as i32 >= v.base[i]
                                    && (self.tile[i] as i32)
                                        < v.base[i].wrapping_add(v.dimensions[i])
                            })
                    })
                    .unwrap_or(root);
                (
                    view,
                    std::array::from_fn(|i| {
                        (self.tile[i] as i32)
                            .wrapping_sub(view.base[i])
                            .wrapping_mul(128)
                            .wrapping_add(self.offset[i])
                    }),
                )
            }
            4 => {
                return entities?
                    .entities
                    .get(&self.index)
                    .map(|e| [e.pose.x, e.pose.z])
            }
            _ => return None,
        };
        Some(if view.id == 0 {
            fine
        } else {
            entities
                .and_then(|s| s.entities.get(&view.id))
                .map_or(fine, |e| {
                    let p = e.transform_horizontal(fine);
                    [p[0] as i32, p[2] as i32]
                })
        })
    }
    /// Convert actor copies' absolute cache coordinates into view-local input,
    /// then return the root-absolute coordinates used by the minimap listener.
    pub fn minimap_fine(&self, client: &crate::client_state::ClientState) -> Option<[i32; 2]> {
        let entity = |id| {
            client
                .world_entities
                .values()
                .find_map(|s| s.entities.get(&id))
        };
        let views: Vec<_> = client
            .view_registry
            .ordered()
            .into_iter()
            .filter_map(|id| {
                if id == 0 {
                    Some(crate::projectile_world::View {
                        id,
                        base: client.world_base,
                        dimensions: [104; 2],
                    })
                } else {
                    let e = entity(id)?;
                    Some(crate::projectile_world::View {
                        id,
                        base: e.rebuild.as_ref().map_or([0; 2], |r| r.base),
                        dimensions: e.dimensions,
                    })
                }
            })
            .collect();
        self.minimap_point(&views, client.world_entities.get(&0), |id, npc, index| {
            let fine = if npc {
                if id == 0 {
                    client.npc_state.npcs.get(&index).map(|n| n.path.position)
                } else {
                    entity(id)?.npcs.npcs.get(&index).map(|n| n.path.position)
                }
            } else {
                let state = client.player_state.as_ref()?;
                if id == 0 {
                    state.root_actor(index).map(|p| p.path.position)
                } else {
                    state
                        .views
                        .get(&id)?
                        .players
                        .get(&index)
                        .map(|p| p.path.position)
                }
            }?;
            let base = views.iter().find(|v| v.id == id)?.base;
            Some(std::array::from_fn(|i| {
                fine[i].wrapping_sub(base[i].wrapping_mul(128))
            }))
        })
        .map(|p| std::array::from_fn(|i| p[i].wrapping_add(client.world_base[i].wrapping_mul(128))))
    }
}

impl Hint {
    /// cm.af / mk.aa: scene hints use child terrain plus the root terrain at
    /// the entity origin, without actor animation or overlap-collapse matrices.
    pub fn scene_point(
        &self,
        views: &[crate::projectile_world::View],
        entities: Option<&crate::world_entity::State>,
        mut elevation: impl FnMut(u16, [i32; 2]) -> Option<i32>,
    ) -> Option<[i32; 3]> {
        let root = views.iter().find(|v| v.id == 0)?;
        let (view, fine) = match self.kind {
            2 => {
                let v = views
                    .iter()
                    .find(|v| {
                        v.id != 0
                            && (0..2).all(|i| {
                                self.tile[i] as i32 >= v.base[i]
                                    && (self.tile[i] as i32)
                                        < v.base[i].wrapping_add(v.dimensions[i])
                            })
                    })
                    .unwrap_or(root);
                (
                    v,
                    std::array::from_fn(|i| {
                        (self.tile[i] as i32)
                            .wrapping_sub(v.base[i])
                            .wrapping_mul(128)
                            .wrapping_add(self.offset[i])
                    }),
                )
            }
            4 => {
                let entity = entities?.entities.get(&self.index)?;
                let v = views.iter().find(|v| v.id == self.index)?;
                (v, entity.dimensions.map(|n| n.wrapping_mul(64)))
            }
            _ => return None,
        };
        let mut height =
            elevation(view.id, fine)?.wrapping_sub((self.height as i32).wrapping_mul(2));
        let mut horizontal = fine;
        if view.id != 0 {
            if let Some(entity) = entities.and_then(|s| s.entities.get(&view.id)) {
                let p = entity.transform_horizontal(fine);
                horizontal = [p[0] as i32, p[2] as i32];
                height = height.wrapping_add(elevation(0, [entity.pose.x, entity.pose.z])?);
            }
        }
        Some([horizontal[0], height, horizontal[1]])
    }
}
/// GameItemContainer.av fixed-point scene projection, including root borders.
pub fn project_scene(
    point: [i32; 3],
    dimensions: [i32; 2],
    eye: [i32; 3],
    angles: [i32; 2],
    size: [i32; 2],
    focal: i32,
) -> Option<[i32; 2]> {
    if point[0] < 128
        || point[2] < 128
        || point[0] > dimensions[0].wrapping_sub(2).wrapping_mul(128)
        || point[2] > dimensions[1].wrapping_sub(2).wrapping_mul(128)
    {
        return None;
    }
    let [x, y, z] = std::array::from_fn(|i| point[i].wrapping_sub(eye[i]));
    let trig = |angle: i32| {
        let r = (angle & 16383) as f64 * 3.834951969714103E-4;
        [(r.sin() * 65536.0) as i32, (r.cos() * 65536.0) as i32]
    };
    let [sp, cp] = trig(angles[0]);
    let [sy, cy] = trig(angles[1]);
    let rx = cy.wrapping_mul(x).wrapping_add(z.wrapping_mul(sy)) >> 16;
    let rz = cy.wrapping_mul(z).wrapping_sub(sy.wrapping_mul(x)) >> 16;
    let ry = y.wrapping_mul(cp).wrapping_sub(sp.wrapping_mul(rz)) >> 16;
    let depth = sp.wrapping_mul(y).wrapping_add(cp.wrapping_mul(rz)) >> 16;
    (depth >= 50).then(|| {
        [
            (size[0] / 2).wrapping_add(focal.wrapping_mul(rx) / depth),
            (size[1] / 2).wrapping_add(focal.wrapping_mul(ry) / depth),
        ]
    })
}
