//! Cache-backed view-qualified dispatch; does not connect to a server.
use gielinor_client::{client_state::ClientState, context_menu::Action, disk_cache::DiskCache,
    game_packet::Update, world_entity::{Entity, State, Definition}, world_action, Result};
fn bits(values: &[(u32, u8)]) -> Vec<u8> {
    let mut out = vec![0; values.iter().map(|v| v.1 as usize).sum::<usize>().div_ceil(8)];
    let mut cursor = 0;
    for &(value, count) in values { for shift in (0..count).rev() {
        out[cursor / 8] |= (((value >> shift) & 1) as u8) << (7 - cursor % 8); cursor += 1;
    }}
    out
}
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(std::env::args().nth(1).expect("cache"), 64 << 20)?)?;
    let mut definitions = c.npcs.values().filter(|d| d.interactable && d.transforms.is_none())
        .map(|d| d.id as u16).collect::<Vec<_>>();
    definitions.sort();
    let [root_id, child_id] = [definitions[0], definitions[1]];
    c.apply(Update::NpcOrigin([12, 20]))?;
    c.apply(Update::NpcInfo { large: false, payload: bits(&[
        (0, 8), (1234, 16), (0, 1), (0, 1), (0, 6), (0, 3), (0, 6), (1, 1), (root_id as u32, 14),
    ]) })?;
    let root = c.npc_state.npcs[&1234].clone();
    let mut child = root.clone(); child.id = child_id; child.position = [6412, 6420];
    let mut entity = Entity::default();
    entity.definition = Some(Definition::decode(1, &[24, 2, 0])?);
    entity.npcs.npcs.insert(1234, child);
    let mut world = State::default(); world.entities.insert(7, entity);
    c.world_entities.insert(0, world);
    c.menu_action(Action::ExamineNpc(1234).in_view(7))?;
    assert_eq!(c.outbound.pop_front().unwrap(), world_action::examine(0, child_id, [0; 2])?);
    c.menu_action(Action::ExamineNpc(1234))?;
    assert_eq!(c.outbound.pop_front().unwrap(), world_action::examine(0, root_id, [0; 2])?);
    let action = Action::Npc { index: 1234, operation: 1, sub: 0 }.in_view(7);
    c.menu_action(action)?;
    assert_eq!(c.outbound.pop_front().unwrap(), world_action::npc(1, 1234, 0, false)?);
    assert_eq!(c.map_flag, Some((6412 << 14) | 6420));
    // View policy is checked again at dispatch after a menu has opened.
    c.interaction_policy.apply(gielinor_client::interaction_policy::Update::Set { world: 7, input: 0, actions: 0 });
    c.menu_action(action)?; assert!(c.outbound.is_empty());
    c.interaction_policy.remove(7);
    // Neither missing NPCs nor missing worlds may fall back to the root copy.
    let npc = c.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().npcs.npcs.remove(&1234).unwrap();
    c.menu_action(action)?; assert!(c.outbound.is_empty());
    c.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().npcs.npcs.insert(1234, npc);
    let entity = c.world_entities.get_mut(&0).unwrap().entities.remove(&7).unwrap();
    c.menu_action(action)?; assert!(c.outbound.is_empty());
    c.world_entities.get_mut(&0).unwrap().entities.insert(7, entity);
    // Original config collision/interaction mode 1 permits examines only;
    // mode 0 suppresses both, while mode 2 allows ordinary interactions.
    for mode in 0..=2 {
        c.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().definition.as_mut().unwrap().collision_type = mode;
        assert_eq!(c.scene_action_allowed(action)?, mode == 2);
        assert_eq!(c.scene_action_allowed(Action::ExamineNpc(1234).in_view(7))?, mode != 0);
    }
    assert_eq!(c.npc_state.npcs[&1234].id, root.id);
    let mut initial = vec![((3200 << 14) | 3200, 30)];
    initial.extend(std::iter::repeat_n((0, 18), 2046));
    c.initialize_players(&bits(&initial), 1)?;
    {
        let state = c.player_state.as_mut().unwrap();
        let local = state.root_actor(1).unwrap().clone();
        let mut root_player = local.clone(); root_player.position = [0, 3201, 3202];
        let mut child_player = local.clone(); child_player.position = [0, 6411, 6413];
        state.views.insert(0, gielinor_client::player_info::View {
            base: [3200, 3200], dimensions: [104, 104], players: [(1, local), (2, root_player)].into(),
        });
        state.views.insert(7, gielinor_client::player_info::View {
            base: [6400, 6400], dimensions: [24, 40], players: [(2, child_player)].into(),
        });
    }
    let player_action = Action::Player { index: 2, operation: 1 }.in_view(7);
    c.menu_action(player_action)?;
    assert_eq!(c.outbound.pop_front().unwrap(), world_action::player(1, 2, false)?);
    assert_eq!(c.map_flag, Some((6411 << 14) | 6413));
    c.player_state.as_mut().unwrap().views.get_mut(&7).unwrap().players.remove(&2);
    c.menu_action(player_action)?; assert!(c.outbound.is_empty());
    let selection = gielinor_client::spell_action::Selection {
        widget: 1, logical: 1, child: 2, item: -1, mask: 15, option: "Cast".into(), target: "Test".into(),
    };
    c.selected_spell = Some(selection.clone());
    c.menu_action(Action::CastWorld { kind: 2, id: 2, tile: [6411, 6413] }.in_view(7))?;
    assert!(c.outbound.is_empty()); assert!(c.selected_spell.is_none());
    c.selected_spell = Some(selection.clone());
    c.menu_action(Action::CastWorld { kind: 1, id: 1234, tile: [6412, 6420] }.in_view(7))?;
    assert_eq!(c.outbound.pop_front().unwrap(), gielinor_client::spell_action::world(1, 1234, [6412, 6420], false, selection.source())?);
    assert!(c.selected_spell.is_none());
    let mut oracle = gielinor_client::buffer::Reader::new(include_bytes!("../tests/fixtures/world-action-gates-jvm.bin"));
    let count = oracle.u32()?;
    assert_eq!(count, 162);
    for case in 0..count {
        let mode = oracle.u8()?; let local = oracle.u8()? != 0; let owner = oracle.u8()?;
        let opcode = oracle.u32()?; let expected = oracle.u8()? != 0;
        let entity = c.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap();
        entity.definition.as_mut().unwrap().collision_type = mode; entity.owner_type = owner;
        c.player_state.as_mut().unwrap().local_view = if local { 7 } else { 0 };
        let action = match opcode {
            3 => Action::Object { id: 1, tile: [1, 2], operation: 1, sub: 0 },
            9 => Action::Npc { index: 1234, operation: 1, sub: 0 },
            18 => Action::Ground { id: 1, tile: [1, 2], operation: 1, sub: 0 },
            44 => Action::Player { index: 2, operation: 1 },
            1002 => Action::Examine { kind: 1, id: 1, tile: [1, 2] },
            1003 => Action::ExamineNpc(1234),
            1004 => Action::Examine { kind: 2, id: 1, tile: [1, 2] },
            63 => Action::WorldEntity { id: 8, operation: 1 },
            1013 => Action::WorldEntity { id: 1, operation: 6 },
            _ => unreachable!(),
        };
        assert_eq!(c.scene_action_allowed(action.in_view(7))?, expected, "original gate {case}, opcode {opcode}");
    }
    assert_eq!(oracle.remaining(), 0);
    println!("All 162 original child config/action gate decisions passed");
    println!("View-qualified NPC dispatch/examine, duplicate indexes, map destination, policy recheck and stale-view rejection passed");
    Ok(())
}
