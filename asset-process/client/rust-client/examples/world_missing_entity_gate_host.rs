//! Replay original policy mutations through the actual client action gate.
use gielinor_client::{
    buffer::Reader, client_state::ClientState, context_menu::Action, disk_cache::DiskCache,
    game_packet, protocol::Packet,
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
        "../tests/fixtures/interaction-policy-jvm.bin"
    ));
    let count = r.u32()?;
    let mut decisions = 0;
    for case in 0..count {
        let opcode = r.u32()? as u16;
        let n = if opcode == 66 { 2 } else { 4 };
        c.apply(
            game_packet::decode(&Packet {
                opcode,
                payload: r.bytes(n)?.to_vec(),
            })?
            .unwrap(),
        )?;
        let view = r.u32()? as u16;
        let local = r.u32()? as u16;
        let expected = [r.u32()? as u8, r.u32()? as u8];
        c.player_state.as_mut().unwrap().local_view = local;
        assert_eq!(
            c.interaction_policy.get(view as i16),
            expected,
            "case{case} policy"
        );
        for _ in 0..r.u32()? {
            let code = r.u32()?;
            let expected = r.u8()? != 0;
            if let Some(action) = action(code) {
                decisions += 1;
                assert_eq!(
                    c.scene_action_allowed(action.in_view(view))?,
                    expected,
                    "case{case} view{view} local{local} code{code} missing-root-entity policy"
                );
            }
        }
    }
    for _ in 0..r.u32()? {
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u8()?;
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original policy mutations / {decisions} decisions match ClientState with absent root entities");
    Ok(())
}
