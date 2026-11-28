use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, game_packet,
    protocol::Packet, scene::Scene, world_actors::Actors, world_mesh::WorldMesh,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut c = ClientState::load(cache)?;
    let mut actors = Actors::load(&mut c.scripts.source.cache)?;
    let definitions = gielinor_client::object::Definition::load_all(&mut c.scripts.source.cache)?;
    let mut ids = definitions
        .keys()
        .copied()
        .filter(|id| !definitions[id].model_ids(10).is_empty())
        .collect::<Vec<_>>();
    ids.sort_unstable();
    let mut models = gielinor_client::object_model::Models::default();
    let mut selected = None;
    for id in ids {
        if let Some(model) = models.object(&mut c.scripts.source.cache, &definitions[&id], 10, 0)? {
            if !model.faces.is_empty() {
                selected = Some(definitions[&id].clone());
                break;
            }
        }
    }
    let mut def = selected.expect("real cache object model");
    def.id = 100;
    def.size = [1, 1];
    def.contour = -1;
    def.animation = None;
    def.transforms = None;
    let definitions = std::collections::HashMap::from([(100, def)]);
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/world-duplicate-location-merge-jvm.bin"
    ));
    let count = r.u32()?;
    for case in 0..count {
        c.world_entities = Default::default();
        c.detached_world_entities.clear();
        c.active_world = 0;
        c.view_registry = Default::default();
        let id = r.u32()? as u16;
        let _order = r.u32()?;
        for _ in 0..r.u32()? {
            let opcode = r.u32()? as u16;
            let n = r.u32()? as usize;
            let payload = r.bytes(n)?.to_vec();
            c.apply(game_packet::decode(&Packet { opcode, payload })?.unwrap())?;
        }
        let _oldest = r.u32()?;
        let direct_size = r.u32()? as usize;
        let _nested = r.u32()?;
        let n = r.u32()? as usize;
        let initial = r.bytes(n)?;
        assert_eq!(c.initialize_players(initial, 42)?, initial.len());
        let n = r.u32()? as usize;
        c.apply(
            game_packet::decode(&Packet {
                opcode: 28,
                payload: r.bytes(n)?.to_vec(),
            })?
            .unwrap(),
        )?;
        let generations = [
            c.world_entities[&0].entities[&id].spawn_generation,
            c.world_entities[&7].entities[&id].spawn_generation,
        ];
        c.actor_cycle = 100;
        c.zone.origin = [0, 2, 2];
        c.apply(
            game_packet::decode(&Packet {
                opcode: 47,
                payload: vec![(id >> 8) as u8, id as u8, 0],
            })?
            .unwrap(),
        )?;
        let n = r.u32()? as usize;
        c.apply(
            game_packet::decode(&Packet {
                opcode: 34,
                payload: r.bytes(n)?.to_vec(),
            })?
            .unwrap(),
        )?;
        let mut expected = Vec::new();
        for _ in 0..2 {
            expected.push(if r.u8()? != 0 {
                let mut a = [0; 9];
                for v in &mut a {
                    *v = r.u32()? as i32;
                }
                Some(a)
            } else {
                None
            });
        }
        let mut scene = Scene::empty_sized(0, 0, direct_size, direct_size)?;
        let mut mesh = WorldMesh::build(&scene, &mut c.scripts.source.cache, 0)?;
        actors.child_locations(&mut c, &mut scene, &mut mesh, id, 0, &definitions)?;
        for (slot, generation) in generations.into_iter().enumerate() {
            let player = &c
                .player_state
                .as_ref()
                .unwrap()
                .instance_view(id, generation)
                .unwrap()
                .players[&42];
            let actual = player.attachment.as_ref().map(|a| {
                let p = &a.placement;
                [
                    p.start,
                    p.end,
                    p.centre[0],
                    p.centre[2],
                    p.centre[1],
                    p.bounds[0],
                    p.bounds[2],
                    p.bounds[1],
                    p.bounds[3],
                ]
            });
            assert_eq!(
                actual, expected[slot],
                "case{case} slot{slot} selected location attachment"
            );
        }
    }
    assert_eq!(r.remaining(), 0);
    println!(
        "{count} original duplicate-world location merge attachments match actual-cache processing"
    );
    Ok(())
}
