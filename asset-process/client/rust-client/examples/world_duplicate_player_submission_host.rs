use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, game_packet,
    player_appearance, protocol::Packet, scene::Scene, world_actors,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut c = ClientState::load(cache)?;
    let mut actors = world_actors::Actors::load(&mut c.scripts.source.cache)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/world-duplicate-player-submission-planes-jvm.bin"
    ));
    let count = r.u32()?;
    for case in 0..count {
        c.world_entities.clear();
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
        let direct_size = r.u32()? as usize;
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
            .map(|_| r.u32().map(|id| id as u16))
            .collect::<Result<Vec<_>, _>>()?;
        let _nested = (0..r.u32()?)
            .map(|_| r.u32())
            .collect::<Result<Vec<_>, _>>()?;
        c.preferences.render_self = true;
        let scene = Scene::empty_sized(0, 0, direct_size, direct_size)?;
        let actual = world_actors::submission_plan(&mut c, &scene, Some(id), None)?
            .into_iter()
            .filter(|k| !k.npc)
            .map(|k| k.index)
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "case{case} root-owned player submission");
        let triangles = actors.player_triangles(&mut c, &scene, Some(id))?;
        assert_eq!(
            triangles.is_empty(),
            expected.is_empty(),
            "case{case} root-owned body visibility"
        );
        assert!(
            triangles.iter().all(|t| t.tag == 0x200000000000002a),
            "case{case} player body tag"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original duplicate player root-view submissions match actual-cache planner and body visibility");
    Ok(())
}
