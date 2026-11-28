//! Offline child-world mesh replay using the actual cache's Lumbridge region.
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
    // Optional endpoint uses the same read-only, in-memory recovery as live play.
    let open_cache = || -> Result<DiskCache, Box<dyn std::error::Error>> {
        let mut cache = if let Some(endpoint) = args.get(3) {
            DiskCache::open_remote(&args[1],64 << 20,endpoint.parse()?)?
        } else { DiskCache::open(&args[1],64 << 20)? };
        cache.wait_for_recovery(true);
        Ok(cache)
    };
    let mut cache = open_cache()?;
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
        gielinor_client::client_state::ClientState::load(open_cache()?)?;
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
    use gielinor_client::game_packet::Update;
    let spawn = bits(&[
        (0, 8),
        (1234, 16),
        (0, 1),
        (0, 1),
        (0, 6),
        (0, 3),
        (0, 6),
        (1, 1),
        (npc_id, 14),
    ]);
    client.apply(Update::NpcOrigin([12, 20]))?;
    client.apply(Update::NpcInfo {
        large: false,
        payload: spawn.clone(),
    })?;
    let root_position = client.npc_state.npcs[&1234].path.position;
    client.apply(Update::ActiveWorld { id: 7, plane: 0 })?;
    client.apply(Update::NpcInfo {
        large: false,
        payload: spawn,
    })?;
    assert_eq!(
        client.world_entities[&0].entities[&7].npcs.npcs[&1234].position,
        [6412, 6420]
    );
    for id in [default_animation as u16, 65535] {
        let mut wire = gielinor_client::writer::Writer::new();
        wire.transformed(7, gielinor_client::buffer::Transform::Neg);
        wire.u16_add(1234, false);
        wire.u16(id);
        apply_wire(&mut client, 20, wire.into_vec())?;
        assert_eq!(
            client.world_entities[&0].entities[&7].npcs.npcs[&1234]
                .sequence
                .id,
            if id == 65535 { -1 } else { id as i32 }
        );
        assert_eq!(client.npc_state.npcs[&1234].sequence.id, -1);
    }
    println!("Standalone NPC animation routed to child view and removed without changing root NPC");
    let icon = client
        .npcs
        .values()
        .flat_map(|d| d.head_icons.iter().copied())
        .find(|&(g, i)| g >= 0 && i >= 0)
        .expect("cache NPC icon");
    for (group, image) in [icon, (-1, -1)] {
        let mut wire = gielinor_client::writer::Writer::new();
        wire.u16_add(1234, false);
        wire.transformed(3, gielinor_client::buffer::Transform::Sub);
        wire.u16_le(image as u16);
        wire.u32_middle(group as u32);
        apply_wire(&mut client, 4, wire.into_vec())?;
        assert_eq!(
            client.world_entities[&0].entities[&7].npcs.npcs[&1234]
                .appearance
                .head_icons
                .unwrap()[3],
            (group, image as i32)
        );
        assert!(client.npc_state.npcs[&1234].appearance.head_icons.is_none());
    }
    println!("Standalone child NPC head icon set and removed with independent root state");

    client.apply(Update::NpcInfo {
        large: false,
        payload: bits(&[(1, 8), (1, 1), (1, 2), (7, 3), (0, 1)]),
    })?;
    client.apply(Update::ActiveWorld { id: 0, plane: 0 })?;
    for _ in 0..32 {
        client.tick_actor_sequences()?;
    }
    assert_eq!(client.npc_state.npcs[&1234].path.position, root_position);
    assert_eq!(
        client.world_entities[&0].entities[&7].npcs.npcs[&1234]
            .path
            .tiles[0],
        [6413, 6419]
    );
    let mut actors = gielinor_client::world_actors::Actors::load(&mut cache)?;
    let (npc_triangles, pending) = actors.npc_triangles(&mut client, &scene, Some(7))?;
    assert!(pending.is_empty(), "{pending:?}");
    assert!(!npc_triangles.is_empty(), "child NPC has no mesh");
    println!(
        "Independent child NPC {npc_id}: {} triangles after 32 simulation ticks",
        npc_triangles.len()
    );

    let triangles = mesh
        .triangles
        .iter()
        .chain(npc_triangles.iter())
        .map(|triangle| {
            let mut result = *triangle;
            result.vertices = result.vertices.map(|v| transform.point(v));
            result
        })
        .collect::<Vec<_>>();
    let spot_id = client
        .spots
        .values()
        .filter(|d| d.sequence >= 0)
        .map(|d| d.id)
        .min()
        .unwrap() as u16;
    client.apply(Update::ActiveWorld { id: 7, plane: 0 })?;
    client.apply(Update::ActorSpot(gielinor_client::actor_spot::Update {
        player: false,
        index: 1234,
        slot: 0,
        id: spot_id,
        height: 64,
        delay: 2,
    }))?;
    assert!(client.npc_state.npcs[&1234].spots.slots.is_empty());
    let (delayed, _) = actors.npc_triangles(&mut client, &scene, Some(7))?;
    assert_eq!(delayed.len(), npc_triangles.len());
    client.tick_actor_sequences()?;
    client.tick_actor_sequences()?;
    let (with_spot, pending) = actors.npc_triangles(&mut client, &scene, Some(7))?;
    assert!(pending.is_empty());
    assert!(
        with_spot.len() > npc_triangles.len(),
        "child NPC spot has no triangles"
    );

    let cycle = client.actor_cycle;
    {
        let npc = client
            .world_entities
            .get_mut(&0)
            .unwrap()
            .entities
            .get_mut(&7)
            .unwrap()
            .npcs
            .npcs
            .get_mut(&1234)
            .unwrap();
        npc.visual.apply(
            &gielinor_client::actor_update::Block::Tint {
                delays: [0, 10],
                hsl: [20, 5, 80],
                weight: 100,
            },
            cycle,
        );
        npc.visual.apply(
            &gielinor_client::actor_update::Block::Transparency {
                delays: [0, 0],
                alpha: [0, 128u8 as i8],
                use_start: true,
            },
            cycle,
        );
    }
    let (tinted, _) = actors.npc_triangles(&mut client, &scene, Some(7))?;
    assert_eq!(tinted.len(), with_spot.len());
    assert!(tinted[..npc_triangles.len()]
        .iter()
        .zip(&with_spot)
        .any(|(a, b)| a.colors != b.colors));
    for (a, b) in tinted[npc_triangles.len()..]
        .iter()
        .zip(&with_spot[npc_triangles.len()..])
    {
        assert_eq!(a.colors, b.colors, "spot effect must not receive body tint");
    }
    for (a, b) in tinted.iter().zip(&with_spot) {
        assert_eq!(
            a.alpha,
            gielinor_client::actor_visual::face_alpha(b.alpha, 128)
        );
    }
    assert!(client.npc_state.npcs[&1234].visual.transparency.is_none());
    npc_overlap_replay(&mut client, &scene, &mut actors, &with_spot, &tinted)?;
    client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .get_mut(&7)
        .unwrap()
        .npcs
        .npcs
        .get_mut(&1234)
        .unwrap()
        .visual = Default::default();
    println!("Child NPC body-only tint and body/spot transparency preserve independent root NPC");
    println!(
        "Child NPC delayed spot {spot_id}: {} added triangles",
        with_spot.len() - npc_triangles.len()
    );
    client.apply(Update::ActorSpot(gielinor_client::actor_spot::Update {
        player: false,
        index: 1234,
        slot: 0,
        id: 65535,
        height: 0,
        delay: 0,
    }))?;
    assert!(client.world_entities[&0].entities[&7].npcs.npcs[&1234]
        .spots
        .slots
        .is_empty());
    client.apply(Update::ActiveWorld { id: 0, plane: 0 })?;
    client.apply(Update::ActiveWorld { id: 7, plane: 0 })?;
    let (before_graphics, _) = actors.npc_triangles(&mut client, &scene, Some(7))?;
    apply_wire(&mut client, 9, vec![20 + 128, 12, 0])?;
    let mut graphic = gielinor_client::writer::Writer::new();
    graphic.u8(64);
    graphic.u16_le(spot_id);
    graphic.u16_le(2);
    graphic.u8(0);
    apply_wire(&mut client, 86, graphic.into_vec())?;
    assert!(client.world_spots.is_empty());
    assert_eq!(
        client.world_entities[&0].entities[&7].world_spots[0].position,
        [0, 6412, 6420]
    );
    let (delayed_graphics, _) = actors.npc_triangles(&mut client, &scene, Some(7))?;
    assert_eq!(delayed_graphics.len(), before_graphics.len());
    for _ in 0..2 {
        client.tick_actor_sequences()?;
        client.draw_child_graphics(7, 1, true, false)?;
    }
    let (still_delayed, _) = actors.npc_triangles(&mut client, &scene, Some(7))?;
    assert_eq!(still_delayed.len(), before_graphics.len(), "delay includes the receive cycle");
    client.tick_actor_sequences()?;
    client.draw_child_graphics(7, 1, true, false)?;
    let (graphics, pending) = actors.npc_triangles(&mut client, &scene, Some(7))?;
    assert!(pending.is_empty());
    assert!(graphics.len() > before_graphics.len());
    println!(
        "Child standalone delayed spot {spot_id}: {} added triangles",
        graphics.len() - before_graphics.len()
    );
    for _ in 0..2048 {
        if client.world_entities[&0].entities[&7]
            .world_spots
            .is_empty()
        {
            break;
        }
        client.tick_actor_sequences()?;
        client.draw_child_graphics(7, 1, true, false)?;
    }
    assert!(
        client.world_entities[&0].entities[&7]
            .world_spots
            .is_empty(),
        "child graphics did not expire"
    );
    client.apply(Update::ActiveWorld { id: 0, plane: 0 })?;
    client.apply(Update::ActiveWorld { id: 7, plane: 0 })?;
    client.apply(Update::Zone(gielinor_client::zone::Packet {
        absolute: None,
        origin: Some([0, 12, 20]),
        clear: false,
        records: vec![gielinor_client::zone::Record::Add {
            at: 0,
            item: gielinor_client::zone::Item {
                id: 1277,
                quantity: 1,
                operations: 31,
                visible: 0,
                despawn: 65535,
                ownership: 0,
                private: false,
                custom: None,
            },
        }],
    }))?;
    assert!(client.zone.piles.is_empty());
    let (ground, pending) =
        actors.ground_triangles(&mut client, &scene, &mesh.item_support, Some(7))?;
    assert!(pending.is_empty(), "{pending:?}");
    assert!(!ground.is_empty());
    println!("Independent child ground sword: {} triangles", ground.len());
    client.apply(Update::Zone(gielinor_client::zone::Packet {
        absolute: None,
        origin: None,
        clear: false,
        records: vec![gielinor_client::zone::Record::Remove {
            at: 0,
            id: 1277,
            quantity: 1,
        }],
    }))?;
    let (removed_ground, _) =
        actors.ground_triangles(&mut client, &scene, &mesh.item_support, Some(7))?;
    assert!(removed_ground.is_empty());
    client.apply(Update::ActiveWorld { id: 0, plane: 0 })?;
    client.apply(Update::ActiveWorld { id: 7, plane: 0 })?;
    let target = scene
        .objects
        .iter()
        .find(|o| o.plane == 0 && o.x > 0 && o.x < 22 && o.y > 0 && o.y < 38)
        .unwrap();
    let origin = [target.plane as i32, target.x as i32, target.y as i32];
    let key = [
        origin[0],
        scene.base_x + origin[1],
        scene.base_y + origin[2],
        gielinor_client::location_update::category(target.shape)? as i32,
    ];
    let original = scene.location(key);
    let before_objects = scene.objects.len();
    apply_wire(
        &mut client,
        9,
        vec![
            (origin[2] as u8).wrapping_add(128),
            origin[1] as u8,
            (origin[0] as u8).wrapping_neg(),
        ],
    )?;
    apply_wire(
        &mut client,
        10,
        vec![(original.shape << 2) | original.rotation, 0],
    )?;
    let object_definitions = gielinor_client::object::Definition::load_all(&mut cache)?;
    actors.child_locations(
        &mut client,
        &mut scene,
        &mut mesh,
        7,
        1,
        &object_definitions,
    )?;
    assert_eq!(scene.location(key).id, -1);
    assert_eq!(scene.objects.len(), before_objects - 1);
    apply_wire(
        &mut client,
        33,
        vec![
            128,
            97,
            ((original.shape << 2) | original.rotation).wrapping_add(128),
            0,
            (original.id as u8).wrapping_add(128),
            (original.id >> 8) as u8,
        ],
    )?;
    actors.child_locations(
        &mut client,
        &mut scene,
        &mut mesh,
        7,
        1,
        &object_definitions,
    )?;
    assert_eq!(scene.location(key), original);
    assert_eq!(scene.objects.len(), before_objects);
    apply_wire(
        &mut client,
        30,
        vec![
            ((original.shape << 2) | original.rotation).wrapping_add(128),
            255,
            255,
            0,
            origin[2] as u8,
            origin[1] as u8,
        ],
    )?;
    actors.child_locations(
        &mut client,
        &mut scene,
        &mut mesh,
        7,
        1,
        &object_definitions,
    )?;
    assert_eq!(scene.location(key), original);
    assert!(client.world_entities[&0].entities[&7]
        .zone
        .locations
        .is_empty());
    println!("Specific location animation routes to child scene without changing zone origin");

    assert!(client.zone.locations.is_empty());
    println!(
        "Child dynamic location {} removed and restored",
        original.id
    );
    client.apply(Update::ActiveWorld { id: 0, plane: 0 })?;
    client.npc_state.npcs.get_mut(&1234).unwrap().sequence.id = 42;
    client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .get_mut(&7)
        .unwrap()
        .npcs
        .npcs
        .get_mut(&1234)
        .unwrap()
        .sequence
        .id = 43;
    client.apply(Update::ActiveWorld { id: 7, plane: 0 })?;
    client.apply(Update::ResetAnimations)?;
    assert_eq!(client.npc_state.npcs[&1234].sequence.id, 42);
    assert_eq!(
        client.world_entities[&0].entities[&7].npcs.npcs[&1234]
            .sequence
            .id,
        -1
    );
    client.apply(Update::ActiveWorld { id: 0, plane: 0 })?;
    client.apply(Update::ResetAnimations)?;
    assert_eq!(client.npc_state.npcs[&1234].sequence.id, -1);
    println!("Actor animation reset respects active world");
    let npc_only = npc_triangles
        .iter()
        .map(|triangle| {
            let mut triangle = *triangle;
            triangle.vertices = triangle.vertices.map(|v| transform.point(v));
            triangle
        })
        .collect::<Vec<_>>();
    let fine = client.world_entities[&0].entities[&7].npcs.npcs[&1234]
        .path
        .position;
    let local = [fine[0] - scene.base_x * 128, fine[1] - scene.base_y * 128];
    let height = scene.elevation(0, local[0], local[1]).unwrap();
    let centre = transform.point([local[0] as f32, height as f32 - 100.0, local[1] as f32]);
    let textures = Definition::load_all(&mut cache)?
        .into_iter()
        .map(|(id, definition)| Ok((id, Texture::load(&mut cache, definition, 128, 0.8)?)))
        .collect::<gielinor_client::Result<std::collections::HashMap<_, _>>>()?;
    let mut isolated = Frame::new(320, 320)?;
    let actor_camera = Camera::look_at(
        [centre[0] + 400.0, centre[1] - 250.0, centre[2] - 400.0],
        centre,
        600.0,
    )?;
    isolated.draw(
        &actor_camera,
        &npc_only,
        &gielinor_client::floor::palette(0.8)?,
        &textures,
    );
    let mut overlay_raster =
        Raster::from_pixels(isolated.width, isolated.height, isolated.pixels.clone())?;
    client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .get_mut(&7)
        .unwrap()
        .npcs
        .npcs
        .get_mut(&1234)
        .unwrap()
        .overlays
        .speech
        .say("Child world", false);
    let overlay_transform = gielinor_client::world_entity::SceneTransform(vec![(transform, None)]);
    let child_view = gielinor_client::overlay_render::ChildView {
        id: 7,
        scene: &scene,
        transform: &overlay_transform,
        heights: &actors.heights,
    };
    let mut overlays = gielinor_client::overlay_render::Assets::load(&mut cache)?;
    overlays.draw_at_views(
        &mut client,
        &scene,
        &actor_camera,
        &Default::default(),
        &mut overlay_raster,
        0,
        &[child_view],
    )?;
    let changed = overlay_raster
        .pixels()
        .iter()
        .zip(&isolated.pixels)
        .filter(|(a, b)| a != b)
        .count();
    assert!(changed > 20, "child speech not drawn");
    // Original kh.ap uses the logical actor index in child views too. Player
    // arrows remain steady; NPC arrows flash on the simulation cycle.
    let saved_hint=client.hint.clone();let saved_hint_cycle=client.actor_cycle;
    let mut hint_frame=|kind:u8,index:u16,cycle:i32| -> gielinor_client::Result<Raster> {
        client.hint.kind=kind;client.hint.index=index;client.actor_cycle=cycle;
        let mut frame=Raster::from_pixels(isolated.width,isolated.height,isolated.pixels.clone())?;
        overlays.draw_at_views(&mut client,&scene,&actor_camera,&Default::default(),&mut frame,0,
            &[gielinor_client::overlay_render::ChildView{id:7,scene:&scene,transform:&overlay_transform,heights:&actors.heights}])?;
        Ok(frame)
    };
    let no_hint=hint_frame(0,1234,0)?;
    let npc_hint=hint_frame(1,1234,0)?;
    let hint_pixels=no_hint.pixels().iter().zip(npc_hint.pixels()).filter(|(a,b)|a!=b).count();
    assert!(hint_pixels>10,"child NPC hint arrow missing: {hint_pixels} pixels");
    assert_eq!(no_hint.pixels(),hint_frame(1,1234,10)?.pixels(),"NPC hint off phase");
    assert_eq!(no_hint.pixels(),hint_frame(1,1235,0)?.pixels(),"hint index must match child NPC");
    assert_eq!(npc_hint.pixels(),hint_frame(1,1234,20)?.pixels(),"NPC hint next flash period");
    client.hint=saved_hint;client.actor_cycle=saved_hint_cycle;
    println!("Child NPC hint: {hint_pixels} arrow pixels, off phase/index rejection and exact next-period raster passed");
    isolated.pixels.copy_from_slice(overlay_raster.pixels());
    println!("Child NPC speech overlay: {changed} changed pixels");
    let actor_pixels = isolated.tags.iter().filter(|&&tag| tag != 0).count();
    assert!(actor_pixels > 100, "child NPC not visible");
    std::fs::write(
        std::path::Path::new(&args[2]).with_extension("npc.bmp"),
        Raster::from_pixels(isolated.width, isolated.height, isolated.pixels)?.bmp()?,
    )?;
    println!("Child NPC isolated render: {actor_pixels} visible pixels");
    projectile_replay(&mut client, &scene, &mut actors, spot_id)?;
    let sprite_id = cache
        .group(2, 72, [0; 4])?
        .into_iter()
        .filter_map(|(id, bytes)| {
            gielinor_client::world_entity::Definition::decode(id as i32, &bytes).ok()
        })
        .map(|d| d.sprite)
        .filter(|&id| id >= 0)
        .min()
        .expect("world entity minimap sprite");
    let sprite = std::sync::Arc::new(gielinor_client::sprite_pixels::Pixels::load(
        &mut cache,
        sprite_id as u32,
    )?);
    let mut minimap_raster = Raster::new(180, 180, 180 * 180)?;
    let mut mask = gielinor_client::minimap::Mask {
        width: 150,
        starts: Vec::new(),
        widths: Vec::new(),
    };
    for y in 0i32..150 {
        let half = ((75 * 75 - (y - 75) * (y - 75)).max(0) as f64).sqrt() as i32;
        mask.starts.push(75 - half);
        mask.widths.push(2 * half);
    }
    let view = gielinor_client::minimap::View {
        steering: None,
        terrain: std::sync::Arc::new(gielinor_client::sprite_pixels::Pixels {
            canvas: [1, 1],
            offset: [0, 0],
            size: [1, 1],
            pixels: vec![0xff101820],
        }),
        assets: std::sync::Arc::new(gielinor_client::minimap::Assets::load(&mut cache)?),
        markers: Vec::new(),
        players: Vec::new(),
        icons: Vec::new(),
        entities: vec![gielinor_client::minimap::EntityView {
            delta: [0, 0],
            yaw: 256,
            sprite: Some(sprite),
            markers: vec![([12, 8], 1)],
        }],
        destination: None,
        hint: None,
        compass_disabled: false,
        source: [0, 0],
        angle: 512,
        disabled: false,
        local_visible: true,
    };
    view.draw(&mut minimap_raster, [15, 15], &mask)?;
    let map_pixels = minimap_raster
        .pixels()
        .iter()
        .filter(|&&p| p & 0xffffff != 0)
        .count();
    assert!(map_pixels > 100, "entity minimap empty");
    std::fs::write(
        std::path::Path::new(&args[2]).with_extension("minimap.bmp"),
        minimap_raster.bmp()?,
    )?;
    println!("World entity minimap sprite {sprite_id}: {map_pixels} visible pixels");
    let mut frame = Frame::new(800, 600)?;
    let camera = Camera::look_at([4200.0, -4500.0, -4200.0], [0.0, -100.0, 0.0], 600.0)?;
    frame.draw(
        &camera,
        &triangles,
        &gielinor_client::floor::palette(0.8)?,
        &textures,
    );
    let pixels = frame.tags.iter().filter(|&&tag| tag != 0).count();
    assert!(pixels > 1000, "no visible child terrain");
    std::fs::write(
        &args[2],
        Raster::from_pixels(frame.width, frame.height, frame.pixels)?.bmp()?,
    )?;
    println!(
        "Child world 24x40: {} objects, {} triangles, {pixels} visible pixels",
        mesh.object_instances,
        triangles.len()
    );
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

fn projectile_replay(
    client: &mut gielinor_client::client_state::ClientState,
    scene: &Scene,
    actors: &mut gielinor_client::world_actors::Actors,
    spot: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    use gielinor_client::{
        buffer::Transform,
        game_packet::Update,
        writer::Writer,
        zone::{Packet, Record},
    };
    let saved_base = client.world_base;
    let saved_cycle = client.actor_cycle;
    let saved_pose = client.world_entities[&0].entities[&7].pose;
    client.world_base = [scene.base_x, scene.base_y];
    client.view_registry.insert(7);
    client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .get_mut(&7)
        .unwrap()
        .pose = Pose {
        x: 12 * 128,
        y: 0,
        z: 20 * 128,
        angle: 256,
    };
    client.apply(Update::ActiveWorld { id: 7, plane: 0 })?;
    let mut wire = Writer::new();
    wire.u16(3);
    wire.u8(0);
    wire.u16_add(32, false);
    wire.u16_add(spot, false);
    wire.u16_le(15);
    wire.u32_le(((6416u32) << 14) | 6422);
    wire.u16_add(100, true);
    wire.bytes(&[0; 3]);
    wire.transformed(32, Transform::Neg);
    wire.u16(80);
    wire.u24(0)?;
    client.apply(Update::Zone(Packet {
        absolute: None,
        origin: Some([0, 12, 20]),
        clear: false,
        records: vec![Record::Pending {
            kind: 13,
            bytes: wire.into_vec(),
        }],
    }))?;
    assert!(client.world_entities[&0].entities[&7]
        .zone
        .projectiles
        .is_empty());
    assert_eq!(client.projectiles.len(), 1);
    let q = client.projectiles[0].request.clone();
    assert_eq!(q.source, [0, 6412, 6420]);
    assert_eq!([q.start, q.end], [saved_cycle + 3, saved_cycle + 15]);
    let mut expected = gielinor_client::projectile::Motion::default();
    let child_scenes = std::collections::HashMap::from([(7, scene)]);
    let mut rendered = 0;
    for tick in 0..=16 {
        client.actor_cycle = saved_cycle + tick + 1;
        if tick == 7 {
            client
                .world_entities
                .get_mut(&0)
                .unwrap()
                .entities
                .get_mut(&7)
                .unwrap()
                .pose
                .x += 128;
        }
        let e = &client.world_entities[&0].entities[&7];
        let endpoint = |tile: [i32; 3], height: u16| {
            let p =
                e.transform_horizontal([(tile[1] - 6400) * 128 + 64, (tile[2] - 6400) * 128 + 64]);
            let fine = [p[0] as i32, p[2] as i32];
            [
                fine[0] + scene.base_x * 128,
                fine[1] + scene.base_y * 128,
                scene.elevation(0, fine[0], fine[1]).unwrap_or(0)
                    + scene.elevation(0, e.pose.x, e.pose.z).unwrap_or(0)
                    - height as i32,
            ]
        };
        if tick <= 15 {
            expected.advance(
                endpoint(q.source, q.heights[0]),
                endpoint(q.target, q.heights[1]),
                q.start,
                q.end,
                saved_cycle + tick,
                1,
                q.slope as i32,
                q.offset as i32,
            );
        }
        client.tick_projectiles_in_views(scene, &child_scenes)?;
        if tick < 3 {
            assert!(!client.projectiles[0].motion.moving);
        }
        if (3..=15).contains(&tick) {
            assert_eq!(
                client.projectiles[0].motion.position, expected.position,
                "child world tick {tick}"
            );
            assert_eq!(client.projectiles[0].motion.pitch, expected.pitch);
            if tick == 8 {
                let (with, _) = actors.triangles(client, scene, &Default::default())?;
                let shots = std::mem::take(&mut client.projectiles);
                let (without, _) = actors.triangles(client, scene, &Default::default())?;
                client.projectiles = shots;
                rendered = with
                    .len()
                    .checked_sub(without.len())
                    .expect("projectile geometry");
                assert!(rendered > 0);
            }
        }
    }
    assert!(client.projectiles.is_empty());

    // The same NPC index in root and child must use the first registry match;
    // once root disappears the child copy is transformed with actor arithmetic.
    let saved_npc = client.npc_state.npcs.remove(&1234).unwrap();
    for root_present in [true, false] {
        if root_present {
            let mut npc = saved_npc.clone();
            npc.path.position = [(scene.base_x + 5) * 128 + 64, (scene.base_y + 6) * 128 + 64];
            client.npc_state.npcs.insert(1234, npc);
        } else {
            client.npc_state.npcs.remove(&1234);
        }
        let e = &client.world_entities[&0].entities[&7];
        let fine = if root_present {
            [5 * 128 + 64, 6 * 128 + 64]
        } else {
            let actor = &e.npcs.npcs[&1234];
            let x = actor.path.position[0] - 6400 * 128 - 24 * 64;
            let z = actor.path.position[1] - 6400 * 128 - 40 * 64;
            let angle = -(e.pose.angle as f64) * std::f64::consts::PI / 1024.0;
            [
                (angle.cos() * x as f64 - angle.sin() * z as f64) as i32 + e.pose.x,
                (z as f64 * angle.cos() + angle.sin() * x as f64) as i32 + e.pose.z,
            ]
        };
        let mut shot = q.clone();
        shot.start = 0;
        shot.end = 0;
        shot.entities[1] = 1235;
        client.apply(Update::Projectile(shot))?;
        client.actor_cycle += 1;
        client.tick_projectiles_in_views(scene, &child_scenes)?;
        for axis in 0..2 {
            let target = fine[axis] + client.world_base[axis] * 128;
            assert!((client.projectiles[0].motion.position[axis] - target as f64).abs() < 1e-8);
        }
        client.actor_cycle += 1;
        client.tick_projectiles_in_views(scene, &child_scenes)?;
        assert!(client.projectiles.is_empty());
    }
    client.npc_state.npcs.insert(1234, saved_npc);
    println!("Projectile NPC homing: root duplicate priority and transformed child fallback pass");
    client.world_base = saved_base;
    client.actor_cycle = saved_cycle;
    client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .get_mut(&7)
        .unwrap()
        .pose = saved_pose;
    println!("Child zone projectile: delayed launch, moved-world homing, {rendered} rendered triangles and expiry pass");
    Ok(())
}

fn npc_overlap_replay(
    client: &mut gielinor_client::client_state::ClientState, scene: &Scene,
    actors: &mut gielinor_client::world_actors::Actors,
    untinted: &[gielinor_client::render3d::Triangle], tinted: &[gielinor_client::render3d::Triangle],
) -> Result<(), Box<dyn std::error::Error>> {
    let npc = &client.world_entities[&0].entities[&7].npcs.npcs[&1234];
    let id = npc.id as u32;
    let fine = [npc.path.position[0] - scene.base_x*128, npc.path.position[1] - scene.base_y*128];
    let old_definition = client.npcs[&id].clone();
    client.npcs.get_mut(&id).unwrap().hide_overlap = true;
    client.npcs.get_mut(&id).unwrap().overlap = 39188;
    actors.overlap_bounds = vec![gielinor_client::world_overlap::Bounds::new([0,0,512,512], fine, 0)];
    let heights = actors.heights.clone();
    let (collapsed, pending) = actors.npc_triangles(client, scene, Some(7))?;
    assert!(pending.is_empty()); assert_eq!(collapsed.len(), untinted.len());
    assert_eq!(actors.heights, heights, "overhead heights remain pre-collapse");
    let palette = gielinor_client::floor::palette(0.8)?;
    let mut changed_colors = 0;
    for (a,b) in collapsed.iter().zip(untinted) {
        for (index,(v,w)) in a.vertices.iter().zip(b.vertices).enumerate() {
            assert_eq!(v[0], w[0]); assert_eq!(v[2], w[2]);
            // Collapse local floats, truncate at draw time, then add the displaced origin.
            let draw_base=b.model_bounds.unwrap().position[1] as f32;
            let local_y=b.local_vertices.unwrap()[index][1];
            let expected=draw_base+(local_y*0.01-5.0) as i32 as f32;
            assert!((v[1]-expected).abs()<0.001,"NPC/spot collapsed height");
        }
        let mut expected = *b; expected.scene_tint(39188, &palette);
        assert_eq!(a.colors, expected.colors, "overlap replaces body tint and also tints spots");
        assert_eq!(a.alpha, gielinor_client::actor_visual::face_alpha(b.alpha, 128));
        changed_colors += (a.colors != b.colors) as usize;
    }
    assert!(changed_colors > 0);
    client.npcs.insert(id, old_definition); actors.overlap_bounds.clear();
    let (restored, _) = actors.npc_triangles(client, scene, Some(7))?;
    assert_eq!(restored.len(), tinted.len());
    for (a,b) in restored.iter().zip(tinted) {
        assert_eq!(a.vertices,b.vertices); assert_eq!(a.colors,b.colors); assert_eq!(a.alpha,b.alpha);
    }
    println!("NPC overlap replay: {} body/spot triangles flattened, {changed_colors} tinted, alpha/overhead state preserved, exact restoration", collapsed.len());
    Ok(())
}
