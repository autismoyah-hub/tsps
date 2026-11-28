use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, game_packet,
    player_appearance, protocol::Packet, scene::Scene,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut c = ClientState::load(cache)?;
    let mut live = gielinor_client::live_runtime::Connection::connect(
        &mut c,
        "127.0.0.1:43617",
        "rustclient239",
        "work/kotlin-world/test-account.password",
        "work/kotlin-world/.data/client.key",
    )?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(90);
    while live.world.is_none() || !live.session.is_established() || live.minimap_view(&c).is_none()
    {
        live.poll(&mut c)?;
        assert!(std::time::Instant::now() < deadline, "scene login deadline");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    // Freeze polling and replay the controlled local scenario without sending fixture packets.
    let scene = Scene::empty(0, 0)?;
    let mesh =
        gielinor_client::world_mesh::WorldMesh::build(&scene, &mut c.scripts.source.cache, 0)?;
    live.world = Some((scene, mesh));
    c.world_base = [0, 0];
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/world-duplicate-player-minimap-jvm.bin"
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
        let _oldest_size = r.u32()?;
        let _direct_size = r.u32()? as usize;
        let _nested_size = r.u32()?;
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
        let hidden = r.u32()?;
        let plane_case = r.u32()?;
        c.world_entities
            .get_mut(&0)
            .unwrap()
            .entities
            .get_mut(&id)
            .unwrap()
            .plane = if plane_case == 0 { 0 } else { 1 };
        c.world_entities
            .get_mut(&7)
            .unwrap()
            .entities
            .get_mut(&id)
            .unwrap()
            .plane = if plane_case == 0 { 0 } else { 2 };
        let generations = [
            c.world_entities[&0].entities[&id].spawn_generation,
            c.world_entities[&7].entities[&id].spawn_generation,
        ];
        for (slot, generation) in generations.into_iter().enumerate() {
            let actor = c
                .player_state
                .as_mut()
                .unwrap()
                .instance_view_mut(id, generation)
                .unwrap()
                .players
                .get_mut(&42)
                .unwrap();
            actor.path.position = [320 + 128 * slot as i32; 2];
            actor.position[0] = if plane_case == 0 { 0 } else { slot as i32 + 1 };
            actor.appearance = Some(player_appearance::Update {
                appearance: gielinor_client::player_model::Appearance {
                    npc: Some(16299),
                    ..Default::default()
                },
                skull: -1,
                overhead: -1,
                animations: [-1; 7],
                name: "fixture".into(),
                combat: 3,
                skill: 0,
                hidden: if slot == 0 { hidden == 1 } else { hidden == 0 },
                force_refresh: false,
                actions: Default::default(),
                team: 0,
            });
        }
        let expected = (0..r.u32()?)
            .map(|_| r.u32())
            .collect::<Result<Vec<_>, _>>()?;
        let _nested = (0..r.u32()?)
            .map(|_| r.u32())
            .collect::<Result<Vec<_>, _>>()?;
        c.player_state.as_mut().unwrap().local_view = 0;
        c.player_state.as_mut().unwrap().root_actor_mut(42).unwrap().path.position = [-1024, -1024];
        let view = live.minimap_view(&c).expect("minimap prepared");
        let mut raster = gielinor_client::raster::Raster::new(180, 180, 180 * 180)?;
        let sprite = gielinor_client::sprite_pixels::Pixels {
            canvas: [1, 1],
            offset: [0, 0],
            size: [1, 1],
            pixels: vec![0xffffff],
        };
        let mask = gielinor_client::minimap::Mask {
            width: 150,
            starts: vec![0; 150],
            widths: vec![150; 150],
        };
        for (delta, dot) in view.entities.iter().flat_map(|e| e.markers.iter()) {
            if *dot >= 2 {
                gielinor_client::minimap::marker(&sprite, &mut raster, [15, 15], *delta, 0, &mask)?;
            }
        }
        let actual = raster
            .pixels()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| (*p != 0).then_some(i as u32))
            .collect::<Vec<_>>();
        assert_eq!(
            actual, expected,
            "case{case} root-owned minimap player raster"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original duplicate player minimap rasters match actual-cache native marker preparation");
    Ok(())
}
