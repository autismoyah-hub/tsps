use gielinor_client::{
    buffer::Reader,
    disk_cache::DiskCache,
    interface_script,
    interface_state::Interfaces,
    item::Definitions,
    vm::{Object, Stacks},
    widget::Widget,
};
fn text(r: &mut Reader) -> String {
    let n = r.u16().unwrap() as usize;
    std::str::from_utf8(r.bytes(n).unwrap()).unwrap().into()
}
fn main() {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20).unwrap();
    let defs = Definitions::load(&mut cache).unwrap();
    let mut state = Interfaces::default();
    let mut widget = Widget::empty(0, 12);
    widget.font = 495;
    widget.raw_size = [100, 50];
    state.install_group(0, vec![widget]).unwrap();
    state.root = Some(0);
    for opcode in [1437, 1439] {
        state.runtime.get_mut(&0).unwrap().listeners.insert(
            opcode,
            (vec![gielinor_client::vm::Value::Int(opcode as i32)], None),
        );
    }
    state.scripts.clear();
    let mut active = [Some(0), Some(0)];
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-editor-mouse-jvm.bin"
    ));
    for case in 0..r.u32().unwrap() {
        let op = r.u32().unwrap() as u16;
        let a: [i32; 3] = std::array::from_fn(|_| r.u32().unwrap() as i32);
        let dot = r.u8().unwrap() != 0;
        let input = text(&mut r);
        if op >= 19000 {
            let editor = state.runtime.get_mut(&0).unwrap().editor.as_mut().unwrap();
            match op {
                19000 => editor
                    .mouse_press([a[0], a[1]], a[2] & 1 != 0, a[2] & 2 != 0)
                    .unwrap(),
                19001 => editor.mouse_drag([a[0], a[1]]).unwrap(),
                19002 => editor.text.set_focus(a[0] == 1),
                19003 => {
                    for _ in 0..a[0] {
                        editor.tick();
                    }
                }
                _ => unreachable!(),
            }
            let events = editor.text.take_events();
            for opcode in events {
                let r = &state.runtime[&0];
                state
                    .scripts
                    .push_back(gielinor_client::interface_state::PendingEvent::Script(
                        gielinor_client::interface_state::ScriptEvent {
                            widget: 0,
                            context: Default::default(),
                            arguments: r.listeners[&opcode].0.clone(),
                        },
                    ));
            }
        } else if op == 0 {
            let layout = state.layout([160, 96]).unwrap();
            state.prepare_editors(&mut cache, &layout).unwrap();
        } else {
            let mut stack = Stacks::default();
            if op == 1112 || op == 1135 {
                stack.push_object(Object::String(input)).unwrap();
            } else {
                for value in &a[..if op == 1142 || op == 1100 {
                    2
                } else if op == 1114 {
                    3
                } else {
                    1
                }] {
                    stack.push_int(*value).unwrap();
                }
            }
            let explicit = case % 3 == 0;
            if explicit {
                stack.push_int(0).unwrap();
            }
            assert!(interface_script::execute(
                op + if explicit { 1000 } else { 0 },
                dot,
                &mut stack,
                &mut cache,
                &mut state,
                &mut active,
                [160, 96],
                &defs
            )
            .unwrap());
            assert!(stack.ints().is_empty());
            assert!(stack.objects().is_empty());
        }
        let expected_pending = text(&mut r);
        let expected_hint_pending = text(&mut r);
        let expected_text = text(&mut r);
        let expected_hint = text(&mut r);
        let ready = r.u8().unwrap() != 0;
        let expected: [i32; 19] = std::array::from_fn(|_| r.u32().unwrap() as i32);
        let expected_events = (0..r.u32().unwrap())
            .map(|_| r.u32().unwrap() as i32)
            .collect::<Vec<_>>();
        let actual_events = state
            .scripts
            .drain(..)
            .map(|event| match event {
                gielinor_client::interface_state::PendingEvent::Script(event)
                | gielinor_client::interface_state::PendingEvent::Deferred { event, .. } => {
                    match event.arguments[0] {
                        gielinor_client::vm::Value::Int(id) => id,
                        _ => panic!(),
                    }
                }
                _ => panic!(),
            })
            .collect::<Vec<_>>();
        assert_eq!(actual_events, expected_events, "events case{case} op{op}");
        let w = &state.widgets[&0];
        let e = state.runtime[&0].editor.as_ref().unwrap();
        assert_eq!(
            (&w.text, &e.pending_placeholder),
            (&expected_pending, &expected_hint_pending),
            "pending case{case} op{op}"
        );
        assert_eq!(
            (
                String::from_utf16_lossy(&e.text.value),
                String::from_utf16_lossy(&e.placeholder)
            ),
            (expected_text, expected_hint),
            "text case{case} op{op}"
        );
        assert_eq!(e.text.ready, ready, "ready case{case}");
        assert_eq!(
            [
                w.font,
                e.text.anchor,
                e.text.cursor,
                e.scroll[0],
                e.scroll[1],
                e.text.clock as i32,
                e.text.filter,
                e.text.max_chars,
                e.options.align[0],
                e.options.align[1],
                e.options.line_height,
                e.options.mask,
                e.options.wrap,
                e.colors[0] as i32,
                e.options.width,
                e.options.max_lines,
                e.selection_mode as i32,
                e.click_anchor,
                e.click_ticks as i32,
            ],
            expected,
            "state case{case} op{op}"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("4096 original editable-widget active/explicit scripts and deferred text/font initialization transitions pass");
}
