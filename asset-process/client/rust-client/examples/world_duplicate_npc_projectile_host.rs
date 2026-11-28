#[path = "../tests/support/duplicate_camera.rs"]
mod fixture;
use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, game_packet, protocol::Packet,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let mut endpoints = gielinor_client::buffer::Reader::new(include_bytes!(
        "../tests/fixtures/world-duplicate-npc-projectile-jvm.bin"
    ));
    assert_eq!(endpoints.u32()?, 512);
    let cases = fixture::cases();
    for case in &cases {
        client.world_entities.clear();
        client.detached_world_entities.clear();
        client.active_world = 0;
        client.view_registry = Default::default();
        for (opcode, payload) in &case.packets {
            client.apply(
                game_packet::decode(&Packet {
                    opcode: *opcode,
                    payload: payload.clone(),
                })?
                .unwrap(),
            )?;
        }
        let (base, npcs) = fixture::setup(case, &mut client.world_entities);
        client.world_base = base;
        client.data.position[0] = 0;
        client.npc_state = npcs;
        let tile = [0, endpoints.u32()? as i32, endpoints.u32()? as i32];
        let _view = endpoints.u32()?;
        let standard = [
            f32::from_bits(endpoints.u32()?),
            f32::from_bits(endpoints.u32()?),
        ];
        let expected = [
            f32::from_bits(endpoints.u32()?),
            f32::from_bits(endpoints.u32()?),
        ];
        let scene = gielinor_client::scene::Scene::empty(base[0], base[1])?;
        client.scripted_projectiles = Default::default();
        client.scripted_events.clear();
        let endpoint = gielinor_client::scripted_projectile::Endpoint {
            tile,
            entity: 42,
            offset: [0; 3],
        };
        client.scripted_events.push_back((
            100,
            gielinor_client::scripted_projectile::Event::Create {
                key: 1,
                definition: gielinor_client::scripted_projectile::Definition {
                    spot: 0,
                    source: endpoint.clone(),
                    target: endpoint,
                    start: 0,
                    end: 10,
                    curves: [0; 3],
                },
            },
        ));
        client.apply_scripted_projectile_events(&scene, &Default::default())?;
        let position = client.scripted_projectiles.effects[&1].motion.position;
        assert_eq!(
            [position[0], position[2]],
            expected,
            "id{} mask{} rebased{}",
            case.id,
            case.mask,
            case.rebased
        );
        client.scripted_projectiles = Default::default();
        client.projectiles.clear();
        client.actor_cycle = 100;
        client.apply(game_packet::Update::Projectile(
            gielinor_client::projectile::Request {
                id: 0,
                source: tile,
                target: tile,
                heights: [0; 2],
                entities: [42; 2],
                start: 0,
                end: 10,
                slope: 0,
                offset: 0,
            },
        ))?;
        client.actor_cycle = 101;
        client.draw_projectiles_in_views(&scene, &Default::default(), 0)?;
        let position = client.projectiles[0].motion.position;
        let local = [
            position[0] - (base[0] * 128) as f64,
            position[1] - (base[1] * 128) as f64,
        ];
        assert_eq!(
            local,
            standard.map(|n| n as i32 as f64),
            "normal projectile id{} mask{} rebased{}",
            case.id,
            case.mask,
            case.rebased
        );
    }
    assert_eq!(endpoints.remaining(), 0);
    println!("{} original duplicate NPC normal/scripted projectile endpoints passed actual-cache creation/draw",cases.len());
    Ok(())
}
