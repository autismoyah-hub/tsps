//! Actual-cache packet application for duplicate child-world identities.
use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, game_packet, protocol::Packet,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/world-duplicate-player-dispatch-jvm.bin"
    ));
    let count = r.u32()?;
    for case in 0..count {
        client.player_state = None;
        client.npc_state = Default::default();
        client.world_entities.clear();
        client.detached_world_entities.clear();
        client.active_world = 0;
        client.view_registry = Default::default();
        let id = r.u32()? as u16;
        let _order = r.u32()?;
        for _ in 0..r.u32()? {
            let opcode = r.u32()? as u16;
            let n = r.u32()? as usize;
            let payload = r.bytes(n)?.to_vec();
            client.apply(game_packet::decode(&Packet { opcode, payload })?.unwrap())?;
        }
        let expected = r.u32()? as i32;
        let direct = r.u32()? as i32;
        let nested = r.u32()? as i32;
        assert_eq!(client.view_entity(id).unwrap().dimensions, [expected; 2]);
        assert_eq!(
            client.world_entities[&0].entities[&id].dimensions,
            [direct; 2]
        );
        assert_eq!(
            client.world_entities[&7].entities[&id].dimensions,
            [nested; 2]
        );
        let present = r.u32()? as usize;
        let control = r.u32()? != 0;
        let mut initial = vec![0; 4608];
        let position = (1u32 << 14) | 1;
        for bit in 0..30 {
            if position & (1 << (29 - bit)) != 0 {
                initial[bit / 8] |= 1 << (7 - bit % 8);
            }
        }
        client.initialize_players(&initial, 42)?;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 28,
                payload: vec![0, 127, 244],
            })?
            .unwrap(),
        )?;
        client.player_state.as_mut().unwrap().local = 0;
        client.player_state.as_mut().unwrap().local_view = 0;
        for (slot, parent) in [0, 7].into_iter().enumerate() {
            let e = client
                .world_entities
                .get_mut(&parent)
                .unwrap()
                .entities
                .get_mut(&id)
                .unwrap();
            e.definition = Some(gielinor_client::world_entity::Definition::decode(
                0,
                &[14, 23, 1, 0],
            )?);
            if slot != present {
                client
                    .player_state
                    .as_mut()
                    .unwrap()
                    .instance_view_mut(id, e.spawn_generation)
                    .unwrap()
                    .players
                    .remove(&42);
            }
        }
        if control {
            client.input.held.insert(82);
        } else {
            client.input.held.remove(&82);
        }
        for _ in 0..r.u32()? {
            let code = r.u32()?;
            let sent = r.u8()? != 0;
            let expected = if sent {
                let opcode = r.u32()? as u16;
                let n = r.u32()? as usize;
                Some((opcode, r.bytes(n)?.to_vec()))
            } else {
                None
            };
            use gielinor_client::context_menu::Action;
            let action = match code {
                44 => Action::Player {
                    index: 42,
                    operation: 1,
                },
                51 => Action::Player {
                    index: 42,
                    operation: 8,
                },
                _ => unreachable!(),
            };
            client.outbound.clear();
            client.menu_action(action.in_view(id))?;
            let actual = client
                .outbound
                .pop_front()
                .map(|packet| (packet.opcode, packet.payload));
            assert_eq!(
                actual, expected,
                "case{case} code{code} oldest registered player dispatch"
            );
            assert!(
                client.outbound.is_empty(),
                "unexpected extra outbound packet"
            );
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original duplicate player dispatch scenarios / {} actions match oldest registered view",count*2);
    Ok(())
}
