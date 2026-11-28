use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    identity_kit::Kit,
    item::Definition,
    player_appearance,
    player_info::{Player, State},
    player_model::Appearance,
    scene::Scene,
    world_actors::Actors,
    Result,
};
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    c.data.position = [0, 50, 50];
    let scene = Scene::empty(0, 0)?;
    let kits = Kit::load_all(&mut c.scripts.source.cache)?;
    let base = Appearance::standard(&kits, 0);
    let appearance = player_appearance::Update {
        appearance: base.clone(),
        skull: -1,
        overhead: -1,
        animations: [-1; 7],
        name: "Fallback".into(),
        combat: 3,
        skill: 0,
        hidden: false,
        force_refresh: false,
        actions: Default::default(),
        team: 0,
    };
    let (mut players, _) = State::initialize(&[0; 4608], 1)?;
    players.players[1] = Some(Player::at([0, 50, 50], Some(appearance.clone()), 1, None));
    c.player_state = Some(players);
    let mut actors = Actors::load(&mut c.scripts.source.cache)?;
    let original = actors.player_triangles(&mut c, &scene, None)?;
    assert!(!original.is_empty());
    let key = base.render_key();
    assert_eq!(
        c.player_state
            .as_ref()
            .unwrap()
            .root_actor(1)
            .unwrap()
            .last_model
            .get(),
        key
    );
    let mut item = Definition::decode(60000, &[0])?;
    item.wear_models = [[i32::MAX, -1, -1]; 2];
    c.items.items.insert(60000, item);
    let missing = {
        let p = c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();
        p.appearance.as_mut().unwrap().appearance.equipment[3] = 62048;
        p.appearance.clone()
    };
    let fallback = actors.player_triangles(&mut c, &scene, None)?;
    assert_eq!(original.len(), fallback.len());
    for (a, b) in original.iter().zip(&fallback) {
        assert_eq!(a.vertices, b.vertices);
        assert_eq!(a.colors, b.colors);
    }
    // A second, fresh actor with the unavailable appearance cannot borrow another
    // actor's last-built pointer, even though both share the model cache.
    c.player_state.as_mut().unwrap().players[2] = Some(Player::at([0, 51, 50], missing, 1, None));
    c.player_state.as_mut().unwrap().high.push(2);
    let independent = actors.player_triangles(&mut c, &scene, None)?;
    assert_eq!(independent.len(), original.len());
    assert!(!actors.heights.contains_key(&0x2000000000000002));
    c.player_state
        .as_mut()
        .unwrap()
        .model_invalidations
        .insert(key);
    assert!(actors.player_triangles(&mut c, &scene, None)?.is_empty());
    assert!(!actors.heights.contains_key(&0x2000000000000001));
    c.player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .appearance = Some(appearance);
    let restored = actors.player_triangles(&mut c, &scene, None)?;
    assert_eq!(restored.len(), original.len());
    let models = c
        .scripts
        .source
        .cache
        .reference(7)?
        .groups
        .iter()
        .map(|g| g.id)
        .collect::<std::collections::HashSet<_>>();
    let effect_id = c
        .spots
        .iter()
        .filter(|(_, d)| models.contains(&d.model))
        .map(|(&id, _)| id)
        .min()
        .unwrap() as u16;
    c.spots.get_mut(&(effect_id as u32)).unwrap().sequence = -1;
    let mut child = c
        .player_state
        .as_ref()
        .unwrap()
        .root_actor(2)
        .unwrap()
        .clone();
    child.spots.apply(0, effect_id, 0, 0, c.actor_cycle);
    c.player_state.as_mut().unwrap().views.insert(
        7,
        gielinor_client::player_info::View {
            base: [0, 0],
            dimensions: [104, 104],
            players: std::collections::BTreeMap::from([(2, child)]),
        },
    );
    let mut world = gielinor_client::world_entity::State::default();
    world.entities.insert(7, Default::default());
    c.world_entities.insert(0, world);
    let effects = actors.player_triangles(&mut c, &scene, Some(7))?;
    assert!(!effects.is_empty());
    assert!(effects
        .iter()
        .all(|f| f.model_order.unwrap().actor_spot == Some(0x2000000000000002)));
    assert!(!actors.heights.contains_key(&0x2000000000000002));
    println!("Child actor with missing body retains independently available attached effect");
    println!("Player model fallback: {} faces retained; unavailable fresh actor skipped; eviction hides and restored assets rebuild",original.len());
    Ok(())
}
