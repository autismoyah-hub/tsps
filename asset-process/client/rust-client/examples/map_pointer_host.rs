use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    vm::{Control, Host, Stacks},
    widget::Widget,
    Result,
};
fn command(c: &mut ClientState, op: u16, args: &[i32]) -> Result<()> {
    let mut s = Stacks::default();
    for &v in args {
        s.push_int(v)?;
    }
    assert_eq!(c.extended(op, false, &mut s)?, Control::Continue);
    Ok(())
}
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    c.canvas = [256, 256];
    let mut widget = Widget::empty(0, 0);
    widget.if3 = true;
    widget.content = 1400;
    widget.raw_size = c.canvas;
    let mut overlay = Widget::empty(1, 0);
    overlay.if3 = true;
    overlay.no_click_through = true;
    overlay.raw_size = c.canvas;
    overlay.hidden = true;
    c.interfaces.install_group(0, vec![widget, overlay])?;
    c.interfaces.root = Some(0);
    command(&mut c, 6602, &[0])?;
    command(&mut c, 6607, &[3200 << 14 | 3200])?;
    c.world_map.zoom = 2.0;
    c.world_map.target_zoom = 2.0;
    c.world_map
        .detailed_image(&mut c.scripts.source.cache, true, c.canvas, 0.8)?;
    c.world_map.display_origin = [0, 0];
    c.outbound.clear();
    let initial = c.world_map.center;
    c.input.pointer = [128, 128];
    c.input.button(0, true);
    c.tick_interfaces(false)?;
    assert_eq!(c.world_map.center, initial);
    assert!(c.outbound.iter().all(|p| p.opcode != 99));
    c.input.begin_cycle();
    c.input.pointer = [144, 116];
    c.tick_interfaces(false)?;
    assert_eq!(c.world_map.center, [initial[0] - 8, initial[1] - 6]);
    c.input.begin_cycle();
    c.input.button(0, false);
    c.tick_interfaces(false)?;
    assert_eq!(c.world_map.pointer.drag_center, [-1, -1]);
    command(&mut c, 6607, &[3200 << 14 | 3200])?;
    c.world_map.pointer.last_time = 0;
    c.input.begin_cycle();
    c.input.pointer = [128, 128];
    c.input.button(0, true);
    c.tick_interfaces(false)?;
    c.input.begin_cycle();
    c.input.button(0, false);
    c.input.button(0, true);
    c.tick_interfaces(false)?;
    let packet = c.outbound.iter().find(|p| p.opcode == 99).unwrap();
    assert_eq!(packet.payload, (3200u32 << 14 | 3200).to_le_bytes());
    c.outbound.clear();
    c.input.begin_cycle();
    c.input.button(0, true);
    c.tick_interfaces(true)?;
    assert!(c.outbound.iter().all(|p| p.opcode != 99));
    assert_eq!(c.world_map.pointer.drag_center, [-1, -1]);
    c.input.begin_cycle();
    c.input.button(0, false);
    c.input.button(2, true);
    c.preferences.mouse_camera = true;
    c.tick_interfaces(false)?;
    assert_eq!(c.world_map.pointer.drag_center, [-1, -1]);
    c.input.begin_cycle();
    c.input.button(2, false);
    c.input.button(2, true);
    c.preferences.mouse_camera = false;
    c.tick_interfaces(false)?;
    assert_eq!(c.world_map.pointer.drag_center, c.world_map.center);
    c.input.begin_cycle();
    c.input.button(2, false);
    c.input.button(0, true);
    c.world_map.target = [c.world_map.center[0] + 80, c.world_map.center[1] + 80];
    c.tick_interfaces(false)?;
    assert_eq!(c.world_map.pointer.drag_center, [-1, -1]);
    c.interfaces.widgets.get_mut(&1).unwrap().hidden = false;
    c.world_map.target = [-1, -1];
    c.input.begin_cycle();
    c.input.button(0, true);
    c.outbound.clear();
    c.tick_interfaces(false)?;
    assert_eq!(c.world_map.pointer.drag_center, [-1, -1]);
    assert!(c.outbound.iter().all(|p| p.opcode != 99));

    // Explicit opens preserve the script stack and reset an existing map's zoom/target.
    c.world_map.zoom = 4.0;
    c.world_map.target_zoom = 8.0;
    let before = c.world_map.center;
    command(&mut c, 6600, &[])?;
    assert_eq!(c.world_map.center, before);
    assert_eq!(c.world_map.target_zoom, 8.0); // no actor
    command(&mut c, 6620, &[0, 3200 << 14 | 3200])?;
    assert_eq!(c.world_map.display(), [3200, 3200]);
    assert_eq!(c.world_map.target, [-1, -1]);
    assert_eq!(c.world_map.zoom, c.world_map.target_zoom);
    command(&mut c, 6619, &[0, 3210 << 14 | 3220])?;
    assert_eq!(c.world_map.display(), [3210, 3220]);
    let (mut players, _) = gielinor_client::player_info::State::initialize(&[0; 4608], 42)?;
    players.players[42] = Some(gielinor_client::player_info::Player::at(
        [2, 3204, 3223],
        None,
        1,
        None,
    ));
    c.world_base = [3200; 2];
    c.player_state = Some(players);
    assert_eq!(c.local_map_coordinate(), [2, -3196, -3177]);
    command(&mut c, 6600, &[])?;
    let a = c.world_map.area().unwrap();
    assert_eq!(
        c.world_map.display(),
        a.position(a.origin[0], a.origin[1], a.origin[2]).unwrap()
    );
    let before = c.world_map.center;
    command(&mut c, 6620, &[-999, 0])?;
    assert_eq!(c.world_map.center, before);
    println!("Map input host: drag/release, exact click packet, pointer suppression, middle-button camera mode, target scroll, overlay blocking and local/target/fallback script opens passed");
    Ok(())
}
