use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, game_packet, protocol::Packet,
};
fn dispatch(
    c: &mut ClientState,
    opcode: u16,
    payload: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    c.apply(game_packet::decode(&Packet { opcode, payload })?.unwrap())?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut c = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/world-self-player-spot-jvm.bin"
    ));
    let count = r.u32()?;
    for case in 0..count {
        c.world_entities.clear();
        c.detached_world_entities.clear();
        c.active_world = 0;
        c.view_registry = Default::default();
        let sizes = [r.u32()? as i32, r.u32()? as i32];
        for _ in 0..r.u32()? {
            let op = r.u32()? as u16;
            let n = r.u32()? as usize;
            dispatch(&mut c, op, r.bytes(n)?.to_vec())?;
        }
        let generations = [
            c.world_entities[&0].entities[&7].spawn_generation,
            c.world_entities[&7].entities[&7].spawn_generation,
        ];
        let n = r.u32()? as usize;
        let initial = r.bytes(n)?;
        assert_eq!(c.initialize_players(initial, 42)?, initial.len());
        let n = r.u32()? as usize;
        dispatch(&mut c, 28, r.bytes(n)?.to_vec())?;
        for ((g, s), size) in generations.into_iter().zip([3, 4]).zip(sizes) {
            let v = c
                .player_state
                .as_mut()
                .unwrap()
                .instance_view_mut(7, g)
                .unwrap();
            assert_eq!(v.dimensions, [size; 2]);
            v.players.get_mut(&42).unwrap().sequence.id = s;
        }
        dispatch(&mut c, 122, vec![0])?;
        let expected = (0..6)
            .map(|_| r.u8().map(|v| v != 0))
            .collect::<Result<Vec<_>, _>>()?;
        let p = c.player_state.as_ref().unwrap();
        let actual = vec![
            c.active_world == 7,
            c.view_entity(7).unwrap().spawn_generation == generations[1],
            p.referenced_view(7, generations[0])
                .is_some_and(|v| v.players.contains_key(&42)),
            p.referenced_view(7, generations[1])
                .is_some_and(|v| v.players.contains_key(&42)),
            c.world_entities[&0].entities.contains_key(&7),
            c.world_entities[&7].entities.contains_key(&7),
        ];
        assert_eq!(actual, expected, "case{case} lifetime");
        dispatch(&mut c, 92, vec![])?;
        for g in generations {
            let expected = r.u32()? as i32;
            assert_eq!(
                c.player_state
                    .as_ref()
                    .unwrap()
                    .referenced_view(7, g)
                    .unwrap()
                    .players
                    .get(&42)
                    .unwrap()
                    .sequence
                    .id,
                expected,
                "case{case} selected reset"
            );
        }
        c.actor_cycle = 10;
        dispatch(&mut c, 72, vec![0, 251, 0, 2, 0, 3, 127, 0, 42])?;
        for g in generations {
            let expected = [r.u32()? as i32, r.u32()? as i32, r.u32()? as i32];
            let effect = c
                .player_state
                .as_ref()
                .unwrap()
                .referenced_view(7, g)
                .unwrap()
                .players[&42]
                .spots
                .slots
                .get(&1);
            assert_eq!(
                effect.map_or([-1; 3], |e| [e.id as i32, e.height, e.start]),
                expected,
                "case{case} retired selected spot"
            );
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original same-ID parent/child removals preserve selected player lifetime and spot effects");
    Ok(())
}
