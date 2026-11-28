use gielinor_client::{
    buffer::Reader, client_state::ClientState, context_menu::Action, disk_cache::DiskCache,
    spell_action::Selection, widget::Widget, Result,
};
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    client.canvas = [200, 100];
    let mut r = Reader::new(include_bytes!("../tests/fixtures/widget-menu-label-jvm.bin"));
    for case in 0..r.u32()? {
        let cp = r.u32()?;
        let padded = r.u8()? != 0;
        let op = r.u8()? != 0;
        let spell = r.u8()? != 0;
        let mut label = char::from_u32(cp).unwrap().to_string();
        if padded {
            label = format!(" \0{label}\u{1f} ");
        }
        let mut w = Widget::empty(1, 4);
        w.raw_size = [50, 50];
        w.flags = 2050;
        w.actions = vec![label.clone()];
        w.spell_action = label.clone();
        let selected = Selection::from_widget(1, 1, -1, -1, w.flags, &w);
        assert_eq!(
            selected.option,
            if spell { label.clone() } else { "null".into() },
            "selected case {case} U+{cp:04x}"
        );
        client.interfaces = Default::default();
        client.interfaces.install_group(0, vec![w])?;
        client.interfaces.root = Some(0);
        let menu = client.interface_menu([10, 10])?;
        assert_eq!(
            menu.iter()
                .any(|e| matches!(e.action, Action::Interface { .. })),
            op,
            "op case {case} U+{cp:04x}"
        );
        assert_eq!(
            menu.iter()
                .any(|e| matches!(e.action, Action::SelectSpell(_))),
            spell,
            "spell case {case} U+{cp:04x}"
        );
        for entry in menu {
            assert_eq!(entry.option, label);
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("512 actual ClientState operation/spell labels and selected-spell options");
    Ok(())
}
