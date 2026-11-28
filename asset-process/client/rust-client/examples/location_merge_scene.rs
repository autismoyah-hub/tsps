use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    location_merge::{Attachment, Request},
    location_update::{Object, Queue},
    scene::Scene,
    world_actors::Actors,
    Result,
};
fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("cache path");
    let mut client = ClientState::load(DiskCache::open(path, 64 << 20)?)?;
    let mut scene = Scene::empty(0, 0)?;
    client.data.position = [0, 50, 50];
    client.actor_cycle = 100;
    let kits = gielinor_client::identity_kit::Kit::load_all(&mut client.scripts.source.cache)?;
    let (mut state, _) = gielinor_client::player_info::State::initialize(&vec![0; 5000], 1)?;
    let player = state.players[1].as_mut().unwrap();
    player.position = [0, 50, 50];
    player.path.teleport([50, 50], 1);
    player.appearance = Some(gielinor_client::player_appearance::Update {
        appearance: gielinor_client::player_model::Appearance::standard(&kits, 0),
        skull: -1,
        overhead: -1,
        animations: [-1; 7],
        name: "fixture".into(),
        combat: 3,
        skill: 0,
        hidden: false,
        force_refresh: false,
        actions: Default::default(),
        team: 0,
    });
    client.player_state = Some(state);
    let definitions =
        gielinor_client::object::Definition::load_all(&mut client.scripts.source.cache)?;
    let mut ids = definitions
        .keys()
        .copied()
        .filter(|id| {
            *id <= 65535
                && !definitions[id].model_ids(10).is_empty()
                && definitions[id].size.iter().all(|v| *v <= 4)
        })
        .collect::<Vec<_>>();
    ids.sort_unstable();
    let mut models = gielinor_client::object_model::Models::default();
    let mut selected = None;
    for id in ids {
        if let Some(model) =
            models.object(&mut client.scripts.source.cache, &definitions[&id], 10, 1)?
        {
            if !model.faces.is_empty() {
                selected = Some((id, model));
                break;
            }
        }
    }
    let (id, model) = selected.expect("object model");
    let request = Request {
        player: 1,
        object: id as u16,
        at: 0,
        shape: 10,
        rotation: 1,
        start: 2,
        end: 5,
        bounds: [-1, -2, 2, 3],
    };
    let key = request.key([0, 50, 50])?;
    scene.replace_location(
        key,
        &Object {
            id: id as i32,
            shape: 10,
            rotation: 1,
        },
        &definitions,
    )?;
    let attachment = Attachment::prepare(
        request,
        [0, 50, 50],
        client.actor_cycle,
        &scene,
        &definitions[&id],
        (*model).clone(),
    )?;
    let mut queue = Queue::default();
    queue.merge(key, 2, 5, |k| scene.location(k));
    let mut actors = Actors::load(&mut client.scripts.source.cache)?;
    let (base, pending) = actors.triangles(&mut client, &scene, &Default::default())?;
    assert!(pending.is_empty());
    client.player_state.as_mut().unwrap().players[1]
        .as_mut()
        .unwrap()
        .attachment = Some(attachment);
    let (waiting, pending) = actors.triangles(&mut client, &scene, &Default::default())?;
    assert!(pending.is_empty());
    assert_eq!(waiting.len(), base.len());
    client.actor_cycle = 102;
    queue.advance([0, 0], 3, |key, object, _| {
        scene.replace_location(key, object, &definitions)?;
        Ok(true)
    })?;
    assert_eq!(scene.location(key).id, -1);
    let (attached, pending) = actors.triangles(&mut client, &scene, &Default::default())?;
    assert!(pending.is_empty());
    assert!(attached.len() > base.len());
    let combined = attached[0].model_bounds.unwrap();
    assert!(attached.iter().all(|t| t.model_bounds == Some(combined)));
    let body = base[0].model_bounds.unwrap();
    assert!(
        combined.radius >= body.radius
            && combined.height >= body.height
            && combined.below >= body.below
    );
    assert!(attached
        .iter()
        .all(|t| t.vertices.iter().flatten().all(|v| v.is_finite())));
    client.preferences.render_self = false;
    let (hidden, pending) = actors.triangles(&mut client, &scene, &Default::default())?;
    assert!(hidden.is_empty());
    assert!(pending.is_empty());
    client.preferences.render_self = true;
    let (restored, pending) = actors.triangles(&mut client, &scene, &Default::default())?;
    assert_eq!(restored.len(), attached.len());
    assert!(pending.is_empty());
    client.actor_cycle = 105;
    client.tick_actor_sequences()?;
    assert!(client.player_state.as_ref().unwrap().players[1]
        .as_ref()
        .unwrap()
        .attachment
        .is_none());
    queue.advance([0, 0], 3, |key, object, _| {
        scene.replace_location(key, object, &definitions)?;
        Ok(true)
    })?;
    assert_eq!(scene.location(key).id, id as i32);
    let (expired, pending) = actors.triangles(&mut client, &scene, &Default::default())?;
    assert!(pending.is_empty());
    assert_eq!(expired.len(), base.len());
    assert!(expired
        .iter()
        .all(|t| t.model_bounds == base[0].model_bounds));
    println!("Timed location merge: object {id}, player triangles {}, attached triangles {}, original scene object restored at end tick",base.len(),attached.len());
    Ok(())
}
