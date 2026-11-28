use gielinor_client::{
    actor_exact::Exact,
    actor_update::{Face, FaceTarget},
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    player_info, world_entity, Result,
};
use std::collections::BTreeMap;
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/player-live-facing-jvm.bin"
    ));
    assert_eq!(r.u32().unwrap(), 32);
    let initial = player_info::State::initialize(&[0; 4608], 1).unwrap().0;
    for case in 0..32 {
        let world = r.u32().unwrap();
        let target = r.u32().unwrap() as u16;
        let dx = r.u32().unwrap() as i32;
        let dz = r.u32().unwrap() as i32;
        let expected = [
            r.u32().unwrap() as i32,
            r.u32().unwrap() as i32,
            r.u32().unwrap() as i32,
            r.u32().unwrap() as i32,
        ];
        let mut players = BTreeMap::new();
        for id in [1, 2] {
            let mut p = initial.players[1].as_ref().unwrap().clone();
            p.position = [0, 12, 12];
            p.path.tiles[0] = [12; 2];
            p.path.position = [1600; 2];
            p.path.fine = [1600.; 2];
            players.insert(id, p);
        }
        players.get_mut(&target).unwrap().exact = Some(Exact {
            start: 10,
            end: -1,
            from: [12 + dx, 12 + dz],
            to: [0; 2],
            direction: 0,
        });
        players.get_mut(&(3 - target)).unwrap().facing = Some(Face {
            target: FaceTarget::Entity {
                kind: 2,
                index: target as i32,
                fallback: 777,
            },
            turn_on_walk: false,
            instant: false,
        });
        let mut state = player_info::State::initialize(&[0; 4608], 42)?.0;
        let mut local = initial.players[1].as_ref().unwrap().clone();
        local.position = [0, 12, 12];
        local.path.tiles[0] = [12; 2];
        local.path.position = [1600; 2];
        local.path.fine = [1600.; 2];
        state.high = vec![1, 2, 42];
        state.views.clear();
        let mut root_players = BTreeMap::new();
        root_players.insert(42, local);
        if world == 0 {
            root_players.extend(players);
        } else {
            state.views.insert(
                world as u16,
                player_info::View {
                    base: [0; 2],
                    dimensions: [24; 2],
                    players,
                },
            );
        }
        state.views.insert(
            0,
            player_info::View {
                base: [0; 2],
                dimensions: [104; 2],
                players: root_players,
            },
        );
        client.world_entities.clear();
        client.detached_world_entities.clear();
        client.npc_state = Default::default();
        client.view_registry = Default::default();
        if world != 0 {
            client.view_registry.insert(world as u16);
            let mut root = world_entity::State::default();
            let mut entity = world_entity::Entity::default();
            entity.dimensions = [24; 2];
            root.visible.push(world as u16);
            root.entities.insert(world as u16, entity);
            client.world_entities.insert(0, root);
        }
        client.player_state = Some(state);
        client.world_base = [0; 2];
        client.actor_cycle = 10;
        client.tick_actor_sequences()?;
        let players = &client.player_state.as_ref().unwrap().views[&(world as u16)].players;
        let observer = &players[&(3 - target)];
        let moving = &players[&target];
        assert_eq!(
            [
                observer.turn.target as i32,
                observer.turn.angle as i32,
                moving.path.position[0],
                moving.path.position[1]
            ],
            expected,
            "case{case} world{world} target{target} offset{dx},{dz}"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("32 original player live-target traces match cache-loaded ClientState in root and child worlds");
    Ok(())
}
