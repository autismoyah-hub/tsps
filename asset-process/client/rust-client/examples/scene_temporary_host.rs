use gielinor_client::{
    model_bounds::Bounds,
    model_order::{Mode, Submission},
    render3d::{Camera, Frame, Triangle},
};
fn actor(z: i32, color: i32, batch: usize) -> Triangle {
    let local = [[-30., -30., 0.], [0., 30., 0.], [30., -30., 0.]];
    Triangle {
        vertices: local.map(|v| [v[0], v[1], z as f32]),
        colors: [color; 3],
        flat: true,
        alpha: 128,
        near_alpha: None,
        uv: [[0.; 2]; 3],
        texture: None,
        #[cfg(feature="enhanced-renderer")] material_hsl:None, texture_mapping: None,
        local_vertices: Some(local),
        model_bounds: Some(Bounds::new(&local, [0, 0, z])),
        model_order: Some(Submission {
            actor_spot: None, source_ordinal: None, temporary_id: None, scene_height_offset: 0,
            mode: Mode::Priority,
            priority: 0,
            has_priorities: false,
            has_alpha: true,
            batch,
        }),
        depth_bias: 0,
        tag: 0x2000000000000000 | batch as u64,
        view: 0,
    }
}
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    use gielinor_client::{
        client_state::ClientState, disk_cache::DiskCache, scene::Scene, world_actors,
    };
    let path = std::env::args().nth(1).expect("cache path");
    let mut client = ClientState::load(DiskCache::open(path, 64 << 20)?)?;
    client.world_base = [3200, 3200];
    let mut data = vec![0; ((30 + 2046 * 18) + 7) / 8];
    let packed = ((3208u32) << 14) | 3212;
    for bit in 0..30 {
        if packed & (1 << (29 - bit)) != 0 {
            data[bit / 8] |= 1 << (7 - bit % 8);
        }
    }
    client.initialize_players(&data, 1)?;
    let state = client.player_state.as_mut().unwrap();
    let mut player = state.root_actor(1).unwrap().clone();
    player.turn.angle = 256;
    player.sequence.stretch = true;
    state.players[1] = Some(player.clone());
    player.sequence.stretch = false;
    state.players[2] = Some(player);
    let scene = Scene::empty(3200, 3200)?;
    let camera = Camera::angles([1088, 0, 1000], 0, 0, 100.)?;
    let palette = [0xff0000, 0x00ff00];
    let textures = Default::default();
    let mut faces = [actor(1536, 0, 1), actor(1536, 1, 2)];
    for face in &mut faces {
        for vertex in &mut face.vertices {
            vertex[0] += 1088.;
        }
        face.model_bounds.as_mut().unwrap().position[0] = 1088;
    }
    let mut expected = Frame::new(100, 100)?;
    expected.draw(&camera, &faces, &palette, &textures);
    faces.reverse();
    let mut unsorted = Frame::new(100, 100)?;
    unsorted.draw(&camera, &faces, &palette, &textures);
    assert_ne!(expected.pixels, unsorted.pixels);
    let mut actual = Frame::new(100, 100)?;
    world_actors::register_root_order(&mut actual, &camera, &client, &scene, &faces);
    actual.draw(&camera, &faces, &palette, &textures);
    assert_eq!(actual.pixels, expected.pixels);
    // Both footprints now occupy the same zone. The local-player bias sorts it last.
    client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .sequence
        .stretch = false;
    let mut same_zone = Frame::new(100, 100)?;
    world_actors::register_root_order(&mut same_zone, &camera, &client, &scene, &faces);
    same_zone.draw(&camera, &faces, &palette, &textures);
    assert_eq!(same_zone.pixels, unsorted.pixels);

    let mut spot = *faces.iter().find(|f| f.tag as u16 == 1).unwrap();
    spot.model_order.as_mut().unwrap().actor_spot = Some(spot.tag);
    spot.model_order.as_mut().unwrap().batch = 12;
    spot.colors = [0; 3];
    let mut body = *faces.iter().find(|f| f.tag as u16 == 1).unwrap();
    body.model_order.as_mut().unwrap().batch = 13;
    body.colors = [1; 3];
    let mut spot_expected = Frame::new(100, 100)?;
    spot_expected.draw(&camera, &[spot, body], &palette, &textures);
    let mut spot_actual = Frame::new(100, 100)?;
    world_actors::register_root_order(&mut spot_actual, &camera, &client, &scene, &[body, spot]);
    spot_actual.draw(&camera, &[body, spot], &palette, &textures);
    assert_eq!(spot_actual.pixels, spot_expected.pixels);
    let state = client.player_state.as_mut().unwrap();
    let mut copies = std::collections::BTreeMap::new();
    for index in [1, 2] {
        copies.insert(index, state.root_actor(index).unwrap().clone());
    }
    copies.get_mut(&1).unwrap().sequence.stretch = true;
    state.local_view = 7;
    state.views.insert(
        7,
        gielinor_client::player_info::View {
            base: [3200, 3200],
            dimensions: [104, 104],
            players: copies,
        },
    );
    let mut world = gielinor_client::world_entity::State::default();
    world.entities.insert(7, Default::default());
    client.world_entities.insert(0, world);
    let mut child_faces = faces;
    for face in &mut child_faces {
        face.view = 7;
    }
    let mut child_expected = Frame::new(100, 100)?;
    child_expected.child_projection(
        7,
        gielinor_client::matrix_pick::identity(),
        [1088, 0, 1000],
        [0, 0],
    );
    child_expected.draw(
        &camera,
        &[child_faces[1], child_faces[0]],
        &palette,
        &textures,
    );
    let mut child = Frame::new(100, 100)?;
    child.child_projection(
        7,
        gielinor_client::matrix_pick::identity(),
        [1088, 0, 1000],
        [0, 0],
    );
    world_actors::register_view_order(&mut child, &camera, &client, &scene, child_faces.iter(), 7);
    child.draw(&camera, &child_faces, &palette, &textures);
    assert_eq!(child.pixels, child_expected.pixels);
    assert!(child.views.contains(&7));
    println!("Root and child temporary registration: stretch zone boundary and local-player identity passed");
    println!("Root temporary registration: cache-backed client, stretch zone boundary, rebased batch identity and local-player bias passed");
    Ok(())
}
