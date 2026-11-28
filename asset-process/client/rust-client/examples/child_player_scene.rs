//! Cache-backed player copies, simulation, packet routing and child scene rendering.
use gielinor_client::{
    disk_cache::DiskCache,
    raster::Raster,
    rebuild::{Chunk, WorldEntity},
    render3d::{Camera, Frame},
    scene::Scene,
    texture::{Definition, Texture},
    world_entity::{Entity, Pose},
    world_mesh::WorldMesh,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let mut cache = DiskCache::open(&args[1], 64 << 20)?;
    let mut chunks = vec![None; 4 * 3 * 5];
    for x in 0..3 {
        for y in 0..5 {
            chunks[x * 5 + y] = Some(Chunk {
                plane: 0,
                zone_x: 400 + x as u16,
                zone_y: 400 + y as u16,
                rotation: 0,
            });
        }
    }
    let rebuild = WorldEntity {
        base: [6400, 6400],
        dimensions: [24, 40],
        chunks,
        regions: vec![12850],
    };
    let mut scene = Scene::load_world_entity(&mut cache, &rebuild)?;
    assert!(scene.missing_regions.is_empty());
    assert_eq!([scene.width, scene.height], [24, 40]);
    let mut mesh = WorldMesh::build(&scene, &mut cache, 0)?;
    assert!(!mesh.triangles.is_empty());
    let mut entity = Entity::default();
    entity.dimensions = [24, 40];
    entity.pose = Pose {
        x: 0,
        y: 0,
        z: 0,
        angle: 256,
    };
    let transform = entity.geometry_transform(0);
    entity.rebuild = Some(rebuild.clone());
    let sequences = gielinor_client::animation::Sequence::load_all(&mut cache)?;
    let default_animation = sequences
        .iter()
        .filter(|(_, s)| s.cached_animation.is_some() && s.sounds.is_empty())
        .map(|(&id, _)| id)
        .min()
        .expect("silent cached entity sequence");
    entity
        .default_animation
        .set(default_animation as i32, 0, 0, &sequences)?;

    let mut client =
        gielinor_client::client_state::ClientState::load(DiskCache::open(&args[1], 64 << 20)?)?;
    let npc_id = client
        .npcs
        .values()
        .filter(|d| d.name == "Man" && !d.models.is_empty() && d.transforms.is_none())
        .map(|d| d.id)
        .min()
        .expect("cache Man NPC");
    let mut worlds = gielinor_client::world_entity::State::default();
    worlds.visible.push(7);
    worlds.entities.insert(7, entity);
    client.world_entities.insert(0, worlds);

    use gielinor_client::{game_packet::Update, writer::Writer};
    client.world_base = [6400, 6400];
    client.data.position = [0, 6412, 6420];
    let mut initial = vec![((6412 << 14) | 6420, 30)];
    initial.extend(std::iter::repeat_n((0, 18), 2046));
    client.initialize_players(&bits(&initial), 1)?;
    let kits = gielinor_client::identity_kit::Kit::load_all(&mut cache)?;
    let appearance = gielinor_client::player_model::Appearance::standard(&kits, 0);
    let mut bytes = Writer::new();
    bytes.u8(0);
    bytes.u8(255);
    bytes.u8(255);
    for equipment in [appearance.equipment, appearance.base_equipment] {
        for id in equipment {
            if id == 0 {
                bytes.u8(0);
            } else {
                bytes.u16(id as u16);
            }
        }
    }
    for color in appearance.colors {
        bytes.u8(color);
    }
    for _ in 0..7 {
        bytes.u16(65535);
    }
    bytes.string("ChildPlayer")?;
    bytes.u8(3);
    bytes.u16(0);
    bytes.u8(0);
    bytes.u16(0);
    for _ in 0..3 {
        bytes.string("")?;
    }
    bytes.u8(255);
    let bytes = bytes.into_vec();
    let mut packet = vec![0xc0, 0x7f, 0xf4, 32, 128u8.wrapping_sub(bytes.len() as u8)];
    packet.extend(bytes.iter().map(|v| v.wrapping_add(128)));
    client.apply(Update::PlayerInfo(packet))?;
    assert_eq!(client.player_state.as_ref().unwrap().local_view, 7);
    let projected =
        client.world_entities[&0].entities[&7].transform_horizontal([12 * 128 + 64, 20 * 128 + 64]);
    assert_eq!(
        client.local_minimap_fine(),
        [
            6400 * 128 + projected[0] as i32,
            6400 * 128 + projected[2] as i32
        ]
    );
    let state = client.player_state.as_ref().unwrap();
    assert_eq!(
        state.views[&7].players[&1]
            .appearance
            .as_ref()
            .unwrap()
            .name,
        "ChildPlayer"
    );
    assert_eq!(
        state.views[&7].players[&1].path.position,
        state.root_actor(1).unwrap().path.position
    );
    for _ in 0..32 {
        client.tick_actor_sequences()?;
    }
    let mut actors = gielinor_client::world_actors::Actors::load(&mut cache)?;
    let before = actors.player_triangles(&mut client, &scene, Some(7))?;
    assert!(!before.is_empty());
    let (saved_pose, saved_fallback) = {
        let p = client
            .player_state
            .as_mut()
            .unwrap()
            .views
            .get_mut(&7)
            .unwrap()
            .players
            .get_mut(&1)
            .unwrap();
        let saved = (p.pose.clone(), p.idle_fallback.clone());
        p.pose.state.id = 808;
        p.pose.state.frame = 4;
        saved
    };
    let posed = actors.player_triangles(&mut client, &scene, Some(7))?;
    {
        let p = client
            .player_state
            .as_mut()
            .unwrap()
            .views
            .get_mut(&7)
            .unwrap()
            .players
            .get_mut(&1)
            .unwrap();
        p.idle_fallback.state = p.pose.state.clone();
        p.pose.state.reset();
    }
    let fallback = actors.player_triangles(&mut client, &scene, Some(7))?;
    assert_eq!(posed.len(), fallback.len());
    for (a, b) in posed.iter().zip(&fallback) {
        assert_eq!(a.vertices, b.vertices);
        assert_eq!(a.colors, b.colors);
    }
    client
        .player_state
        .as_mut()
        .unwrap()
        .views
        .get_mut(&7)
        .unwrap()
        .players
        .get_mut(&1)
        .unwrap()
        .idle_fallback
        .state
        .loading = 1;
    assert!(
        actors
            .player_triangles(&mut client, &scene, Some(7))?
            .is_empty(),
        "child actor waits during fallback loading"
    );
    {
        let p = client
            .player_state
            .as_mut()
            .unwrap()
            .views
            .get_mut(&7)
            .unwrap()
            .players
            .get_mut(&1)
            .unwrap();
        p.pose = saved_pose;
        p.idle_fallback = saved_fallback;
    }
    let (saved_local, saved_high, saved_pose, saved_idle, saved_fallback) = {
        let state = client.player_state.as_mut().unwrap();
        let local = state.local;
        let high = state.high.clone();
        state.local = 0;
        let p = state
            .views
            .get_mut(&7)
            .unwrap()
            .players
            .get_mut(&1)
            .unwrap();
        let saved = (
            local,
            high,
            p.pose.clone(),
            p.appearance.as_ref().unwrap().animations[0],
            p.idle_fallback.clone(),
        );
        p.pose.state.id = 808;
        p.pose.state.frame = 4;
        p.appearance.as_mut().unwrap().animations[0] = 808;
        p.idle_fallback.state.id = 808;
        p.idle_fallback.state.frame = 0;
        saved
    };
    let full = actors.player_triangles(&mut client, &scene, Some(7))?;
    // The crowd threshold counts logical players across views, not this child scene alone.
    client.player_state.as_mut().unwrap().high = (0..201).collect();
    let crowded = actors.player_triangles(&mut client, &scene, Some(7))?;
    assert_eq!(full.len(), crowded.len());
    assert!(full
        .iter()
        .zip(&crowded)
        .any(|(a, b)| a.vertices != b.vertices));
    client.player_state.as_mut().unwrap().local = 1;
    let priority = actors.player_triangles(&mut client, &scene, Some(7))?;
    for (a, b) in full.iter().zip(&priority) {
        assert_eq!(a.vertices, b.vertices);
    }
    client.player_state.as_mut().unwrap().local = 0;
    {
        let p = client
            .player_state
            .as_mut()
            .unwrap()
            .views
            .get_mut(&7)
            .unwrap()
            .players
            .get_mut(&1)
            .unwrap();
        p.attachment = Some(gielinor_client::location_merge::Attachment {
            placement: gielinor_client::location_merge::Placement {
                start: client.actor_cycle,
                end: client.actor_cycle + 1,
                centre: [p.path.position[0], 0, p.path.position[1]],
                // This synthetic attachment occupies the player's actual world tile.
                bounds: [p.path.position[0] >> 7, p.path.position[1] >> 7,
                    p.path.position[0] >> 7, p.path.position[1] >> 7],
            },
            model: gielinor_client::model::Model::combine(&[])?,
            colors: Vec::new(),
        });
    }
    let attached = actors.player_triangles(&mut client, &scene, Some(7))?;
    assert_eq!(full.len(), attached.len());
    for (a, b) in full.iter().zip(&attached) {
        assert_eq!(a.vertices, b.vertices);
    }
    {
        let state = client.player_state.as_mut().unwrap();
        state.local = saved_local;
        state.high = saved_high;
        let p = state
            .views
            .get_mut(&7)
            .unwrap()
            .players
            .get_mut(&1)
            .unwrap();
        p.pose = saved_pose;
        p.idle_fallback = saved_fallback;
        p.appearance.as_mut().unwrap().animations[0] = saved_idle;
        p.attachment = None;
    }
    // Root/child share wire movement, not the simulation objects.
    let mut walk = bits(&[(1, 1), (0, 1), (1, 2), (4, 3)]);
    walk.extend([0x7f, 0xf4]);
    client.apply(Update::PlayerInfo(walk))?;
    for _ in 0..64 {
        client.tick_actor_sequences()?;
    }
    let state = client.player_state.as_ref().unwrap();
    assert_eq!(
        state.views[&7].players[&1].path.position,
        [6413 * 128 + 64, 6420 * 128 + 64]
    );
    assert_eq!(
        state.root_actor(1).unwrap().path.position,
        state.views[&7].players[&1].path.position
    );
    // Active-view spot and animation reset must not touch the root copy.
    client.apply(Update::ActiveWorld { id: 7, plane: 0 })?;
    let spot = client
        .spots
        .values()
        .filter(|d| d.sequence >= 0)
        .map(|d| d.id)
        .min()
        .unwrap() as u16;
    client.apply(Update::ActorSpot(gielinor_client::actor_spot::Update {
        player: true,
        index: 1,
        slot: 0,
        id: spot,
        height: 0,
        delay: 2,
    }))?;
    assert!(client
        .player_state
        .as_ref()
        .unwrap()
        .root_actor(1)
        .unwrap()
        .spots
        .slots
        .is_empty());
    let delayed = actors.player_triangles(&mut client, &scene, Some(7))?;
    assert_eq!(delayed.len(), before.len());
    client.tick_actor_sequences()?;
    client.tick_actor_sequences()?;
    let with_spot = actors.player_triangles(&mut client, &scene, Some(7))?;
    assert!(with_spot.len() > before.len());
    client.apply(Update::ActorSpot(gielinor_client::actor_spot::Update {
        player: true,
        index: 1,
        slot: 0,
        id: 65535,
        height: 0,
        delay: 0,
    }))?;
    let state = client.player_state.as_mut().unwrap();
    state.root_actor_mut(1).unwrap().sequence.id = 42;
    state
        .views
        .get_mut(&7)
        .unwrap()
        .players
        .get_mut(&1)
        .unwrap()
        .sequence
        .id = 43;
    client.apply(Update::ResetAnimations)?;
    assert_eq!(
        client.player_state.as_ref().unwrap().views[&7].players[&1]
            .sequence
            .id,
        -1
    );
    assert_eq!(
        client
            .player_state
            .as_ref()
            .unwrap()
            .root_actor(1)
            .unwrap()
            .sequence
            .id,
        42
    );
    client.apply(Update::ActiveWorld { id: 0, plane: 0 })?;
    client.apply(Update::ResetAnimations)?;
    let triangles = actors.player_triangles(&mut client, &scene, Some(7))?;
    let local = [13 * 128 + 64, 20 * 128 + 64];
    let height = scene.elevation(0, local[0], local[1]).unwrap();
    let centre = transform.point([local[0] as f32, height as f32 - 100., local[1] as f32]);
    let eye = [centre[0] + 400., centre[1] - 250., centre[2] - 400.].map(|v| v as i32);
    let angles = [1085, 2048];
    let camera = Camera::angles(eye, angles[0], angles[1], 600.)?;
    let textures = Definition::load_all(&mut cache)?
        .into_iter()
        .map(|(id, d)| Ok((id, Texture::load(&mut cache, d, 128, 0.8)?)))
        .collect::<gielinor_client::Result<std::collections::HashMap<_, _>>>()?;
    let transformed = triangles
        .iter()
        .map(|t| {
            let mut t = *t;
            t.vertices = t.vertices.map(|v| transform.point(v));
            t.view = 7;
            t
        })
        .collect::<Vec<_>>();
    let mut frame = Frame::new(320, 320)?;
    frame.child_projection(7, transform.matrix(None), eye, angles);
    frame.draw(
        &camera,
        &transformed,
        &gielinor_client::floor::palette(0.8)?,
        &textures,
    );
    let pixels = frame.tags.iter().filter(|&&v| v != 0).count();
    assert!(pixels > 100);
    let before = frame.pixels.clone();
    let mut raster = Raster::from_pixels(frame.width, frame.height, frame.pixels)?;
    client
        .player_state
        .as_mut()
        .unwrap()
        .views
        .get_mut(&7)
        .unwrap()
        .players
        .get_mut(&1)
        .unwrap()
        .overlays
        .speech
        .say("Independent child player", true);
    let composed = gielinor_client::world_entity::SceneTransform(vec![(transform, None)]);
    let view = gielinor_client::overlay_render::ChildView {
        id: 7,
        scene: &scene,
        transform: &composed,
        heights: &actors.heights,
    };
    let mut overlays = gielinor_client::overlay_render::Assets::load(&mut cache)?;
    overlays.draw_at_views(
        &mut client,
        &scene,
        &camera,
        &Default::default(),
        &mut raster,
        0,
        &[view],
    )?;
    assert_ne!(raster.pixels(), before);
    std::fs::write("work/child-player-cache.bmp", raster.bmp()?)?;
    // Timed player/object merge is routed from real packet 34 into this view.
    client.apply(Update::ActiveWorld { id: 7, plane: 0 })?;
    let object = scene
        .objects
        .iter()
        .find(|o| o.plane == 0 && o.x > 0 && o.x < 22 && o.y > 0 && o.y < 38)
        .unwrap();
    apply_wire(
        &mut client,
        9,
        vec![(object.y as u8).wrapping_add(128), object.x as u8, 0],
    )?;
    let mut merge = Writer::new();
    merge.u8(255);
    merge.u16_le(1);
    merge.u8((object.shape << 2) | object.rotation);
    merge.u8(255);
    merge.u16_le(object.id as u16);
    merge.u16(4);
    merge.u16_add(1, true);
    merge.transformed(1, gielinor_client::buffer::Transform::Add);
    merge.transformed(0, gielinor_client::buffer::Transform::Sub);
    merge.transformed(1, gielinor_client::buffer::Transform::Sub);
    apply_wire(&mut client, 34, merge.into_vec())?;
    let definitions = gielinor_client::object::Definition::load_all(&mut cache)?;
    actors.child_locations(&mut client, &mut scene, &mut mesh, 7, 0, &definitions)?;
    assert!(client
        .player_state
        .as_ref()
        .unwrap()
        .root_actor(1)
        .unwrap()
        .attachment
        .is_none());
    assert!(client.player_state.as_ref().unwrap().views[&7].players[&1]
        .attachment
        .is_some());
    let waiting = actors.player_triangles(&mut client, &scene, Some(7))?;
    assert_eq!(waiting.len(), triangles.len());
    client.tick_actor_sequences()?;
    let merged = actors.player_triangles(&mut client, &scene, Some(7))?;
    assert!(merged.len() > triangles.len());
    // Body tint excludes the attached object; transparency covers both.
    let cycle = client.actor_cycle;
    {
        let p = client
            .player_state
            .as_mut()
            .unwrap()
            .views
            .get_mut(&7)
            .unwrap()
            .players
            .get_mut(&1)
            .unwrap();
        p.visual.apply(
            &gielinor_client::actor_update::Block::Tint {
                delays: [0, 10],
                hsl: [20, 5, 80],
                weight: 100,
            },
            cycle,
        );
        p.visual.apply(
            &gielinor_client::actor_update::Block::Transparency {
                delays: [0, 0],
                alpha: [0, 128u8 as i8],
                use_start: true,
            },
            cycle,
        );
    }
    let tinted = actors.player_triangles(&mut client, &scene, Some(7))?;
    assert_eq!(tinted.len(), merged.len());
    assert!(tinted[..triangles.len()]
        .iter()
        .zip(&merged)
        .any(|(a, b)| a.colors != b.colors));
    for (a, b) in tinted[triangles.len()..]
        .iter()
        .zip(&merged[triangles.len()..])
    {
        assert_eq!(a.colors, b.colors, "attachment must not receive body tint");
    }
    for (a, b) in tinted.iter().zip(&merged) {
        assert_eq!(
            a.alpha,
            gielinor_client::actor_visual::face_alpha(b.alpha, 128)
        );
        assert_eq!(a.vertices, b.vertices);
    }
    assert!(client
        .player_state
        .as_ref()
        .unwrap()
        .root_actor(1)
        .unwrap()
        .visual
        .transparency
        .is_none());
    let transformed = tinted
        .iter()
        .map(|t| {
            let mut t = *t;
            t.vertices = t.vertices.map(|v| transform.point(v));
            t.view = 7;
            t
        })
        .collect::<Vec<_>>();
    let mut tinted_frame = Frame::new(320, 320)?;
    tinted_frame.child_projection(7, transform.matrix(None), eye, angles);
    tinted_frame.draw(
        &camera,
        &transformed,
        &gielinor_client::floor::palette(0.8)?,
        &textures,
    );
    std::fs::write(
        "work/child-player-tinted.bmp",
        Raster::from_pixels(320, 320, tinted_frame.pixels)?.bmp()?,
    )?;
    {
        let p = client
            .player_state
            .as_mut()
            .unwrap()
            .views
            .get_mut(&7)
            .unwrap()
            .players
            .get_mut(&1)
            .unwrap();
        p.visual.apply(
            &gielinor_client::actor_update::Block::Transparency {
                delays: [0, 0],
                alpha: [0, -1],
                use_start: true,
            },
            cycle,
        );
    }
    let invisible = actors.player_triangles(&mut client, &scene, Some(7))?;
    assert!(!invisible.is_empty(), "invisible actor retains model selection");
    assert!(invisible.iter().all(|t| t.alpha == 255), "255 override suppresses body and attachment pixels");
    client
        .player_state
        .as_mut()
        .unwrap()
        .views
        .get_mut(&7)
        .unwrap()
        .players
        .get_mut(&1)
        .unwrap()
        .visual = Default::default();
    println!("Child visual effects: body-only tint, body/attachment transparency, 255 suppression and root isolation");
    for _ in 0..4 {
        client.tick_actor_sequences()?;
    }
    assert!(client.player_state.as_ref().unwrap().views[&7].players[&1]
        .attachment
        .is_none());
    println!(
        "Packet 34 child attachment: delayed start, {} added triangles, expiration, root isolation",
        merged.len() - triangles.len()
    );
    println!("Child player: {} body triangles, {pixels} pixels, independent spot/reset, movement and transformed speech passed; {} scene triangles; NPC reference {npc_id}",triangles.len(),mesh.triangles.len());
    Ok(())
}

fn bits(values: &[(u32, u8)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut bit = 0;
    for &(value, count) in values {
        for shift in (0..count).rev() {
            if bit % 8 == 0 {
                out.push(0);
            }
            out[bit / 8] |= (((value >> shift) & 1) as u8) << (7 - bit % 8);
            bit += 1;
        }
    }
    out
}

fn apply_wire(
    client: &mut gielinor_client::client_state::ClientState,
    opcode: u16,
    payload: Vec<u8>,
) -> gielinor_client::Result<()> {
    let update = gielinor_client::game_packet::decode(&gielinor_client::protocol::Packet {
        opcode,
        payload,
    })?
    .ok_or(gielinor_client::Error::Invalid(
        "Replay packet missing handler",
    ))?;
    client.apply(update)
}
