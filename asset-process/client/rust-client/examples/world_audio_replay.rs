//! Cache-backed child zone packet -> delayed world-aware PCM regression.
use gielinor_client::{
    area_sound::Request,
    audio_runtime::Audio,
    audio_synth::Effect,
    buffer::Transform,
    client_state::ClientState,
    disk_cache::DiskCache,
    game_packet::Update,
    world_audio::Context,
    world_entity::{Entity, Pose, State},
    writer::Writer,
    zone::{Packet, Record},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("cache directory");
    let mut cache = DiskCache::open(&path, 64 << 20)?;
    let ids = cache
        .reference(4)?
        .groups
        .iter()
        .filter(|g| {
            g.id <= 65535 && g.files.iter().any(|f| f.id == 0) && !g.files.iter().any(|f| f.id == 1)
        })
        .map(|g| g.id)
        .collect::<Vec<_>>();
    let mut effect_id = None;
    for id in ids {
        let mut effect = Effect::decode(&cache.file(4, id, 0, [0; 4])?)?;
        effect.trim_delay();
        if effect.synthesize(false)?.iter().any(|&s| s != 0) {
            effect_id = Some(id as u16);
            break;
        }
    }
    let id = effect_id.expect("non-silent synthesized cache effect");
    let mut client = ClientState::load(DiskCache::open(&path, 64 << 20)?)?;
    client.world_base = [3200, 6400];
    client.preferences.device.set_game(7, 0)?;
    client.preferences.device.set_game(8, 127)?;
    client.preferences.device.set_game(9, 127)?;
    let mut entity = Entity::default();
    entity.dimensions = [24, 40];
    entity.pose = Pose {
        x: 4096,
        y: 0,
        z: 4096,
        angle: 256,
    };
    let mut root = State::default();
    root.visible.push(7);
    root.entities.insert(7, entity);
    client.world_entities.insert(0, root);
    client.apply(Update::ActiveWorld { id: 7, plane: 0 })?;
    let mut raw = Writer::new();
    raw.transformed(0, Transform::Sub);
    raw.transformed(0, Transform::Neg);
    raw.u8(8);
    raw.transformed(4, Transform::Neg);
    raw.transformed(1, Transform::Add);
    raw.u16(id);
    let bytes = raw.into_vec();
    let request = Request::decode([0, 12, 20], [0; 2], &bytes)?;
    client.apply(Update::Zone(Packet {
        absolute: None,
        origin: Some([0, 12, 20]),
        clear: false,
        records: vec![Record::Pending { kind: 14, bytes }],
    }))?;
    assert_eq!(client.audio.pending_count(), 1);
    assert!(client.world_entities[&0].entities[&7]
        .zone
        .sounds
        .is_empty());
    assert!(client.zone.sounds.is_empty());

    // Move after packet admission: the delayed request must resolve the new pose.
    client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .get_mut(&7)
        .unwrap()
        .pose
        .x += 2048;
    let point = client.world_entities[&0].entities[&7].transform_horizontal([12 * 128, 20 * 128]);
    let tile = [
        client.world_base[0] + ((point[0] as i32) >> 7),
        client.world_base[1] + ((point[2] as i32) >> 7),
    ];
    let listener = [tile[0] * 128 + 64, tile[1] * 128 + 64];
    let options = &client.preferences.device;
    let mut direct = Audio::default();
    let mut positioned = request.clone();
    positioned.tile = tile;
    direct.request_area(positioned, options)?;
    let mut denied = Audio::default();
    denied.request_world_area(7, request, options)?;
    for audio in [&mut client.audio, &mut direct, &mut denied] {
        audio.set_listener(listener);
    }
    let context = Context {
        root: client.world_entities.get(&0),
        base: client.world_base,
        local_view: 7,
    };
    let root_listener = Context {
        local_view: 0,
        ..context
    };
    let mut nonzero = 0;
    for tick in 0..500 {
        client
            .audio
            .tick_in_worlds(&mut cache, options, Some(&context))?;
        direct.tick(&mut cache, options)?;
        denied.tick_in_worlds(&mut cache, options, Some(&root_listener))?;
        let actual = client.audio.mix_stereo(441);
        assert_eq!(
            actual,
            direct.mix_stereo(441),
            "delayed transform PCM at tick {tick}"
        );
        assert!(
            denied.mix_stereo(441).iter().all(|&s| s == 0),
            "root listener heard child effect"
        );
        nonzero += actual.iter().filter(|&&s| s != 0).count();
    }
    assert!(nonzero > 100);
    assert_eq!(client.audio.pending_count(), 0);
    assert!(!client.audio.active());
    println!("Child zone effect {id}: {nonzero} nonzero stereo samples; delayed moved-pose PCM equals root-positioned control; root listener silent");
    Ok(())
}
