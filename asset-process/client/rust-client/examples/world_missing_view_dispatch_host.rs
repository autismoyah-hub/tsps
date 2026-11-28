//! Verify original absent-view gates and silent dispatch through ClientState.
use gielinor_client::{
    buffer::Reader, client_state::ClientState, context_menu::Action, disk_cache::DiskCache,
};
fn action(code: u32) -> Option<Action> {
    let code = if code >= 2000 { code - 2000 } else { code };
    Some(match code {
        1..=6 | 1001 => Action::Object {
            id: 1,
            tile: [1, 2],
            operation: 1,
            sub: 0,
        },
        7..=13 => Action::Npc {
            index: 1,
            operation: 1,
            sub: 0,
        },
        14 | 15 | 44..=51 => Action::Player {
            index: 1,
            operation: 1,
        },
        16..=22 => Action::Ground {
            id: 1,
            tile: [1, 2],
            operation: 1,
            sub: 0,
        },
        1002 => Action::Examine {
            kind: 1,
            id: 1,
            tile: [1, 2],
        },
        1003 => Action::ExamineNpc(1),
        1004 => Action::Examine {
            kind: 2,
            id: 1,
            tile: [1, 2],
        },
        1013 => Action::WorldEntity {
            id: 1,
            operation: 6,
        },
        _ => return None,
    })
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut c = ClientState::load(cache)?;
    let mut initial = vec![0; 4608];
    let position = (1u32 << 14) | 1;
    for bit in 0..30 {
        if position & (1 << (29 - bit)) != 0 {
            initial[bit / 8] |= 1 << (7 - bit % 8);
        }
    }
    c.initialize_players(&initial, 42)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/world-missing-view-dispatch-jvm.bin"
    ));
    let count = r.u32()?;
    let mut decisions = 0;
    for case in 0..count {
        let view = r.u32()? as u16;
        let profile = r.u32()? as u8;
        let local = r.u32()? != 0;
        let control = r.u32()? != 0;
        c.player_state.as_mut().unwrap().local_view = if local { view } else { 0 };
        c.interaction_policy
            .apply(gielinor_client::interaction_policy::Update::Set {
                world: view as i16,
                input: 1,
                actions: profile,
            });
        if control {
            c.input.held.insert(82);
        } else {
            c.input.held.remove(&82);
        }
        for _ in 0..r.u32()? {
            let code = r.u32()?;
            let expected_gate = r.u8()? != 0;
            let expected_sent = r.u8()? != 0;
            let a = if code == 63 {
                Action::WorldEntity {
                    id: 41,
                    operation: 1,
                }
            } else {
                action(code).unwrap()
            }
            .in_view(view);
            assert_eq!(
                c.scene_action_allowed(a)?,
                expected_gate,
                "case{case} view{view} code{code} absent entity gate"
            );
            c.outbound.clear();
            c.menu_action(a)?;
            assert_eq!(
                !c.outbound.is_empty(),
                expected_sent,
                "case{case} view{view} code{code} absent view dispatch"
            );
            decisions += 1;
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original absent-view scenarios / {decisions} gates and suppressed dispatches match ClientState");
    Ok(())
}
