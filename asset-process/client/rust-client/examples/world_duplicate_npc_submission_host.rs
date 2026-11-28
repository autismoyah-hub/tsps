use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, game_packet, npc_info::Npc,
    protocol::Packet, scene::Scene, world_actors,
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
    let mut c = ClientState::load(cache)?;
    let mut actors = world_actors::Actors::load(&mut c.scripts.source.cache)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/world-duplicate-npc-submission-jvm.bin"
    ));
    let count = r.u32()?;
    for case in 0..count {
        c.world_entities = Default::default();
        c.detached_world_entities.clear();
        c.active_world = 0;
        c.view_registry = Default::default();
        c.player_state = None;
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
        let absent = r.u32()?;
        let plane_case = r.u32()?;
        for (slot, parent) in [0, 7].into_iter().enumerate() {
            let e = c
                .world_entities
                .get_mut(&parent)
                .unwrap()
                .entities
                .get_mut(&id)
                .unwrap();
            e.plane = if plane_case == 0 { 0 } else { slot as u8 + 1 };
            e.npcs.indices = vec![41];
            if (slot == 0 && absent == 0) || (slot == 1 && absent == 1) {
                e.npcs.npcs.insert(41, npc(320 + 128 * slot as i32));
                e.npcs.insertion_order = vec![41];
            }
        }
        let expected = (0..r.u32()?)
            .map(|_| r.u32().map(|id| id as u16))
            .collect::<Result<Vec<_>, _>>()?;
        let _nested = (0..r.u32()?)
            .map(|_| r.u32())
            .collect::<Result<Vec<_>, _>>()?;
        let scene = Scene::empty_sized(0, 0, direct_size, direct_size)?;
        let actual = world_actors::submission_plan(&mut c, &scene, Some(id), None)?
            .into_iter()
            .filter(|k| k.npc)
            .map(|k| k.index)
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "case{case} root-owned NPC submission");
        let (triangles, pending) = actors.npc_triangles(&mut c, &scene, Some(id))?;
        assert!(pending.is_empty(), "case{case} pending {pending:?}");
        assert_eq!(
            triangles.is_empty(),
            expected.is_empty(),
            "case{case} root-owned NPC body visibility"
        );
        assert!(
            triangles.iter().all(|t| t.tag == 0x4000000000000029),
            "case{case} NPC body tag"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original duplicate NPC root-view submissions match actual-cache planner and body visibility");
    Ok(())
}
