//! Actual-cache coverage of last-ready model fallback and missing-model isolation.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    identity_kit::Kit,
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
    for index in [2, 4, 7, 9] {
        players.players[index] = Some(Player::at(
            [0, 50 + index as i32, 50],
            Some(appearance.clone()),
            1,
            None,
        ));
    }
    players.high = vec![4, 1, 7, 2, 10];
    c.player_state = Some(players);
    c.preferences.render_self = true;
    let target = gielinor_client::overhead::overlay_varps(&mut c.scripts.source.cache)?[1]
        .expect("target varp");
    c.scripts.variables.from_server(target, 7);
    let mut actors = Actors::load(&mut c.scripts.source.cache)?;
    let check = |triangles: Vec<gielinor_client::render3d::Triangle>, expected: &[u16]| {
        let mut ids = Vec::new();
        for t in triangles {
            let id = t.tag as u16;
            if ids.last() != Some(&id) {
                ids.push(id);
            }
        }
        assert_eq!(ids, expected);
    };
    check(
        actors.player_triangles(&mut c, &scene, None)?,
        &[1, 7, 4, 2],
    );
    c.preferences.render_self = false;
    check(actors.player_triangles(&mut c, &scene, None)?, &[7, 4, 2]);
    c.preferences.render_self = true;
    let state = c.player_state.as_mut().unwrap();
    let copies = state
        .players
        .iter()
        .enumerate()
        .filter_map(|(i, p)| p.clone().map(|p| (i as u16, p)))
        .collect();
    state.views.insert(
        7,
        gielinor_client::player_info::View {
            base: [0, 0],
            dimensions: [104, 104],
            players: copies,
        },
    );
    let mut world = gielinor_client::world_entity::State::default();
    world.entities.insert(7, Default::default());
    c.world_entities.insert(0, world);
    check(
        actors.player_triangles(&mut c, &scene, Some(7))?,
        &[1, 7, 4, 2],
    );
    println!("Actual-cache player submissions: root/child local-target-wire order, render-self toggle and unlisted actor omission passed");
    Ok(())
}
