//! Quantized world-entity bounds and actor intersection (vw/vr, gc.cx, de.as).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub corners: [[i32; 2]; 4],
    pub centre: [i32; 2],
    pub radius_squared: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct Actor {
    pub fine: [i32; 2],
    pub size: i32,
    pub npc: bool,
    pub has_operations: bool,
}
/// GameGraphicsObject.cp: all player copies, NPCs with base operations, and
/// other entities whose scene was already submitted in the current frame.
pub fn obscured(
    bounds: [i32; 4],
    position: [i32; 2],
    yaw: i32,
    actors: impl IntoIterator<Item = Actor>,
    submitted: impl IntoIterator<Item = EntityBox>,
) -> bool {
    let polygon = Bounds::new(bounds, position, yaw);
    if actors
        .into_iter()
        .any(|a| (!a.npc || a.has_operations) && polygon.overlaps_actor(a.fine, a.size))
    {
        return true;
    }
    let area = EntityBox::new(bounds, position, yaw);
    submitted.into_iter().any(|other| area.overlaps(&other))
}
/// oj.ae uses the full yaw and fixed-point trig values converted to doubles.
/// Its final box includes an extra coordinate unit and tu.ax includes its edges.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntityBox {
    pub origin: [i32; 2],
    pub size: [i32; 2],
}
impl EntityBox {
    pub fn new(bounds: [i32; 4], position: [i32; 2], yaw: i32) -> Self {
        let [ox, oz, width, depth] = bounds;
        let angle = (yaw & 2047) as f64 * 0.0030679615757712823;
        let sin = (65536.0 * libm::sin(angle)) as i32 as f64 / 65536.0;
        let cos = (65536.0 * libm::cos(angle)) as i32 as f64 / 65536.0;
        let [w, h] = [(width / 2) as f64, (depth / 2) as f64];
        let x = position[0].wrapping_add((ox as f64 * cos - oz as f64 * sin) as i32);
        let z = position[1].wrapping_add((sin * ox as f64 + cos * oz as f64) as i32);
        let a = (cos * w - sin * h) as i32;
        let b = (cos * h + w * sin) as i32;
        let c = (sin * h + w * cos) as i32;
        let d = (h * cos - sin * w) as i32;
        let xs = [
            x.wrapping_sub(a),
            x.wrapping_add(c),
            x.wrapping_add(a),
            x.wrapping_sub(c),
        ];
        let zs = [
            z.wrapping_add(b),
            z.wrapping_add(d),
            z.wrapping_sub(b),
            z.wrapping_sub(d),
        ];
        let origin = [*xs.iter().min().unwrap(), *zs.iter().min().unwrap()];
        Self {
            origin,
            size: [
                xs.iter()
                    .max()
                    .unwrap()
                    .wrapping_sub(origin[0])
                    .wrapping_add(1),
                zs.iter()
                    .max()
                    .unwrap()
                    .wrapping_sub(origin[1])
                    .wrapping_add(1),
            ],
        }
    }
    pub fn overlaps(&self, other: &Self) -> bool {
        (0..2).all(|i| {
            self.origin[i] <= other.origin[i].wrapping_add(other.size[i])
                && other.origin[i] <= self.origin[i].wrapping_add(self.size[i])
        })
    }
}
impl Bounds {
    pub fn new(bounds: [i32; 4], position: [i32; 2], yaw: i32) -> Self {
        let [ox, oz, width, depth] = bounds;
        let angle = ((yaw.wrapping_add(64) & 2047) / 128 * 128) as f64 * 0.0030679615757712823;
        let sin = (65536.0 * libm::sin(angle)) as i32;
        let cos = (65536.0 * libm::cos(angle)) as i32;
        let [w, h] = [width / 2, depth / 2];
        let cx =
            position[0].wrapping_sub(ox.wrapping_mul(cos).wrapping_sub(oz.wrapping_mul(sin)) >> 16);
        let cz =
            position[1].wrapping_add(sin.wrapping_mul(ox).wrapping_add(oz.wrapping_mul(cos)) >> 16);
        let a = w.wrapping_mul(cos).wrapping_sub(h.wrapping_mul(sin)) >> 16;
        let b = sin.wrapping_mul(w).wrapping_add(h.wrapping_mul(cos)) >> 16;
        let c = w.wrapping_mul(cos).wrapping_add(h.wrapping_mul(sin)) >> 16;
        let d = h.wrapping_mul(cos).wrapping_sub(sin.wrapping_mul(w)) >> 16;
        Self {
            corners: [
                [cx.wrapping_sub(a), cz.wrapping_add(b)],
                [cx.wrapping_add(c), cz.wrapping_add(d)],
                [cx.wrapping_add(a), cz.wrapping_sub(b)],
                [cx.wrapping_sub(c), cz.wrapping_sub(d)],
            ],
            centre: [cx, cz],
            radius_squared: w.wrapping_mul(w).wrapping_add(h.wrapping_mul(h)),
        }
    }
    pub fn overlaps_actor(&self, fine: [i32; 2], size: i32) -> bool {
        let half = size.wrapping_mul(64);
        self.overlaps_rect([
            fine[0].wrapping_sub(half),
            fine[1].wrapping_sub(half),
            fine[0].wrapping_add(half).wrapping_sub(1),
            fine[1].wrapping_add(half).wrapping_sub(1),
        ])
    }
    pub fn overlaps_rect(&self, rect: [i32; 4]) -> bool {
        let [x0, z0, x1, z1] = rect;
        let dx = self.centre[0].wrapping_sub(x0.max(self.centre[0].min(x1)));
        let dz = self.centre[1].wrapping_sub(z0.max(self.centre[1].min(z1)));
        if dz.wrapping_mul(dz).wrapping_add(dx.wrapping_mul(dx)) > self.radius_squared {
            return false;
        }
        // The reference excludes the outer one-unit rectangle boundary.
        let [x0, z0, x1, z1] = [
            x0.wrapping_add(1),
            z0.wrapping_add(1),
            x1.wrapping_sub(1),
            z1.wrapping_sub(1),
        ];
        if self
            .corners
            .iter()
            .any(|&[x, z]| x >= x0 && x <= x1 && z >= z0 && z <= z1)
        {
            return true;
        }
        let rect = [[x0, z0], [x1, z0], [x1, z1], [x0, z1]];
        if rect.iter().any(|&p| {
            projection(p, self.corners[0], self.corners[1])
                && projection(p, self.corners[1], self.corners[2])
        }) {
            return true;
        }
        (0..4).any(|i| {
            (0..4).any(|j| {
                segments(
                    self.corners[i],
                    self.corners[(i + 1) % 4],
                    rect[j],
                    rect[(j + 1) % 4],
                )
            })
        })
    }
}
fn projection(p: [i32; 2], a: [i32; 2], b: [i32; 2]) -> bool {
    let dx = b[0].wrapping_sub(a[0]);
    let dz = b[1].wrapping_sub(a[1]);
    let dot = dx
        .wrapping_mul(p[0].wrapping_sub(a[0]))
        .wrapping_add(p[1].wrapping_sub(a[1]).wrapping_mul(dz));
    dot >= 0 && dot <= dz.wrapping_mul(dz).wrapping_add(dx.wrapping_mul(dx))
}
fn slope(a: [i32; 2], b: [i32; 2]) -> i64 {
    if a[0] == b[0] {
        return if a[1] > b[1] { -(1 << 30) } else { 1 << 30 };
    }
    ((b[1].wrapping_sub(a[1]) as i64) << 15)
        .wrapping_div(b[0].wrapping_sub(a[0]) as i64)
        .clamp(-(1 << 30), 1 << 30)
}
fn segments(a: [i32; 2], b: [i32; 2], c: [i32; 2], d: [i32; 2]) -> bool {
    let s = slope(a, b);
    let t = slope(c, d);
    if s == t {
        return false;
    }
    let u = ((a[1] as i64) << 15).wrapping_sub((a[0] as i64).wrapping_mul(s));
    let v = ((c[1] as i64) << 15).wrapping_sub(t.wrapping_mul(c[0] as i64));
    let x = (u.wrapping_sub(v).wrapping_div(s.wrapping_sub(t)) as i32).wrapping_neg();
    if x < a[0].min(b[0]) || x > a[0].max(b[0]) || x < c[0].min(d[0]) || x > c[0].max(d[0]) {
        return false;
    }
    let z = if s.abs() < t.abs() {
        u.wrapping_add((x as i64).wrapping_mul(s))
    } else {
        t.wrapping_mul(x as i64).wrapping_add(v)
    } >> 15;
    z >= a[1].min(b[1]) as i64
        && z <= a[1].max(b[1]) as i64
        && z >= c[1].min(d[1]) as i64
        && z <= c[1].max(d[1]) as i64
}

/// hy.ch followed by cw.cq for categories 2, 0, and 1. The selected
/// local view bypasses the category limit and is never tested for overlap.
pub fn submission_order(state: &crate::world_entity::State, local: u16, limit: usize) -> Vec<u16> {
    let mut order = Vec::new();
    if local != 0 && state.entities.contains_key(&local) {
        order.push(local);
    }
    for category in [2, 0, 1] {
        order.extend(
            state
                .visible
                .iter()
                .copied()
                .filter(|id| {
                    *id != local
                        && state
                            .entities
                            .get(id)
                            .is_some_and(|e| e.owner_type == category)
                })
                .take(limit),
        );
    }
    order
}
/// Resolve parent-view actor copies in the same coordinates as entity poses.
pub fn parent_actors(client: &crate::client_state::ClientState, parent: u16) -> Vec<Actor> {
    let base = if parent == 0 {
        client.world_base
    } else {
        client
            .world_entities
            .values()
            .find_map(|s| s.entities.get(&parent))
            .and_then(|e| e.rebuild.as_ref())
            .map_or([0; 2], |r| r.base)
    };
    let mut actors = Vec::new();
    if let Some(state) = &client.player_state {
        for &index in &state.high {
            let player = if parent == 0 {
                state.root_actor(index)
            } else {
                state.views.get(&parent).and_then(|v| v.players.get(&index))
            };
            if let Some(player) = player {
                let size = player
                    .appearance
                    .as_ref()
                    .and_then(|a| a.appearance.npc)
                    .and_then(|id| client.npcs.get(&(id as u32)))
                    .map_or(1, |d| d.size as i32);
                actors.push(Actor {
                    fine: [
                        player.path.position[0] - base[0] * 128,
                        player.path.position[1] - base[1] * 128,
                    ],
                    size,
                    npc: false,
                    has_operations: true,
                });
            }
        }
    }
    let npcs = if parent == 0 {
        Some(&client.npc_state)
    } else {
        client
            .world_entities
            .values()
            .find_map(|s| s.entities.get(&parent))
            .map(|e| &e.npcs)
    };
    if let Some(npcs) = npcs {
        for index in &npcs.indices {
            if let Some(npc) = npcs.npcs.get(index) {
                if let Some(def) = client.npcs.get(&(npc.id as u32)) {
                    actors.push(Actor {
                        fine: [
                            npc.path.position[0] - base[0] * 128,
                            npc.path.position[1] - base[1] * 128,
                        ],
                        size: def.size as i32,
                        npc: true,
                        has_operations: def
                            .operations
                            .iter()
                            .any(|op| op.sub.is_none() && op.condition.is_none()),
                    });
                }
            }
        }
    }
    actors
}

/// Root-owned child views in the original local/category submission order.
pub fn frame_order(
    worlds: &std::collections::HashMap<u16, crate::world_entity::State>,
    local: u16,
    limit: usize,
) -> crate::Result<Vec<(u16, u16)>> {
    // pj.cb/hy.ch/cw.cq prepare only root-owned child scenes. Registration of
    // another view under a child does not recursively submit it for drawing.
    Ok(worlds.get(&0).map_or_else(Vec::new, |state| {
        submission_order(state,local,limit).into_iter().map(|id|(0,id)).collect()
    }))
}

/// Scene submission stamps survive a second draw in the same client cycle.
/// Compare current poses, and always exclude the candidate itself (cp).
pub fn submitted_boxes(
    state: &crate::world_entity::State,
    candidate: u16,
    cycle: i32,
    stamps: &std::collections::HashMap<u16, i32>,
) -> Vec<EntityBox> {
    state
        .visible
        .iter()
        .filter_map(|id| {
            if *id == candidate || stamps.get(id) != Some(&cycle) {
                return None;
            }
            let entity = state.entities.get(id)?;
            Some(EntityBox::new(
                entity.definition.as_ref().map_or([0; 4], |d| d.bounds),
                [entity.pose.x, entity.pose.z],
                entity.pose.angle,
            ))
        })
        .collect()
}

/// nq.ct uses visible, current-cycle root entity scenes for all NPC copies.
pub fn visible_bounds(
    state: Option<&crate::world_entity::State>,
    cycle: i32,
    stamps: &std::collections::HashMap<u16, i32>,
    hidden: &std::collections::HashSet<u16>,
) -> Vec<Bounds> {
    state
        .into_iter()
        .flat_map(|s| {
            s.visible.iter().filter_map(|id| {
                if stamps.get(id) != Some(&cycle) || hidden.contains(id) {
                    return None;
                }
                let e = s.entities.get(id)?;
                Some(Bounds::new(
                    e.definition.as_ref().map_or([0; 4], |d| d.bounds),
                    [e.pose.x, e.pose.z],
                    e.pose.angle,
                ))
            })
        })
        .collect()
}
pub fn npc_obscured(enabled: bool, fine: [i32; 2], size: i32, bounds: &[Bounds]) -> bool {
    enabled && bounds.iter().any(|b| b.overlaps_actor(fine, size))
}
/// GameNPC.ak and GameActorSpotAnim: vertical scale precedes the -5 offset.
pub fn collapse_npc_height(y: f32) -> f32 {
    y * 0.01 - 5.0
}
pub fn collapse_npc_model(model: &mut crate::model::Model) {
    for vertex in &mut model.vertices {
        vertex[1] = collapse_npc_height(vertex[1]);
    }
}
