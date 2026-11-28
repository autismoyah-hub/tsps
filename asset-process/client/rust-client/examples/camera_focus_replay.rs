//! Cache-backed camera target ownership; no network traffic or cache mutation.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    game_packet::Update,
    player_info::View,
    rebuild::WorldEntity,
    world_entity::{Entity, Pose, State},
    Result,
};
fn bits(values: &[(u32, u8)]) -> Vec<u8> {
    let mut out = vec![
        0;
        values
            .iter()
            .map(|v| v.1 as usize)
            .sum::<usize>()
            .div_ceil(8)
    ];
    let mut cursor = 0;
    for &(value, count) in values {
        for shift in (0..count).rev() {
            out[cursor / 8] |= (((value >> shift) & 1) as u8) << (7 - cursor % 8);
            cursor += 1;
        }
    }
    out
}
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let mut initial = vec![(1 << 28 | 3200 << 14 | 3200, 30)];
    initial.extend(std::iter::repeat_n((0, 18), 2046));
    c.initialize_players(&bits(&initial), 1)?;
    c.world_base = [3200; 2];
    c.data.position = [1, 3200, 3200];
    let mut player = c
        .player_state
        .as_ref()
        .unwrap()
        .root_actor(1)
        .unwrap()
        .clone();
    player.path.position = [3201 * 128 + 64, 3202 * 128 + 64];
    player.position = [1, 3201, 3202];
    let mut child = player.clone();
    child.path.position = [6412 * 128, 6420 * 128];
    child.position = [3, 6412, 6420];
    let players = c.player_state.as_mut().unwrap();
    players.views.insert(
        0,
        View {
            base: [3200; 2],
            dimensions: [104; 2],
            players: [(1, player.clone())].into(),
        },
    );
    players.views.insert(
        7,
        View {
            base: [6400; 2],
            dimensions: [24, 40],
            players: [(1, child.clone())].into(),
        },
    );
    c.view_registry.insert(7);
    let mut entity = Entity {
        plane: 2,
        dimensions: [24, 40],
        pose: Pose {
            x: 3000,
            y: 0,
            z: 4000,
            angle: 0,
        },
        rebuild: Some(WorldEntity {
            base: [6400; 2],
            dimensions: [24, 40],
            chunks: vec![],
            regions: vec![],
        }),
        ..Entity::default()
    };
    let npc_id = *c.npcs.keys().min().unwrap();
    c.apply(Update::NpcOrigin([12, 20]))?;
    c.apply(Update::NpcInfo {
        large: false,
        payload: bits(&[
            (0, 8),
            (1234, 16),
            (0, 1),
            (0, 1),
            (0, 6),
            (0, 3),
            (0, 6),
            (1, 1),
            (npc_id, 14),
        ]),
    })?;
    let root_npc = c.npc_state.npcs.get_mut(&1234).unwrap();
    root_npc.path.position = [3203 * 128, 3204 * 128];
    let mut npc = root_npc.clone();
    npc.path.position = [6412 * 128, 6420 * 128];
    entity.npcs.npcs.insert(1234, npc);
    let mut root = State::default();
    root.entities.insert(7, entity);
    c.world_entities.insert(0, root);
    let centre = [3200 * 128 + 3000, 3200 * 128 + 4000];
    assert_eq!(
        c.camera_focus(),
        Some((centre, 3)),
        "default local target prefers child copy"
    );
    c.camera.follow_target = Some((0, 1));
    assert_eq!(c.camera_focus(), Some((centre, 3)));
    c.camera.follow_target = Some((1, 1234));
    assert_eq!(
        c.camera_focus(),
        Some(([3203 * 128, 3204 * 128], 1)),
        "NPC root priority"
    );
    c.npc_state.npcs.remove(&1234);
    assert_eq!(
        c.camera_focus(),
        Some((centre, 2)),
        "child NPC projection and plane"
    );
    c.camera.follow_target = Some((2, 7));
    assert_eq!(
        c.camera_focus(),
        Some((centre, 1)),
        "entity uses parent plane"
    );
    c.camera.follow_target = Some((3, 2 << 28 | 6412 << 14 | 6420));
    assert_eq!(
        c.camera_focus(),
        Some(([centre[0] + 64, centre[1] + 64], 0)),
        "tile target ignores packed plane like xf.af"
    );
    c.camera.follow_target = Some((0, 65537));
    assert_eq!(c.camera_focus(), None, "no truncated actor ID alias");
    c.camera.follow_target = Some((2, 8));
    assert_eq!(c.camera_focus(), None, "missing entity");
    c.camera.follow_target = Some((0, 1));
    c.world_entities.get_mut(&0).unwrap().entities.remove(&7);
    assert_eq!(
        c.camera_focus(),
        Some(([3200 * 128 + 1536, 3200 * 128 + 2560], 3)),
        "registered child without projection owner"
    );
    c.view_registry.remove(7);
    assert_eq!(
        c.camera_focus(),
        Some((player.path.position, 1)),
        "removed view falls back to root player"
    );
    println!("Camera focus: default/explicit child player priority, root/child NPCs, entity parent plane, tile projection, missing targets and view removal passed");
    c.camera.mode = 1;
    c.camera.locked = true;
    c.camera.focus = [1000, -240, 2000];
    c.camera.eye = [900, -1000, 1500];
    c.camera.tracked = [1100, 2100];
    c.world_base((3208, 3184));
    assert_eq!(c.camera.focus, [-24, -240, 4048]);
    assert_eq!(c.camera.eye, [-124, -1000, 3548]);
    assert_eq!(c.camera.tracked, [76, 4148]);
    assert!(!c.camera.locked);
    assert_eq!(c.camera.mode, 1);
    assert_eq!(
        c.camera_focus(),
        Some((player.path.position, 1)),
        "global target moved during rebase"
    );
    println!("Client world-base update shifts free camera, eye and tracked target once and releases camera lock");
    Ok(())
}
