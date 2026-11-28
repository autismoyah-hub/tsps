use gielinor_client::{
    client_state::ClientState, context_menu::Action, disk_cache::DiskCache,
    map_element::Definition, script_runtime::CacheSource, widget::Widget, Result,
};
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    c.canvas = [256; 2];
    let mut widget = Widget::empty(0, 0);
    widget.if3 = true;
    widget.content = 1400;
    widget.raw_size = c.canvas;
    c.interfaces.install_group(0, vec![widget])?;
    c.interfaces.root = Some(0);
    c.world_map.load(&mut c.scripts.source.cache)?;
    c.world_map.select(0);
    let area = c.world_map.area().unwrap().clone();
    c.world_map.zoom = 2.0;
    c.world_map.target_zoom = 2.0;
    c.world_map
        .detailed_image(&mut c.scripts.source.cache, true, c.canvas, 0.8)?;
    c.world_map.display_origin = [0, 0];
    let defs = Definition::load_all(&mut c.scripts.source.cache)?;
    let renderer = c.world_map.renderer.as_mut().unwrap();
    let key = renderer
        .icon_start(&area)?
        .into_iter()
        .find(|key| {
            renderer.event(*key).is_some_and(|e| {
                defs.get(&(e.element as u32))
                    .is_some_and(|d| d.actions.iter().any(Option::is_some))
            })
        })
        .expect("actual cache actionable map icon");
    let event = renderer.event(key).unwrap();
    c.world_map.center = [
        event.destination[1] - area.bounds[0] * 64,
        event.destination[2] - area.bounds[1] * 64,
    ];
    c.world_map
        .detailed_image(&mut c.scripts.source.cache, true, c.canvas, 0.8)?;
    let mut chosen = None;
    let mut chosen_point = [0; 2];
    for y in 112..145 {
        for x in 112..145 {
            for entry in c.interface_menu([x, y])? {
                if let Action::Map { .. } = entry.action {
                    chosen = Some(entry.action);
                    chosen_point = [x,y];
                    break;
                }
            }
            if chosen.is_some() {
                break;
            }
        }
        if chosen.is_some() {
            break;
        }
    }
    let action = chosen.expect("widget exposes an original map action");
    c.world_map.elements = Some(defs);
    c.scripts.source =
        CacheSource::new(DiskCache::open("tests/fixtures/map-event-cache", 1 << 20)?);
    c.menu_action(action)?;
    let Action::Map { opcode, event } = action else {
        unreachable!()
    };
    assert_eq!(c.scripts.variables.player(0), opcode as i32 - 998);
    assert_eq!(c.scripts.variables.player(1), event.element);
    for opcode in 1008..=1012 {
        c.menu_action(Action::Map { opcode, event })?;
        assert_eq!(c.scripts.variables.player(0), opcode as i32 - 998);
    }
    for kind in 15..=17 {
        assert!(c.run_map_event(kind, event, Some([123, 456]))? > 0);
        assert_eq!(c.scripts.variables.player(0), kind);
        assert_eq!(c.scripts.variables.player(4), 123);
        assert_eq!(c.scripts.variables.player(5), 456);
    }
    let mut stack = gielinor_client::vm::Stacks::default();
    event.query(6698, &mut stack)?;
    event.query(6699, &mut stack)?;
    assert_eq!(c.scripts.variables.player(2), stack.ints()[0]);
    assert_eq!(c.scripts.variables.player(3), stack.ints()[1]);
    assert_eq!(c.world_map.event, Some(event));
    assert!(c.outbound.is_empty());
    c.world_map.hover = Default::default();
    c.input.pointer = chosen_point;
    assert!(c.tick_interfaces(false)? > 0);
    assert_eq!(c.scripts.variables.player(0),15);
    assert_eq!(c.scripts.variables.player(4),chosen_point[0]);
    c.input.begin_cycle();assert!(c.tick_interfaces(false)? > 0);
    assert_eq!(c.scripts.variables.player(0),17);
    let retained=c.world_map.hover.previous.len();assert!(retained>0);
    c.input.pointer=[-1,-1];c.tick_interfaces(false)?;
    assert_eq!(c.world_map.hover.previous.len(),retained);
    assert_eq!(c.scripts.variables.player(0),17);
    let mut empty=None;
    for y in (0..256).step_by(8){for x in (0..256).step_by(8){if c.world_map.renderer.as_ref().unwrap().hits(&area,c.world_map.center,c.world_map.zoom,[0,0,256,256],[0,0],[x,y])?.is_empty(){empty=Some([x,y]);break;}}if empty.is_some(){break;}}
    c.input.pointer=empty.expect("map background without icons");
    assert!(c.tick_interfaces(false)? > 0);assert_eq!(c.scripts.variables.player(0),16);assert!(c.world_map.hover.previous.is_empty());assert!(c.script_failures.is_empty());

    println!("Actual-cache widget map menu, all five action event types, three hover event types, metadata queries, mouse arguments and normal-tick enter/repeat/exit/inactive retention passed");
    Ok(())
}
