//! Actual-cache packet application for duplicate child-world identities.
use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, game_packet, npc_info::Npc,
    protocol::Packet,
};
fn npc(point: i32) -> Npc {
    Npc {
        visual: Default::default(),
        path: {
            let mut p = gielinor_client::actor_path::Path::default();
            p.position = [point; 2];
            p
        },
        sequence: Default::default(),
        pose: Default::default(),
        idle_fallback: Default::default(),
        spots: Default::default(),
        overlays: Default::default(),
        turn: Default::default(),
        facing: None,
        freezes: Default::default(),
        exact: None,
        id: 16299,
        position: [point >> 7; 2],
        angle: 0,
        spawn_cycle: None,
        steps: Vec::new(),
        jump: false,
        blocks: Vec::new(),
        appearance: Default::default(),
        animations: [-1; 15],
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/world-duplicate-npc-dispatch-jvm.bin"
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
            if slot == present {
                let mut actor = npc(320);
                actor.id = 321 + slot as u16;
                e.npcs.npcs.insert(41, actor);
                e.npcs.indices = vec![41];
                e.npcs.insertion_order = vec![41];
                client.npcs.insert(
                    321 + slot as u32,
                    gielinor_client::npc::Definition::decode(321 + slot as u32, &[0])?,
                );
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
                9 => Action::Npc {
                    index: 41,
                    operation: 1,
                    sub: 7,
                },
                1003 => Action::ExamineNpc(41),
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
                "case{case} code{code} oldest registered NPC dispatch"
            );
            assert!(
                client.outbound.is_empty(),
                "unexpected extra outbound packet"
            );
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original duplicate NPC dispatch scenarios / {} actions match oldest registered view",count*2);
    Ok(())
}
