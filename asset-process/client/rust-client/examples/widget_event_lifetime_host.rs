use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, input::Input,
    interface_events::Events, interface_state::Interfaces, script_runtime::CacheSource, vm::Value,
    widget::Widget, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    client.scripts.source = CacheSource::new(DiskCache::open(
        "tests/fixtures/widget-event-lifetime-vm-cache",
        1 << 20,
    )?);
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-event-lifetime-jvm.bin"
    ));
    assert_eq!(r.u32()?, 30);
    for case in 0..30 {
        let phase = r.u32()?;
        let dynamic = r.u8()? != 0;
        let mutation = r.u32()?;
        let expected = r.u32()?;
        if expected != 0 {
            assert_eq!(expected, 1);
            assert_eq!(r.u32()?, 42);
        }
        client.interfaces = Interfaces::default();
        let mut owner = Widget::empty(0, 0);
        owner.raw_size = [100, 100];
        client.interfaces.install_group(0, vec![owner])?;
        client.interfaces.root = Some(0);
        let id = if dynamic {
            let id = client.interfaces.create_child(0, 5, 4, false)?;
            client.interfaces.widgets.get_mut(&id).unwrap().raw_size = [100, 100];
            id
        } else {
            0
        };
        let opcode = [1408, 1402, 1403][phase as usize];
        client
            .interfaces
            .runtime
            .get_mut(&id)
            .unwrap()
            .listeners
            .insert(opcode, (vec![Value::Int(42)], None));
        let layout = client.interfaces.layout([200, 200])?;
        let mut events = Events::default();
        let mut input = Input {
            pointer: [10, 10],
            press_position: [10, 10],
            ..Default::default()
        };
        if phase == 1 {
            input.buttons = 1;
            input.pressed_buttons = 1;
            assert!(events
                .collect(&client.interfaces, &layout, &input, false, false)
                .is_empty());
            input.buttons = 0;
            input.pressed_buttons = 0;
        }
        let queued = events.collect(&client.interfaces, &layout, &input, false, false);
        assert_eq!(queued.len(), 1, "case{case} queue");
        if mutation != 0 {
            if dynamic {
                match mutation {
                    1 => {
                        client.interfaces.delete_child(id)?;
                    }
                    2 => {
                        client.interfaces.create_child(0, 5, 4, false)?;
                    }
                    3 => client.interfaces.delete_children(0),
                    4 => {
                        client.interfaces.widgets.remove(&0);
                    }
                    _ => unreachable!(),
                }
            } else {
                client.interfaces.widgets.remove(&0);
                if mutation == 2 {
                    client.interfaces.widgets.insert(0, Widget::empty(0, 0));
                }
            }
        }
        client.scripts.variables.set_player(0, -100);
        client.dispatch_interface_events(queued);
        assert!(
            client.script_failures.is_empty(),
            "case{case}: {:?}",
            client.script_failures
        );
        assert_eq!(
            client.scripts.variables.player(0),
            if expected == 1 { 42 } else { -100 },
            "case{case} phase{phase} dynamic{dynamic} mutation{mutation}"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("30 original ps.ac lifetime cases match collected events dispatched through cache-loaded ClientState");
    Ok(())
}
