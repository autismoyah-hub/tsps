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
    let mut active = [Some(0), Some(0)];
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-editor-script-jvm.bin"
    ));
    for case in 0..r.u32().unwrap() {
        let op = r.u32().unwrap() as u16;
        let a: [i32; 3] = std::array::from_fn(|_| r.u32().unwrap() as i32);
        let dot = r.u8().unwrap() != 0;
        let input = text(&mut r);
        if op == 0 {
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
        let expected: [i32; 14] = std::array::from_fn(|_| r.u32().unwrap() as i32);
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
                e.colors[0] as i32
            ],
            expected,
            "state case{case} op{op}"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("4096 original editable-widget active/explicit scripts and deferred text/font initialization transitions pass");
}
