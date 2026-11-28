use std::collections::HashMap;
#[derive(Clone, Copy, Debug)]
pub struct Actor {
    pub npc: bool,
    pub index: u16,
    pub fine: [i32; 2],
    pub size: u8,
}
impl Actor {
    fn tag(self) -> u64 {
        if self.npc {
            0x4000000000000000 | self.index as u64
        } else {
            0x2000000000000000 | self.index as u64
        }
    }
}
pub fn expand(
    picks: &[(u64, u16)],
    views: &HashMap<u16, Vec<Actor>>,
    combat_target: i32,
) -> Vec<(u64, u16)> {
    let mut out = Vec::new();
    let mut deferred = None;
    for &(tag, view) in picks {
        let npc = tag & 0x4000000000000000 != 0;
        let player = tag & 0x2000000000000000 != 0;
        if !npc && !player {
            out.push((tag, view));
            continue;
        }
        let Some(actors) = views.get(&view) else {
            continue;
        };
        let Some(picked) = actors
            .iter()
            .find(|a| a.npc == npc && a.index == tag as u16)
        else {
            continue;
        };
        if (!npc || picked.size == 1) && picked.fine.iter().all(|v| v & 127 == 64) {
            for other in actors.iter().filter(|a| a.npc) {
                if other.size == 1
                    && other.fine == picked.fine
                    && (!npc || other.index != picked.index)
                {
                    out.push((other.tag(), view));
                }
            }
            for other in actors.iter().filter(|a| !a.npc) {
                if other.fine == picked.fine && (npc || other.index != picked.index) {
                    out.push((other.tag(), view));
                }
            }
        }
        if !npc && picked.index as i32 == combat_target {
            deferred = Some((tag, view));
        } else {
            out.push((tag, view));
        }
    }
    if let Some(pick) = deferred {
        out.push(pick);
    }
    out
}
/// Read the original NPC list and global high-resolution player order in one owning view.
pub fn actors(client: &crate::client_state::ClientState, view: u16) -> Vec<Actor> {
    let mut actors = Vec::new();
    let npcs = if view == 0 {
        Some(&client.npc_state)
    } else {
        client
            .world_entities
            .get(&0)
            .and_then(|w| w.entities.get(&view))
            .map(|e| &e.npcs)
    };
    if let Some(npcs) = npcs {
        for &index in &npcs.indices {
            if let Some(npc) = npcs.npcs.get(&index) {
                if let Some(def) = client.npcs.get(&(npc.id as u32)) {
                    actors.push(Actor {
                        npc: true,
                        index,
                        fine: npc.path.position,
                        size: def.size,
                    });
                }
            }
        }
    }
    if let Some(players) = &client.player_state {
        for &index in &players.high {
            if let Some(player) = players.root_owned_actor(view, index, &client.world_entities) {
                actors.push(Actor {
                    npc: false,
                    index,
                    fine: player.path.position,
                    size: 1,
                });
            }
        }
    }
    actors
}
/// Menu construction resolves transforms now, independently of the last rendered definition.
pub fn npc_definition(
    client: &mut crate::client_state::ClientState,
    id: u32,
) -> crate::Result<Option<u32>> {
    use crate::script_runtime::Source;
    let Some(base) = client.npcs.get(&id) else {
        return Ok(None);
    };
    let bit = if let Some(id) = base.transform_varbit {
        let bit = client.scripts.source.varbit(id as i32)?;
        bit.get(client.scripts.variables.player(bit.base))
    } else {
        -1
    };
    let varp = base
        .transform_varp
        .map_or(-1, |id| client.scripts.variables.player(id));
    Ok(base.transformed_id(bit, varp))
}
