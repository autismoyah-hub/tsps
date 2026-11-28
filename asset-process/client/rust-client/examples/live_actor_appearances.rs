use gielinor_client::{
    actor_update::Customisation,
    client_state::ClientState,
    disk_cache::DiskCache,
    npc_info::{Appearance, Npc},
    scene::Scene,
    world_actors::Actors,
    Result,
};
fn main() -> Result<()> {
    let cache = std::env::args().nth(1).expect("cache path");
    let mut source=if let Some(endpoint)=std::env::args().nth(2) {
        DiskCache::open_remote(cache,64 << 20,endpoint.parse().map_err(|_| gielinor_client::Error::Invalid("JS5 endpoint"))?)?
    } else {DiskCache::open(cache,64 << 20)?};
    source.wait_for_recovery(true);
    let mut client=ClientState::load(source)?;
    client.data.position = [0, 50, 50];
    let scene = Scene::empty(0, 0)?;
    let support = Default::default();
    let mut actors = Actors::load(&mut client.scripts.source.cache)?;
    let mut d = client.npcs[&16299].clone();
    d.id = 60000;
    d.transforms = Some(vec![Some(16299), None]);
    d.transform_varbit = None;
    d.transform_varp = Some(42);
    client.npcs.insert(60000, d);
    client.npc_state.npcs.insert(
        1,
        Npc {
            visual: Default::default(),
            path: {
                let mut p = gielinor_client::actor_path::Path::default();
                p.teleport([50, 50], 1);
                p.position[0]+=1;p.fine[0]+=1.;
                p
            },
            sequence: Default::default(),
            pose: Default::default(),
            idle_fallback: Default::default(),
            spots: Default::default(),
            overlays: Default::default(),
            turn: Default::default(),
            facing: None,
            animations: client.npcs[&60000].animations,
            freezes: Default::default(),
            exact: None,
            id: 60000,
            position: [50, 50],
            angle: 0,
            spawn_cycle: None,
            steps: vec![],
            jump: false,
            blocks: vec![],
            appearance: Appearance::default(),
        },
    );
    client.npc_state.indices.push(1);
    let (normal, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert!(!normal.is_empty());
    assert_eq!(actors.visible_npcs[&1], 16299);
    client.scripts.variables.from_server(42, 1);
    let (hidden, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert!(hidden.is_empty());
    assert!(actors.visible_npcs.is_empty());
    client.scripts.variables.from_server(42, 0);
    let (restored, _) = actors.triangles(&mut client, &scene, &support)?;
    assert_eq!(normal.len(), restored.len());
    for (a, b) in normal.iter().zip(&restored) {
        assert_eq!(a.vertices, b.vertices);
        assert_eq!(a.colors, b.colors);
    }
    client.npc_state.npcs.get_mut(&1).unwrap().pose.state.id = 808;
    client.npc_state.npcs.get_mut(&1).unwrap().pose.state.frame = 4;
    let (posed, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    {
        let npc = client.npc_state.npcs.get_mut(&1).unwrap();
        npc.idle_fallback.state = npc.pose.state.clone();
        npc.pose.state.reset();
    }
    let (fallback, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert_eq!(posed.len(), fallback.len());
    for (a, b) in posed.iter().zip(&fallback) {
        assert_eq!(a.vertices, b.vertices);
        assert_eq!(a.colors, b.colors);
    }
    client
        .npc_state
        .npcs
        .get_mut(&1)
        .unwrap()
        .idle_fallback
        .state
        .loading = 1;
    let (waiting, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert!(waiting.is_empty());
    assert!(actors.visible_npcs.is_empty());
    assert!(!actors.heights.contains_key(&0x4000000000000001));
    client
        .npc_state
        .npcs
        .get_mut(&1)
        .unwrap()
        .idle_fallback
        .state
        .loading = 30;
    let (restored, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert_eq!(posed.len(), restored.len());
    for (a, b) in posed.iter().zip(&restored) {
        assert_eq!(a.vertices, b.vertices);
    }
    client.npc_state.npcs.get_mut(&1).unwrap().idle_fallback = Default::default();
    let alternate = client.npcs[&0].models.clone();
    client.npc_state.npcs.get_mut(&1).unwrap().appearance.body = Some(Customisation {
        models: Some(alternate),
        ..Default::default()
    });
    let (custom, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert!(!custom.is_empty());
    assert!(
        custom.len() != normal.len()
            || custom
                .iter()
                .zip(&normal)
                .any(|(a, b)| a.vertices != b.vertices)
    );
    let (mut state, _) = gielinor_client::player_info::State::initialize(&vec![0; 5000], 1)?;
    let kits = gielinor_client::identity_kit::Kit::load_all(&mut client.scripts.source.cache)?;
    let appearance = gielinor_client::player_model::Appearance::standard(&kits, 0);
    let player = state.players[1].as_mut().unwrap();
    player.position = [0, 50, 50];
    player.path.teleport([50, 50], 1);
    player.appearance = Some(gielinor_client::player_appearance::Update {
        appearance,
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
    // Keep both meshes off the deduplicated tile center for comparison.
    let player=client.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();player.path.position[0]+=1;player.path.fine[0]+=1.;

    client.npc_state.npcs.get_mut(&1).unwrap().appearance.body = Some(Customisation {
        use_player: true,
        ..Default::default()
    });
    let (copy, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    let players = copy
        .iter()
        .filter(|t| t.tag & 0x2000000000000000 != 0)
        .collect::<Vec<_>>();
    let npcs = copy
        .iter()
        .filter(|t| t.tag & 0x4000000000000000 != 0)
        .collect::<Vec<_>>();
    assert!(!players.is_empty());
    assert_eq!(players.len(), npcs.len());
    for (a, b) in players.iter().zip(&npcs) {
        assert_eq!(a.vertices, b.vertices);
        assert_eq!(a.colors, b.colors);
    }
    let appearance = client
        .player_state
        .as_ref()
        .unwrap()
        .root_actor(1)
        .unwrap()
        .appearance
        .as_ref()
        .unwrap()
        .appearance
        .clone();
    client.npc_state.npcs.get_mut(&1).unwrap().appearance.body = Some(Customisation {
        models: Some(appearance.equipment.to_vec()),
        recolor: Some(appearance.colors.map(u16::from).to_vec()),
        composition: Some((
            appearance.gender,
            appearance.base_equipment.map(|n| n as u16).to_vec(),
        )),
        ..Default::default()
    });
    let (composition, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert_eq!(copy.len(), composition.len());
    for (a, b) in copy.iter().zip(&composition) {
        assert_eq!(a.vertices, b.vertices);
        assert_eq!(a.colors, b.colors);
    }
    client.npc_state.npcs.clear();
    client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .appearance
        .as_mut()
        .unwrap()
        .appearance
        .npc = Some(16299);
    let (morphed, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert_eq!(normal.len(), morphed.len());
    for (a, b) in normal.iter().zip(&morphed) {
        assert_eq!(a.vertices, b.vertices);
        assert_eq!(a.colors, b.colors);
    }
    {
        let a = &mut client
            .player_state
            .as_mut()
            .unwrap()
            .root_actor_mut(1)
            .unwrap()
            .appearance
            .as_mut()
            .unwrap()
            .appearance;
        a.npc = None;
        a.equipment[3] = 1277 + 2048;
    }
    let (equipped, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    let model = client.npcs[&0].models[0];
    {
        let a = &mut client
            .player_state
            .as_mut()
            .unwrap()
            .root_actor_mut(1)
            .unwrap()
            .appearance
            .as_mut()
            .unwrap()
            .appearance;
        let key = a.render_key();
        let hash = a.hash();
        a.overrides[3] = Some(gielinor_client::item_model::AppearanceOverride {
            wear_model: [Some(model), None],
            ..Default::default()
        });
        assert_eq!(hash, a.hash());
        assert_ne!(key, a.render_key());
    }
    let (overridden, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert!(
        equipped.len() != overridden.len()
            || equipped
                .iter()
                .zip(&overridden)
                .any(|(a, b)| a.vertices != b.vertices)
    );
    let sequences =
        gielinor_client::animation::Sequence::load_all(&mut client.scripts.source.cache)?;
    let player = client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap();
    player.appearance.as_mut().unwrap().appearance.overrides = Default::default();
    let (baseline, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .sequence
        .set(866, 2, 0, &sequences)?;
    let (delayed, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert!(baseline
        .iter()
        .zip(&delayed)
        .all(|(a, b)| a.vertices == b.vertices));
    client.tick_actor_sequences()?;
    client.tick_actor_sequences()?;
    let (animated, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert!(
        baseline.len() != animated.len()
            || baseline
                .iter()
                .zip(&animated)
                .any(|(a, b)| a.vertices != b.vertices)
    );
    for _ in 0..5 {
        client.tick_actor_sequences()?;
    }
    let (advanced, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert!(animated
        .iter()
        .zip(&advanced)
        .any(|(a, b)| a.vertices != b.vertices));
    client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .freezes
        .add(0, 4, true, client.actor_cycle);
    client.tick_actor_sequences()?;
    assert_eq!(
        client
            .player_state
            .as_ref()
            .unwrap()
            .root_actor(1)
            .unwrap()
            .sequence
            .id,
        -1
    );
    let (frozen, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert!(advanced
        .iter()
        .zip(&frozen)
        .all(|(a, b)| a.vertices == b.vertices));
    for _ in 0..4 {
        client.tick_actor_sequences()?;
    }
    let (thawed, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    let p = client.player_state.as_ref().unwrap().root_actor(1).unwrap();
    assert_eq!(
        p.pose.state.id,
        p.appearance.as_ref().unwrap().animations[0]
    );
    assert!(thawed
        .iter()
        .zip(&frozen)
        .any(|(a, b)| a.vertices != b.vertices));
    println!("Live frozen pose: cancelled action retains rendered snapshot until freeze interval expires");
    let reset = gielinor_client::game_packet::decode(&gielinor_client::protocol::Packet {
        opcode: 92,
        payload: vec![],
    })?
    .unwrap();
    client.apply(reset)?;
    let (restored, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert!(thawed
        .iter()
        .zip(&restored)
        .all(|(a, b)| a.vertices == b.vertices));
    let player = client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap();
    player.appearance.as_mut().unwrap().animations = [808, 823, 819, 820, 821, 822, 824];
    player.path.push([51, 50], 1);
    for _ in 0..8 {
        client.tick_actor_sequences()?;
    }
    let player = client.player_state.as_ref().unwrap().root_actor(1).unwrap();
    assert!(player.path.position[0] > 50 * 128 + 64);
    assert_ne!(
        player.pose.state.id,
        player.appearance.as_ref().unwrap().animations[0]
    );
    assert_ne!(player.turn.angle, 0);
    let (walking, pending) = actors.triangles(&mut client, &scene, &support)?;
    assert!(pending.is_empty());
    assert!(walking
        .iter()
        .zip(&restored)
        .any(|(a, b)| a.vertices != b.vertices));
    println!("Live locomotion: movement pose, fine position, turning and rendered geometry advance together");
    let player = client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap();
    player.path.clear();
    player.pose.state.reset();
    player.appearance.as_mut().unwrap().animations = [-1; 7];
    let (before_spot, _) = actors.triangles(&mut client, &scene, &support)?;
    let effect_id = client
        .spots
        .values()
        .filter(|d| {
            d.sequence >= 0
                && sequences
                    .get(&(d.sequence as u32))
                    .is_some_and(|s| !s.frames.is_empty())
        })
        .map(|d| d.id)
        .min()
        .unwrap() as u16;
    client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .spots
        .apply(0, effect_id, 80, 3, client.actor_cycle);
    let (delayed_spot, _) = actors.triangles(&mut client, &scene, &support)?;
    assert_eq!(before_spot.len(), delayed_spot.len());
    for _ in 0..4 {
        client.tick_actor_sequences()?;
    }
    let (visible_spot, _) = actors.triangles(&mut client, &scene, &support)?;
    assert!(visible_spot.len() > before_spot.len());
    assert!(visible_spot[before_spot.len()..].iter().all(|t|t.model_order.unwrap().actor_spot==Some(0x2000000000000001)));
    let original_flag=client.spots[&(effect_id as u32)].flag;
    client.spots.get_mut(&(effect_id as u32)).unwrap().flag=false;
    let (untagged_spot,_)=actors.triangles(&mut client,&scene,&support)?;
    assert!(untagged_spot[before_spot.len()..].iter().all(|t|t.tag==0&&t.model_order.unwrap().actor_spot==Some(0x2000000000000001)));
    client.spots.get_mut(&(effect_id as u32)).unwrap().flag=original_flag;

    client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .visual
        .apply(
            &gielinor_client::actor_update::Block::Transparency {
                delays: [0, 0],
                alpha: [0, 128u8 as i8],
                use_start: true,
            },
            client.actor_cycle,
        );
    let (faded_spot, _) = actors.triangles(&mut client, &scene, &support)?;
    assert_eq!(faded_spot.len(), visible_spot.len());
    for (i, (a, b)) in faded_spot.iter().zip(&visible_spot).enumerate() {
        let affected = b.tag == 0x2000000000000001 || i >= before_spot.len();
        assert_eq!(
            a.alpha,
            if affected {
                gielinor_client::actor_visual::face_alpha(b.alpha, 128)
            } else {
                b.alpha
            },
            "root body/spot opacity {i}"
        );
        assert_eq!(a.colors, b.colors);
    }
    client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .visual = Default::default();
    println!("Root player transparency covers body and separate spot-effect pass");

    for _ in 0..2000 {
        client.tick_actor_sequences()?;
        if client
            .player_state
            .as_ref()
            .unwrap()
            .root_actor(1)
            .unwrap()
            .spots
            .slots
            .is_empty()
        {
            break;
        }
    }
    assert!(client
        .player_state
        .as_ref()
        .unwrap()
        .root_actor(1)
        .unwrap()
        .spots
        .slots
        .is_empty());
    let (finished_spot, _) = actors.triangles(&mut client, &scene, &support)?;
    assert_eq!(before_spot.len(), finished_spot.len());
    println!("Live actor spot effect {effect_id}: delayed visibility, animated mesh and automatic completion verified");
    client.apply(gielinor_client::game_packet::Update::Zone(
        gielinor_client::zone::Packet {
            absolute: None,
            origin: Some([0, 48, 48]),
            clear: false,
            records: vec![gielinor_client::zone::Record::Pending {
                kind: 5,
                bytes: vec![60, effect_id as u8, (effect_id >> 8) as u8, 3, 0, 0x22],
            }],
        },
    ))?;
    assert_eq!(client.world_spots.len(), 1);
    assert!(client.zone.pending.is_empty());
    let (waiting_world, _) = actors.triangles(&mut client, &scene, &support)?;
    assert_eq!(waiting_world.len(), before_spot.len());
    for _ in 0..4 {
        client.tick_actor_sequences()?;
        client.draw_root_graphics(1)?;
    }
    let (active_world, _) = actors.triangles(&mut client, &scene, &support)?;
    assert!(active_world.len() > before_spot.len());
    for _ in 0..2000 {
        client.tick_actor_sequences()?;
        client.draw_root_graphics(1)?;
        if client.world_spots.is_empty() {
            break;
        }
    }
    assert!(client.world_spots.is_empty());
    println!(
        "Live zone graphics: enclosed packet becomes a delayed animated world mesh and expires"
    );
    let mut wire = vec![
        0,
        3,
        0x22,
        0,
        128,
        (effect_id >> 8) as u8,
        (effect_id as u8).wrapping_add(128),
        12,
        0,
    ];
    wire.extend_from_slice(&((52u32 << 14) | 50).to_le_bytes());
    wire.extend_from_slice(&[128, 0, 0, 0, 0, 224, 0, 60, 0, 0, 0]);
    client.apply(gielinor_client::game_packet::Update::Zone(
        gielinor_client::zone::Packet {
            absolute: None,
            origin: Some([0, 48, 48]),
            clear: false,
            records: vec![gielinor_client::zone::Record::Pending {
                kind: 13,
                bytes: wire,
            }],
        },
    ))?;
    assert_eq!(client.projectiles.len(), 1);
    assert!(client.zone.pending.is_empty());
    client.tick_actor_sequences()?;
    client.tick_projectiles(&scene)?;
    assert!(!client.projectiles[0].motion.moving);
    for _ in 0..4 {
        client.tick_actor_sequences()?;
        client.tick_projectiles(&scene)?;
    }
    assert!(client.projectiles[0].motion.moving);
    assert!(client.projectiles[0].motion.position[0] > 50.0 * 128.0 + 64.0);
    let (flying, _) = actors.triangles(&mut client, &scene, &support)?;
    assert!(flying.len() > before_spot.len());
    for _ in 0..14 {
        client.tick_actor_sequences()?;
        client.tick_projectiles(&scene)?;
    }
    assert!(client.projectiles.is_empty());
    println!("Live projectile: enclosed packet, delayed launch, moving mesh and end-cycle removal verified");
    verify_overlays(&mut client, &mut actors, &scene)?;
    println!("Live player action playback: delayed frame, animated geometry, advancing frames and server RESET_ANIMS restoration verified");
    println!("Live actor mesh integration: {} base faces, {} custom faces; morph hiding/restoration, local-player copying, custom player composition and player-to-NPC appearance verified",normal.len(),custom.len());
    Ok(())
}

fn overlay_bits(values: &[(u32, u8)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut bit = 0;
    for &(v, n) in values {
        for i in (0..n).rev() {
            if bit % 8 == 0 {
                out.push(0);
            }
            out[bit / 8] |= (((v >> i) & 1) as u8) << (7 - bit % 8);
            bit += 1;
        }
    }
    out
}
fn verify_overlays(client: &mut ClientState, actors: &mut Actors, scene: &Scene) -> Result<()> {
    use gielinor_client::{
        buffer::Transform, game_packet::Update, overlay_render::Assets, raster::Raster,
        writer::Writer,
    };
    let appearance = client
        .player_state
        .as_ref()
        .unwrap()
        .root_actor(1)
        .unwrap()
        .appearance
        .clone();
    let mut initial = vec![((50 << 14) | 50, 30)];
    initial.extend(std::iter::repeat_n((0, 18), 2046));
    client.initialize_players(&overlay_bits(&initial), 1)?;
    client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .appearance = appearance;
    // This fixture seeds an already visible actor instead of sending an
    // appearance block. Keep it in the root scene table across membership sync.
    let state = client.player_state.as_mut().unwrap();
    let actor = state.root_actor(1).unwrap().clone();
    state.views.insert(
        0,
        gielinor_client::player_info::View {
            base: client.world_base,
            dimensions: [104, 104],
            players: std::collections::BTreeMap::from([(1, actor)]),
        },
    );
    let hit_id = *client
        .overlay_definitions
        .hits
        .iter()
        .filter(|(_, d)| d.transforms.is_none() && !d.text.is_empty())
        .map(|(id, _)| id)
        .min()
        .unwrap();
    let bar_id = 0;
    let fill = client.overlay_definitions.bars[&bar_id].width;
    let cycle = client.actor_cycle;
    let mut wire = overlay_bits(&[(1, 1), (1, 1), (0, 2)]);
    wire.extend(overlay_bits(&[(0, 1), (3, 2), (2045, 11)]));
    let mut block = Writer::new();
    block.bytes(&[8, 8, 5]);
    block.transformed(1, Transform::Sub);
    for n in [hit_id as u16, 12, 3, 4] {
        block.unsigned_smart(n)?;
    }
    block.transformed(1, Transform::Add);
    for n in [bar_id as u16, 5, 3] {
        block.unsigned_smart(n)?;
    }
    block.transformed(fill, Transform::Add);
    block.transformed(fill / 3, Transform::Neg);
    wire.extend(block.into_vec());
    client.apply(Update::PlayerInfo(wire))?;
    let mut wire = overlay_bits(&[
        (0, 8),
        (2, 16),
        (0, 1),
        (1, 1),
        (2, 6),
        (0, 3),
        (0, 6),
        (1, 1),
        (16299, 14),
        (65535, 16),
    ]);
    let mut block = Writer::new();
    block.bytes(&[64, 8, 40, 1]);
    block.transformed(1, Transform::Add);
    for n in [hit_id as u16, 9, 3, 4] {
        block.unsigned_smart(n)?;
    }
    block.transformed(1, Transform::Neg);
    for n in [bar_id as u16, 5, 3] {
        block.unsigned_smart(n)?;
    }
    block.transformed(fill, Transform::Add);
    block.transformed(fill / 2, Transform::Sub);
    wire.extend(block.into_vec());
    client.apply(Update::NpcInfo {
        large: false,
        payload: wire,
    })?;
    let p = &client
        .player_state
        .as_ref()
        .unwrap()
        .root_actor(1)
        .unwrap()
        .overlays;
    assert_eq!(p.hits.slots.len(), 1);
    assert_eq!(p.bars.entries[0].updates[0].start, cycle + 3);
    assert_eq!(p.bars.entries[0].updates[0].interpolation, 5);
    assert_eq!(client.npc_state.npcs[&2].overlays.hits.slots.len(), 1);
    let end = p.hits.slots[0].end;
    let camera = gielinor_client::render3d::Camera::look_at(
        [50.0 * 128.0 + 1200.0, -1800.0, 50.0 * 128.0 - 1400.0],
        [50.0 * 128.0, 0.0, 50.0 * 128.0],
        512.0,
    )?;
    let mut assets = Assets::load(&mut client.scripts.source.cache)?;
    actors.triangles(client, scene, &Default::default())?;
    let mut raster = Raster::new(512, 334, 1 << 20)?;
    assets.draw(client, scene, &camera, &actors.heights, &mut raster)?;
    assert!(raster.pixels().iter().all(|&p| p == 0));
    for _ in 0..4 {
        client.tick_actor_sequences()?;
    }
    let mut skip = overlay_bits(&[(0, 1), (0, 2)]);
    skip.extend(overlay_bits(&[(0, 1), (3, 2), (2045, 11)]));
    client.apply(Update::PlayerInfo(skip))?;
    client.apply(Update::NpcInfo {
        large: false,
        payload: overlay_bits(&[(1, 8), (0, 1)]),
    })?;
    assert_eq!(
        client
            .player_state
            .as_ref()
            .unwrap()
            .root_actor(1)
            .unwrap()
            .overlays
            .hits
            .slots[0]
            .end,
        end
    );
    assert_eq!(client.npc_state.npcs[&2].overlays.hits.slots.len(), 1);
    actors.triangles(client, scene, &Default::default())?;
    assets.draw(client, scene, &camera, &actors.heights, &mut raster)?;
    assert!(raster.pixels().iter().filter(|&&p| p != 0).count() > 100);
    std::fs::write("work/actor-overlays-integration.bmp", raster.bmp()?)?;
    let expiry = end.max(cycle + 3 + 5 + client.overlay_definitions.bars[&bar_id].duration as i32);
    while client.actor_cycle <= expiry {
        client.tick_actor_sequences()?;
    }
    let mut expired = Raster::new(512, 334, 1 << 20)?;
    actors.triangles(client, scene, &Default::default())?;
    assets.draw(client, scene, &camera, &actors.heights, &mut expired)?;
    assert!(expired.pixels().iter().all(|&p| p == 0));
    assert!(client.npc_state.npcs[&2].overlays.bars.entries.is_empty());
    println!("Live actor overlays: player/NPC wire packets, delayed rendering, persistent state and expiry verified");
    let lengths = client.scripts.source.cache.named_file(10, "huffman", "")?;
    let codec = gielinor_client::huffman::Huffman::new(&lengths)?;
    let message = b"Wave from the Rust client";
    let mut compressed = Writer::new();
    compressed.unsigned_smart(message.len() as u16)?;
    compressed.bytes(&codec.encode(message)?);
    let mut compressed = compressed.into_vec();
    compressed.reverse();
    let mut wire = overlay_bits(&[(1, 1), (1, 1), (0, 2)]);
    wire.extend(overlay_bits(&[(0, 1), (3, 2), (2045, 11)]));
    let mut block = Writer::new();
    block.bytes(&[8, 1]);
    block.u16_add((14 << 8) | 2, false);
    block.transformed(0, Transform::Neg);
    block.transformed(0, Transform::Add);
    block.transformed(compressed.len() as u8, Transform::Neg);
    block.bytes(&compressed);
    block.transformed(0, Transform::Neg);
    block.transformed(1, Transform::Neg);
    wire.extend(block.into_vec());
    client.apply(Update::PlayerInfo(wire))?;
    let mut wire = overlay_bits(&[(1, 8), (1, 1), (0, 2), (65535, 16)]);
    wire.push(2);
    wire.extend_from_slice(b"Hello from the tanner!\0");
    client.apply(Update::NpcInfo {
        large: false,
        payload: wire,
    })?;
    let speech = &client
        .player_state
        .as_ref()
        .unwrap()
        .root_actor(1)
        .unwrap()
        .overlays
        .speech;
    assert_eq!(speech.remaining, 150);
    assert_eq!(speech.effect, 2);
    assert_eq!(speech.pattern.as_ref().unwrap().len(), 2);
    assert_eq!(client.npc_state.npcs[&2].overlays.speech.remaining, 100);
    let mut speech_frame = Raster::new(512, 334, 1 << 20)?;
    assets.draw(client, scene, &camera, &actors.heights, &mut speech_frame)?;
    assert!(speech_frame.pixels().iter().filter(|&&p| p != 0).count() > 100);
    std::fs::write("work/actor-speech-integration.bmp", speech_frame.bmp()?)?;
    for _ in 0..100 {
        client.tick_actor_sequences()?;
    }
    assert!(client.npc_state.npcs[&2].overlays.speech.text.is_none());
    assert_eq!(
        client
            .player_state
            .as_ref()
            .unwrap()
            .root_actor(1)
            .unwrap()
            .overlays
            .speech
            .remaining,
        50
    );
    for _ in 0..50 {
        client.tick_actor_sequences()?;
    }
    let mut expired = Raster::new(512, 334, 1 << 20)?;
    assets.draw(client, scene, &camera, &actors.heights, &mut expired)?;
    assert!(expired.pixels().iter().all(|&p| p == 0));
    println!("Live speech: compressed player chat/pattern and NPC Say packets render and expire on actor ticks");
    let groups = gielinor_client::minimap::sprite_groups(
        &client.scripts.source.cache.file(17, 3, 0, [0; 4])?,
    )?;
    let mut wire = overlay_bits(&[(1, 8), (1, 1), (0, 2), (65535, 16)]);
    wire.extend_from_slice(&[64, 8, 64, 127]);
    let group = groups[4].unwrap();
    if group < 32767 {
        wire.extend_from_slice(&(group as u16).to_be_bytes());
    } else {
        wire.extend_from_slice(&(group | 0x80000000).to_be_bytes());
    }
    wire.push(2);
    client.apply(Update::NpcInfo {
        large: false,
        payload: wire,
    })?;
    assert_eq!(
        client.npc_state.npcs[&2]
            .appearance
            .head_icons
            .as_ref()
            .unwrap()[0],
        (group as i32, 1)
    );
    let a = client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .appearance
        .as_mut()
        .unwrap();
    a.skull = 0;
    a.overhead = 0;
    client.preferences.player_names = 8;
    client.hint.kind = 3;
    client.hint.index = 1;
    let mut labels = Raster::new(512, 334, 1 << 20)?;
    assets.draw(client, scene, &camera, &actors.heights, &mut labels)?;
    assert!(labels.pixels().iter().filter(|&&p| p != 0).count() > 300);
    std::fs::write("work/actor-heads-integration.bmp", labels.bmp()?)?;
    println!("Live head overlays: player name/skull/prayer/hint and NPC icon override render with cached assets");
    use gielinor_client::{
        protocol::Packet,
        vm::{Host, Stacks},
    };
    let mut w = Writer::new();
    w.string("Owner")?;
    w.u64(123456789);
    w.u8(3);
    w.unsigned_smart(2)?;
    w.string("fixture")?;
    w.u16(301);
    w.u8(7);
    w.string("")?;
    client.apply(
        gielinor_client::game_packet::decode(&Packet {
            opcode: 27,
            payload: w.into_vec(),
        })?
        .unwrap(),
    )?;
    assert!(client.friends_chat.as_ref().unwrap().contains("Fixture"));
    let mut stacks = Stacks::default();
    client.extended(3618, false, &mut stacks)?;
    assert_eq!(stacks.pop_int()?, 7);
    let mut w = Writer::new();
    w.string("fixture")?;
    w.u16(302);
    w.u8(128);
    client.apply(
        gielinor_client::game_packet::decode(&Packet {
            opcode: 112,
            payload: w.into_vec(),
        })?
        .unwrap(),
    )?;
    assert!(client.friends_chat.as_ref().unwrap().contains("fixture"));
    let mut w = Writer::new();
    w.string("fixture")?;
    w.u16(301);
    w.u8(128);
    client.apply(
        gielinor_client::game_packet::decode(&Packet {
            opcode: 112,
            payload: w.into_vec(),
        })?
        .unwrap(),
    )?;
    assert!(!client.friends_chat.as_ref().unwrap().contains("fixture"));
    client.apply(
        gielinor_client::game_packet::decode(&Packet {
            opcode: 27,
            payload: vec![],
        })?
        .unwrap(),
    )?;
    client.extended(3612, false, &mut stacks)?;
    assert_eq!(stacks.pop_int()?, 0);
    println!("Live friends chat: snapshot, world-qualified removal, local rank query and channel leave pass through ClientState");
    use gielinor_client::vm::Object;
    stacks.push_object(Object::String("fixture".into()))?;
    client.extended(3605, false, &mut stacks)?;
    assert_eq!(
        client.chat.line(30, 0).unwrap().text,
        "You can't add yourself to your own friend list"
    );
    stacks.push_object(Object::String("New Friend".into()))?;
    client.extended(3605, false, &mut stacks)?;
    let packet = client.outbound.pop_back().unwrap();
    assert_eq!(packet.opcode, 28);
    assert_eq!(packet.payload, b"New Friend\0");
    client.social.apply_friends(
        vec![gielinor_client::social::FriendUpdate {
            rename: false,
            current: "New Friend".into(),
            previous: "Old Friend".into(),
            world: 301,
            rank: 0,
            flags: 0,
        }],
        301,
        0,
    )?;
    stacks.push_object(Object::String("Old Friend".into()))?;
    client.extended(3606, false, &mut stacks)?;
    assert!(client.social.is_friend("New Friend"));
    stacks.push_object(Object::String("New Friend".into()))?;
    client.extended(3606, false, &mut stacks)?;
    assert!(!client.social.is_friend("New Friend"));
    assert_eq!(client.outbound.pop_back().unwrap().opcode, 65);
    println!("Live social commands: self-add feedback, outbound add and current-name-only removal verified");
    let mut channel = gielinor_client::friends_chat::Channel::default();
    channel.members = [("UserC", 301), ("UserB", 0), ("UserA", 301)]
        .into_iter()
        .enumerate()
        .map(|(i, (name, world))| gielinor_client::friends_chat::Member {
            name: name.into(),
            world,
            rank: 0,
            sequence: i as i32,
        })
        .collect();
    client.friends_chat = Some(channel);
    client.extended(3644, false, &mut stacks)?;
    stacks.push_int(1)?;
    client.extended(3649, false, &mut stacks)?;
    stacks.push_int(1)?;
    client.extended(3646, false, &mut stacks)?;
    client.extended(3655, false, &mut stacks)?;
    assert_eq!(
        client
            .friends_chat
            .as_ref()
            .unwrap()
            .members
            .iter()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>(),
        ["UserA", "UserC", "UserB"]
    );
    println!("Live social sorting: comparator reset, online/name chaining and apply opcodes reorder channel members");

    Ok(())
}
