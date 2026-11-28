//! Cache-backed regression for visible Continue dispatch through the primary-action API.
use gielinor_client::{
    client_state::ClientState, context_menu::Action, disk_cache::DiskCache, interface_action,
    widget::Widget, Result,
};
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        std::env::args().nth(1).expect("cache"),
        64 << 20,
    )?)?;
    for legacy in [false, true] {
        let mut w = Widget::empty(0, 4);
        w.if3 = !legacy;
        w.raw_position = [20, 20];
        w.raw_size = [200, 30];
        w.text = "Click here to continue".into();
        w.flags = 1;
        if legacy {
            w.button = 6;
            w.button_text = "Continue".into();
        }
        c.interfaces = Default::default();
        c.interfaces.install_group(0, vec![w])?;
        c.interfaces.root = Some(0);
        let point = [30, 30];
        assert!(matches!(
            c.interface_menu(point)?.last().unwrap().action,
            Action::Continue(0)
        ));
        assert!(
            c.primary_interface_action(point)?,
            "visible Continue not dispatched; legacy={legacy}"
        );
        assert_eq!(
            c.outbound.pop_front(),
            Some(interface_action::resume(0, -1))
        );
        assert_eq!(c.interfaces.continue_widget, Some(0));
        c.primary_interface_action(point)?;
        assert!(c.outbound.is_empty(), "duplicate resume while waiting");
        c.interfaces.continue_widget = None;
        c.interfaces.widgets.get_mut(&0).unwrap().hidden = true;
        // ps.ap ignores the hidden flag on legacy non-container widgets.
        assert_eq!(c.primary_interface_action(point)?, legacy);
        if legacy {
            assert_eq!(c.outbound.pop_front(), Some(interface_action::resume(0, -1)));
        }
        assert!(c.outbound.is_empty());
        println!(
            "Continue dispatch, duplicate suppression and Java visibility rules passed: legacy={legacy}"
        );
    }
    // The real NPC-dialogue group is decoded without replacing its widgets.
    c.interfaces = Default::default();
    c.interfaces.load_group(&mut c.scripts.source.cache, 231)?;
    c.interfaces.root = Some(231);
    let layout = c.interfaces.layout(c.canvas)?;
    let candidates: Vec<_> = layout
        .order
        .iter()
        .filter_map(|id| {
            let w = &c.interfaces.widgets[id];
            let p = &layout.widgets[id];
            ((w.flags & 1 != 0 || w.button == 6)
                && !p.hidden
                && p.clip[0] < p.clip[2]
                && p.clip[1] < p.clip[3])
                .then_some((*id, [p.clip[0], p.clip[1]]))
        })
        .collect();
    assert!(
        !candidates.is_empty(),
        "cached NPC dialogue has no visible Continue candidate"
    );
    for (id, point) in candidates {
        c.interfaces.continue_widget = None;
        assert!(
            matches!(c.interface_menu(point)?.last().map(|e| e.action), Some(Action::Continue(v)) if v == id)
        );
        assert!(c.primary_interface_action(point)?);
        let r = &c.interfaces.runtime[&id];
        assert_eq!(
            c.outbound.pop_front(),
            Some(interface_action::resume(r.logical_id, r.child_index))
        );
        println!("Actual-cache NPC dialogue widget {id} Continue emits opcode 115");
    }
    // Dynamic widget identities and ordinary operation ordering must survive
    // consolidation of the previously separate default-action implementation.
    c.interfaces = Default::default();
    let mut parent = Widget::empty(0, 0);
    parent.raw_size = [300, 100];
    c.interfaces.install_group(0, vec![parent])?;
    c.interfaces.root = Some(0);
    let id = c.interfaces.create_child(0, 7, 4, true)?;
    let w = c.interfaces.widgets.get_mut(&id).unwrap();
    w.raw_size = [200, 30];
    w.flags = 6;
    w.actions = vec!["First".into(), "Second".into()];
    assert!(c.primary_interface_action([10, 10])?);
    assert_eq!(
        c.outbound.pop_front(),
        Some(interface_action::operation(0, 7, -1, 1, None)?)
    );
    c.interfaces.widgets.get_mut(&id).unwrap().flags |= 1;
    assert!(c.primary_interface_action([10, 10])?);
    assert_eq!(c.outbound.pop_front(), Some(interface_action::resume(0, 7)));
    println!("Dynamic child identity and Continue/default-operation priority passed");
    Ok(())
}
