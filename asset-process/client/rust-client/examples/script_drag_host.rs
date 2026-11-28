use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    vm::{Control, Host, Stacks},
    widget::Widget,
    widget_drag::{Drag, Motion},
    Result,
};
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let font_id = client
        .scripts
        .source
        .cache
        .reference(13)?
        .groups
        .iter()
        .find(|g| g.name_hash == Some(gielinor_client::disk_cache::name_hash("b12_full")))
        .unwrap()
        .id;
    let font = gielinor_client::font::Font::load(&mut client.scripts.source.cache, font_id)?;
    let mut r = Reader::new(include_bytes!("../tests/fixtures/script-drag-jvm.bin"));
    for case in 0..r.u32()? {
        let opcode = r.u32()?;
        let dot = r.u8()? != 0;
        let menu = r.u8()? != 0;
        let busy = r.u8()? != 0;
        let parent = r.u8()? != 0;
        let widget = r.u8()? != 0;
        let menu_count = r.u32()?;
        let offset = [r.u32()? as i32, r.u32()? as i32];
        let expected = r.u32()?;
        let expected_parent = r.u8()? != 0;
        let expected_offset = [r.u32()? as i32, r.u32()? as i32];
        let age = r.u32()? as i32;
        let dragging = r.u8()? != 0;
        assert_eq!(r.u32()?, 0);
        let captured = r.u8()? != 0;
        let captured_param = if captured { Some(r.u32()?) } else { None };
        let entries = (0..menu_count)
            .map(|index| gielinor_client::context_menu::Entry {
                option: "test".into(),
                target: String::new(),
                action: gielinor_client::context_menu::Action::Button {
                    id: if index == 1 { 123 } else { 99 },
                    kind: 1,
                },
                submenu: false,
                children: vec![],
            })
            .collect::<Vec<_>>();
        client.interfaces = Default::default();
        let interfaces = &mut client.interfaces;
        interfaces.install_group(0, vec![Widget::empty(0, 0), Widget::empty(1, 4)])?;
        interfaces.runtime.get_mut(&1).unwrap().drag_parent = parent.then_some(0);
        client.pointer_menu = entries;
        client.active_widgets = [widget.then_some(1); 2];
        if !widget {
            client.interfaces.widgets.remove(&1);
            client.interfaces.runtime.remove(&1);
        }
        client.widget_drag = busy.then(|| Drag {
            instances: [0; 2],
            widget: 2,
            parent: 99,
            motion: Motion {
                age: 0,
                dragging: false,
                active: true,
                offset: [0; 2],
                origin: [0; 2],
                dead_zone: 0,
                dead_time: 0,
            },
            point: [0; 2],
            entries: vec![],
        });
        client.menu = menu
            .then(|| gielinor_client::context_menu::Menu::open(vec![], [0; 2], [765, 503], &font));
        let mut stack = Stacks::default();
        stack.push_int(offset[0])?;
        stack.push_int(offset[1])?;
        if opcode == 3108 {
            stack.push_int(1)?;
        }
        assert_eq!(
            client.extended(opcode as u16, dot, &mut stack)?,
            Control::Continue
        );
        assert!(stack.ints().is_empty());
        let result = client.widget_drag.as_ref().filter(|d| d.widget == 1);
        let actual = if busy {
            1
        } else if result.is_some() {
            2
        } else {
            0
        };
        assert_eq!(actual, expected, "case {case}");
        assert_eq!(result.is_some(), expected_parent, "parent case {case}");
        assert_eq!(
            result.as_ref().is_some_and(|d| !d.entries.is_empty()),
            captured,
            "menu capture case {case}"
        );
        if let Some(param) = captured_param {
            assert!(
                matches!(result.as_ref().unwrap().entries.last().unwrap().action,gielinor_client::context_menu::Action::Button{id,..} if id==param)
            );
        }
        if let Some(drag) = result {
            assert_eq!(drag.parent, 0);
            assert_eq!(drag.widget, 1);
            assert_eq!(drag.motion.offset, expected_offset);
            assert_eq!(drag.motion.age, age);
            assert_eq!(drag.motion.dragging, dragging);
            assert_eq!(
                drag.point,
                [
                    drag.motion.origin[0].wrapping_add(offset[0]),
                    drag.motion.origin[1].wrapping_add(offset[1])
                ]
            );
        } else {
            assert_eq!(expected_offset, [0, 0]);
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("Actual ClientState: 256 script drag starts; both opcodes, menu/busy/null guards and signed offsets");
    Ok(())
}
