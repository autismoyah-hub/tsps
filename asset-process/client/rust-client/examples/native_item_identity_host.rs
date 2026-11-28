use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, item_submission, scene::Scene,
    scene_object_layout::Call, scene_visibility::Visibility, terrain_submission::View,
    world_actors::Actors, zone::Item,
};
use std::collections::{HashMap, HashSet};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let mut actors = Actors::load(&mut client.scripts.source.cache)?;
    let mut scene = Scene::empty_sized(0, 0, 16, 16)?;
    scene.add_object_collision(&HashMap::new())?;
    let item = |id| Item {
        id,
        quantity: 1,
        operations: 31,
        visible: 0,
        despawn: i32::MAX,
        ownership: 0,
        private: false,
        custom: None,
    };
    for plane in 0..4 {
        client.zone.piles.insert(
            [plane, 4, 4],
            vec![item(995), item(995), item(314), item(1436)],
        );
    }
    let support = HashMap::from([([2, 4, 4], 37)]);
    let sources = item_submission::sources(&client.zone, &scene, &support);
    scene.sync_item_piles(&sources.iter().map(|source| source.pile).collect::<Vec<_>>());
    let keys = item_submission::keys(&scene, &sources);
    assert_eq!(keys.len(), 12);
    assert!(item_submission::batches(&keys, std::iter::empty(), 0).is_empty());
    let (mut triangles, pending) = actors.ground_triangles(&mut client, &scene, &support, None)?;
    assert!(pending.is_empty(), "{pending:?}");
    let models = item_submission::batches(&keys, triangles.iter(), 0);
    assert_eq!(
        models.len(),
        12,
        "three actual item models on every logical plane"
    );
    let identities = models.iter().map(|&(id, _, _)| id).collect::<HashSet<_>>();
    assert_eq!(identities.len(), 12);
    let view = View {
        root: false,
        size: [16; 2],
        camera: [4; 2],
        eye_y: -1000,
        radius: 25,
        maximum_plane: 3,
    };
    let mut visibility = Visibility::new([765, 503], [256, 320], 25, [0, 0])?;
    let calls = scene
        .draw_render_calls(&view, &mut visibility, [512; 2], &[])
        .unwrap();
    let ids = |pass: &Vec<Call>| {
        pass.iter()
            .filter_map(|call| {
                if let Call::Item(id) = call {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>()
    };
    assert_eq!(ids(&calls[0]), identities);
    assert_eq!(
        ids(&calls[1]).len(),
        3,
        "only supported piles complete without terrain"
    );
    gielinor_client::model_order::rebase(&mut triangles, 700);
    for triangle in &mut triangles {
        triangle.view = 7;
    }
    let child = item_submission::batches(&keys, triangles.iter(), 7);
    assert_eq!(child.len(), 12);
    assert_eq!(
        child.iter().map(|&(id, _, _)| id).collect::<HashSet<_>>(),
        identities
    );
    assert!(child.iter().all(|&(_, _, batch)| batch >= 700));
    assert!(item_submission::batches(&keys, triangles.iter(), 0).is_empty());
    scene.sync_item_piles(&[]);
    let calls = scene
        .draw_render_calls(&view, &mut visibility, [512; 2], &[])
        .unwrap();
    assert!(calls
        .iter()
        .flatten()
        .all(|call| !matches!(call, Call::Item(_))));
    assert!(scene.temporary_slots().occupied(3, 4, 4));
    println!("12 actual-cache item models across four planes retain source identities, raised completion order, child rebasing and removal occupancy");
    Ok(())
}
