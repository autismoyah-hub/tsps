use gielinor_client::{
    buffer::Reader,
    interface_state::{Interfaces, PendingEvent},
    vm::Value,
    widget::Widget,
};
fn main() {
    let mut cache =
        gielinor_client::disk_cache::DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)
            .unwrap();
    let defs = gielinor_client::item::Definitions::load(&mut cache).unwrap();
    let mut state = Interfaces::default();
    let widgets = (0..3)
        .map(|id| {
            let mut w = Widget::empty(id, 0);
            w.kind = if id < 2 { 12 } else { 4 };
            w
        })
        .collect();
    state.install_group(0, widgets).unwrap();
    for id in 0..2 {
        let r = state.runtime.get_mut(&id).unwrap();
        r.editor.as_mut().unwrap().text.select(2, 5);
        r.listeners
            .insert(1436, (vec![Value::Int(id as i32)], None));
    }
    state.scripts.clear();
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-editor-focus-jvm.bin"
    ));
    for case in 0..r.u32().unwrap() {
        let op = r.u32().unwrap();
        let target = r.u32().unwrap() as i32;
        let value = r.u32().unwrap() as i32;
        let other = r.u32().unwrap() as i32;
        match op {
            0 => state.focus_editor((target >= 0).then_some(target as u32)),
            1 | 2 => {
                let mut stack = gielinor_client::vm::Stacks::default();
                stack.push_int(value).unwrap();
                let explicit = case % 3 == 0;
                if explicit {
                    stack.push_int(target).unwrap();
                }
                let mut active = [Some(target as u32); 2];
                assert!(gielinor_client::interface_script::execute(
                    (if op == 1 { 1140 } else { 1141 }) + if explicit { 1000 } else { 0 },
                    case % 2 == 0,
                    &mut stack,
                    &mut cache,
                    &mut state,
                    &mut active,
                    [160, 96],
                    &defs
                )
                .unwrap());
                assert!(stack.ints().is_empty());
            }
            3 => state
                .runtime
                .get_mut(&(target as u32))
                .unwrap()
                .editor
                .as_mut()
                .unwrap()
                .text
                .select(value, other),
            4 => state.blur_editor(),
            _ => unreachable!(),
        }
        assert_eq!(
            state.focused_editor().map_or(-1, |id| id as i32),
            r.u32().unwrap() as i32,
            "owner case {case}"
        );
        for id in 0..2 {
            let text = &state.runtime[&id].editor.as_ref().unwrap().text;
            assert_eq!(
                text.focused,
                r.u8().unwrap() != 0,
                "focus case {case} widget {id}"
            );
            assert_eq!(text.enabled, r.u8().unwrap() != 0, "enabled case {case}");
            assert_eq!(text.anchor, r.u32().unwrap() as i32, "anchor case {case}");
            assert_eq!(text.cursor, r.u32().unwrap() as i32, "cursor case {case}");
        }
        let expected = (0..r.u32().unwrap())
            .map(|_| r.u32().unwrap() as i32)
            .collect::<Vec<_>>();
        let actual = state
            .scripts
            .drain(..)
            .map(|event| match event {
                PendingEvent::Script(event) | PendingEvent::Deferred { event, .. } => match event.arguments[0] {
                    Value::Int(id) => id,
                    _ => panic!("event argument"),
                },
                _ => panic!("unexpected lifecycle"),
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "callbacks case {case}");
    }
    assert_eq!(r.remaining(), 0);
    println!("4096 original editor focus transitions through active/explicit script dispatch");
}
