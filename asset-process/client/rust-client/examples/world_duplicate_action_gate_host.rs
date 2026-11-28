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
        "../tests/fixtures/world-duplicate-action-gate-jvm.bin"
    ));
    let count = r.u32()?;
    for case in 0..count {
        client.player_state = None;
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
        let mode = r.u32()? as u8;
        let owner = r.u32()? as u8;
        let local = r.u32()? != 0;
        let mut state = gielinor_client::player_info::State::initialize(&vec![0; 4608], 1)?.0;
        state.local_view = if local { id } else { 0 };
        client.player_state = Some(state);
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
                &[24, if slot == 0 { mode } else { 2 - mode }, 0],
            )?);
            e.owner_type = if slot == 0 { owner } else { 2 - owner };
        }
        for _ in 0..r.u32()? {
            use gielinor_client::context_menu::Action;
            let opcode = r.u32()?;
            let expected = r.u8()? != 0;
            let action = match opcode {
                3 => Action::Object {
                    id: 1,
                    tile: [1, 2],
                    operation: 1,
                    sub: 0,
                },
                9 => Action::Npc {
                    index: 1234,
                    operation: 1,
                    sub: 0,
                },
                18 => Action::Ground {
                    id: 1,
                    tile: [1, 2],
                    operation: 1,
                    sub: 0,
                },
                44 => Action::Player {
                    index: 2,
                    operation: 1,
                },
                1002 => Action::Examine {
                    kind: 1,
                    id: 1,
                    tile: [1, 2],
                },
                1003 => Action::ExamineNpc(1234),
                1004 => Action::Examine {
                    kind: 2,
                    id: 1,
                    tile: [1, 2],
                },
                63 => Action::WorldEntity {
                    id: 8,
                    operation: 1,
                },
                1013 => Action::WorldEntity {
                    id: 1,
                    operation: 6,
                },
                _ => unreachable!(),
            };
            assert_eq!(client.scene_action_allowed(action.in_view(id))?,expected,
                "case {case}: view {id}, mode {mode}, owner {owner}, local {local}, opcode {opcode}");
        }
    }
    assert_eq!(r.remaining(), 0);
    println!(
        "{count} original duplicate-world action-gate scenarios / {} decisions match ClientState",
        count * 9
    );
    Ok(())
}
