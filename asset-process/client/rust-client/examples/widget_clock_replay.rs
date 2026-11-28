//! Clock cadence and input preservation through actual ClientState drag steps.
use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, input::{Input, Update},
    context_menu::{Action, Entry}, widget::Widget, widget_drag::{Drag, Motion}, game_clock::Clock, Result};
fn install_drag(client: &mut ClientState, threshold: i32) {
    let widget = client.interfaces.widgets.get_mut(&1).unwrap();
    widget.drag_zone = 5; widget.drag_threshold = threshold;
    client.widget_drag = Some(Drag { instances: [client.interfaces.runtime[&1].instance, client.interfaces.runtime[&0].instance], widget: 1, parent: 0, point: [12, 12],
        entries: vec![Entry { option: "Test".into(), target: String::new(), action: Action::Cancel,
            submenu: false, children: Vec::new() }],
        motion: Motion { age: 0, dragging: false, active: true, offset: [2, 2], origin: [10, 10],
            dead_zone: 5, dead_time: threshold } });
}
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?)?;
    let mut root = Widget::empty(0, 0); root.raw_size = [200, 200];
    let mut child = Widget::empty(1, 3); child.parent = Some(0);
    child.raw_position = [10, 10]; child.raw_size = [20, 20];
    c.interfaces.install_group(0, vec![root, child])?; c.interfaces.root = Some(0);
    let mut traces = Vec::new();
    for interval in [10_000_000, 100_000_000] {
        install_drag(&mut c, 4); c.cycle = 0;
        let mut pending = Input::default(); pending.apply(Update::Button(0, true));
        pending.apply(Update::Pointer([70, 60]));
        let mut clock = Clock::default(); let mut trace = Vec::new();
        for at in (1..1_000_000_000).step_by(interval) {
            let ticks = clock.advance(at).ticks; c.cycle += ticks as i32;
            c.tick_interfaces_elapsed_with(ticks, &mut pending, true, |c| {
                assert!(c.tick_widget_drag()?.is_none());
                let d = c.widget_drag.as_ref().unwrap();
                trace.push((c.cycle, d.motion.age, d.motion.dragging)); Ok(())
            })?;
        }
        let ticks = clock.advance(999_999_999).ticks; c.cycle += ticks as i32;
        c.tick_interfaces_elapsed_with(ticks, &mut pending, true, |c| {
            assert!(c.tick_widget_drag()?.is_none()); let d = c.widget_drag.as_ref().unwrap();
            trace.push((c.cycle, d.motion.age, d.motion.dragging)); Ok(())
        })?;
        assert_eq!(trace.len(), 50); assert_eq!(trace[3], (4, 4, false)); assert_eq!(trace[4], (5, 5, true));
        traces.push(trace);
    }
    assert_eq!(traces[0], traces[1]);
    // Releasing during a zero-tick frame must stay pending until simulation resumes.
    install_drag(&mut c, 100); let mut pending = Input::default();
    pending.apply(Update::Button(0, true)); pending.apply(Update::Button(0, false));
    c.input.pointer = [90, 80]; let mut released = 0;
    c.tick_interfaces_elapsed_with(0, &mut pending, true, |_| panic!("zero-tick callback"))?;
    assert_eq!(pending.released_buttons, 1); assert_eq!(c.widget_drag.as_ref().unwrap().motion.age, 0);
    c.tick_interfaces_elapsed_with(5, &mut pending, true, |c| {
        if c.tick_widget_drag()?.is_some() { released += 1; } Ok(())
    })?;
    assert_eq!(released, 1); assert!(c.widget_drag.is_none()); assert_eq!(c.input.pointer, [90, 80]);
    assert_eq!(pending.released_buttons, 0);
    // Layout failure cannot destroy the drag or leak simulation input/cycle.
    install_drag(&mut c, 4); c.canvas = [-1, 503];
    assert!(c.tick_widget_drag().is_err()); assert_eq!(c.widget_drag.as_ref().unwrap().motion.age, 0);
    c.canvas = [765, 503]; let cycle = c.cycle;
    assert!(c.tick_interfaces_elapsed_with(3, &mut pending, true, |_| Err(gielinor_client::Error::Invalid("callback test"))).is_err());
    assert_eq!(c.cycle, cycle); assert_eq!(c.input.pointer, [90, 80]);
    println!("Widget drag: equal 50-tick cadence, exact threshold, zero-tick release retention, single completion and error restoration passed");
    Ok(())
}
