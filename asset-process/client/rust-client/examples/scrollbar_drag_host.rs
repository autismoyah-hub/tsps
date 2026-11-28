use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    vm::{Host, Status, Value, Vm},
    widget::Widget,
    Result,
};
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    for id in [34, 6159, 8207] {
        client.interfaces = Default::default();
        client.widget_drag = None;
        client.menu = None;
        let owner = 65533u32 << 16;
        let mut root = Widget::empty(owner, 0);
        root.if3 = true;
        root.raw_size = [500, 400];
        client.interfaces.install_group(65533, vec![root])?;
        client.interfaces.root = Some(65533);
        let child = client.interfaces.create_child(owner, 1, 4, false)?;
        let w = client.interfaces.widgets.get_mut(&child).unwrap();
        w.raw_size = [40, 60];
        w.raw_position = [20, 30];
        client
            .interfaces
            .runtime
            .get_mut(&child)
            .unwrap()
            .drag_parent = Some(owner);
        client
            .interfaces
            .runtime
            .get_mut(&owner)
            .unwrap()
            .params
            .insert(2351, Value::Int(0));
        let script = client.script(id)?;
        let args = if id == 8207 {
            vec![owner as i32, -1]
        } else {
            vec![owner as i32, 0, 0]
        };
        let mut vm = Vm::new(script, args, vec![], vec![], 1000)?;
        assert_eq!(vm.run(&mut client, 1000)?, Status::Finished);
        let drag = client
            .widget_drag
            .as_ref()
            .expect("cache script should start drag");
        assert_eq!(drag.widget, child);
        assert_eq!(drag.parent, owner);
        assert_eq!(
            drag.motion.offset,
            if id == 6159 { [20, 0] } else { [0, 30] },
            "cache script {id}"
        );
    }
    println!("Actual cache scripts34,6159,8207 start scrollbar drags with selected child and half-size offsets");
    Ok(())
}
