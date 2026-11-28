use gielinor_client::{
    animation::Sequence,
    animation_runtime::Clock,
    audio_runtime::Audio,
    audio_synth::Effect,
    client_state::ClientState,
    disk_cache::DiskCache,
    sequence_sound,
    world_audio::Context,
    world_entity::{Entity, Pose, State},
    world_spot, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let sequences = Sequence::load_all(&mut cache)?;
    let mut client = ClientState::load(cache)?;
    let mut candidates = client
        .spots
        .values()
        .filter_map(|spot| {
            let sequence = sequences.get(&(spot.sequence as u32))?;
            let mut frames = sequence.sounds.keys().copied().collect::<Vec<_>>();
            frames.sort();
            frames.into_iter().find_map(|frame| {
                let sounds = &sequence.sounds[&frame];
                (frame > 0
                    && sounds.len() == 1
                    && sounds[0].range > 0
                    && sounds[0].loops > 0
                    && (sequence.cached_animation.is_some()
                        || (frame as usize) < sequence.frames.len()))
                .then(|| (spot.id, sequence.id, frame, sounds[0].clone()))
            })
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|v| (v.0, v.2));
    let mut chosen = None;
    for c in candidates {
        let source = &mut client.scripts.source.cache;
        if !source.reference(4)?.groups.iter().any(|g| {
            g.id == c.3.id as u32
                && g.files.iter().any(|f| f.id == 0)
                && !g.files.iter().any(|f| f.id == 1)
        }) {
            continue;
        }
        let mut effect = Effect::decode(&source.file(4, c.3.id as u32, 0, [0; 4])?)?;
        effect.trim_delay();
        if effect.synthesize(false)?.iter().any(|&v| v != 0) {
            chosen = Some(c);
            break;
        }
    }
    let (spot, sequence, frame, sound) =
        chosen.expect("actual cache graphic with non-silent single frame sound");
    client.world_base = [3200, 6400];
    client.preferences.device.set_game(7, 0)?;
    client.preferences.device.set_game(8, 100)?;
    client.preferences.device.set_game(9, 100)?;
    let mut entity = Entity::default();
    entity.dimensions = [24, 40];
    entity.pose = Pose {
        x: 4096,
        y: 0,
        z: 4096,
        angle: 256,
    };
    entity.rebuild = Some(gielinor_client::rebuild::WorldEntity {
        base: [100, 200],
        dimensions: [24, 40],
        chunks: vec![],
        regions: vec![],
    });
    let mut effect = world_spot::Request {
        id: spot as u16,
        height: 0,
        delay: 0,
        position: [0, 112, 220],
    }
    .start(0);
    effect.animation.clock = Clock {
        frame: frame as usize - 1,
        elapsed: sequences[&sequence]
            .lengths
            .get(frame as usize - 1)
            .copied()
            .unwrap_or(0) as u32,
        ..Default::default()
    };
    entity.world_spots.push(effect);
    let mut root = State::default();
    root.visible.push(7);
    root.entities.insert(7, entity);
    client.world_entities.insert(0, root);
    client.view_registry.insert(7);
    client.tick_actor_sequences()?;
    assert_eq!(client.audio.pending_count(),0,"child graphics wait for the draw phase");
    client.draw_child_graphics(7,1,false,false)?;
    client.draw_child_graphics(7,1,true,true)?;
    assert_eq!(client.audio.pending_count(),0,"hidden or denied graphics cannot emit sounds");
    client.draw_child_graphics(7,1,true,false)?;
    assert_eq!(
        client.audio.pending_count(),
        1,
        "actual child graphic emits one queued sound"
    );
    client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .get_mut(&7)
        .unwrap()
        .pose
        .x += 2048;
    let request =
        sequence_sound::request(&sound, false, [12 * 128 + 64, 20 * 128 + 64], [0; 2]).unwrap();
    let point = client.world_entities[&0].entities[&7].transform_horizontal([12 * 128, 20 * 128]);
    let tile = [
        client.world_base[0] + ((point[0] as i32) >> 7),
        client.world_base[1] + ((point[2] as i32) >> 7),
    ];
    let listener = [tile[0] * 128 + 64, tile[1] * 128 + 64];
    let options = &client.preferences.device;
    let mut direct = Audio::default();
    let mut placed = request.clone();
    placed.tile = tile;
    direct.request_sequence(placed, options)?;
    let mut denied = Audio::default();
    denied.request_world_sequence(7, false, request.clone(), options)?;
    let mut cross = Audio::default();
    cross.request_world_sequence(7, true, request.clone(), options)?;
    let mut root_denied = Audio::default();
    root_denied.request_world_sequence(7, true, request, options)?;
    // A second root child establishes a distinct local-view listener.
    client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .insert(8, Entity::default());
    for a in [
        &mut client.audio,
        &mut direct,
        &mut denied,
        &mut cross,
        &mut root_denied,
    ] {
        a.set_listener(listener);
    }
    let context = Context {
        root: client.world_entities.get(&0),
        base: client.world_base,
        local_view: 7,
    };
    let other = Context {
        local_view: 8,
        ..context
    };
    let root_listener = Context {
        local_view: 0,
        ..context
    };
    let mut nonzero = 0;
    for tick in 0..500 {
        let cache = &mut client.scripts.source.cache;
        client
            .audio
            .tick_in_worlds(cache, options, Some(&context))?;
        direct.tick(cache, options)?;
        denied.tick_in_worlds(cache, options, Some(&other))?;
        cross.tick_in_worlds(cache, options, Some(&other))?;
        root_denied.tick_in_worlds(cache, options, Some(&root_listener))?;
        let expected = direct.mix_stereo(441);
        let actual = client.audio.mix_stereo(441);
        assert_eq!(actual, expected, "moving view sound tick {tick}");
        assert_eq!(
            cross.mix_stereo(441),
            expected,
            "cross-world flag tick {tick}"
        );
        assert!(denied.mix_stereo(441).iter().all(|&v| v == 0));
        assert!(root_denied.mix_stereo(441).iter().all(|&v| v == 0));
        nonzero += actual.iter().filter(|&&v| v != 0).count();
    }
    assert!(nonzero > 100);
    assert_eq!(client.audio.pending_count(), 0);
    println!("Actual child graphic {spot}/sequence {sequence}/frame {frame}/sound {}: {nonzero} nonzero stereo samples; moved-pose PCM matches control; cross-world flag gates other-child playback and root stays silent",sound.id);
    Ok(())
}
