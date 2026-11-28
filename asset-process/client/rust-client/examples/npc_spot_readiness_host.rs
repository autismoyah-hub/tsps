use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, npc_info::Npc, scene::Scene, world_actors,
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
    let mut actors = world_actors::Actors::load(&mut client.scripts.source.cache)?;
    let scene = Scene::empty(0, 0)?;
    client.player_state = None;
    client.data.position = [0, 0, 0];
    client.actor_cycle = 10;
    let mut actor = npc(320);
    actor.idle_fallback.state.id = 0;
    actor.idle_fallback.state.loading = 1;
    actor.spots.apply(2, 3, 0, 0, client.actor_cycle);
    actor.spots.apply(7, 3, 0, 5, client.actor_cycle);
    client.npc_state.indices = vec![41];
    client.npc_state.insertion_order = vec![41];
    client.npc_state.npcs.clear();
    client.npc_state.npcs.insert(41, actor);
    let (records, tags) = actors.temporary_draw_objects(&mut client, &scene, None)?;
    assert_eq!(records.len(), 1);
    assert_eq!(tags, vec![Some(0x4000000000000029)]);
    let (spots, keys) = actors.attached_spot_objects(&client, None, &records, &tags);
    assert_eq!(spots, vec![(0, [320, 320])]);
    assert_eq!(
        keys,
        vec![(0x4000000000000029, 2)],
        "delayed slot must not enter scene traversal"
    );
    let (triangles, pending) = actors.npc_triangles(&mut client, &scene, None)?;
    assert!(pending.is_empty(), "pending resources {pending:?}");
    assert!(
        !triangles.is_empty(),
        "admitted NPC with unavailable body pose lost its independent attached spot"
    );
    assert!(
        triangles.iter().all(|face| face
            .model_order
            .is_some_and(|order| order.actor_spot == Some(0x4000000000000029)
                && order.source_ordinal == Some(2))),
        "spot owner/slot identity missing or body unexpectedly drawn"
    );
    println!("Unavailable NPC body pose preserves {} cached attached-spot triangles with owner and slot identity",triangles.len());
    Ok(())
}
