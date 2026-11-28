use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_state::{Interfaces, RemoteMappings},
    remote_card::Card,
    remote_json,
    remote_url::Context,
    remote_widget::Loader,
    widget::Widget,
    Result,
};
fn close(client: &mut ClientState) -> Result<()> {
    client.interfaces.children.insert(
        987,
        gielinor_client::interface_packet::SubInterface {
            parent: 987,
            group: 0,
            kind: 0,
        },
    );
    client.interfaces.close(987)
}
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!("../tests/fixtures/remote-pointer-jvm.bin"));
    assert_eq!(r.u32()?, 160);
    for case in 0..160 {
        let dynamic = r.u8()? != 0;
        let binding = r.u32()?;
        let kind = r.u32()?;
        let point = [r.u32()? as i32, r.u32()? as i32];
        let status = r.u32()? as i32;
        let failed = r.u8()? != 0;
        let calls = r.u32()?;
        client.interfaces = Interfaces::default();
        client.remote = Context {
            requester: None,
            ..Default::default()
        };
        client.outbound.clear();
        client.input = Default::default();
        client.canvas = [100, 80];
        let mut w = Widget::empty(0, if dynamic { 0 } else { 11 });
        w.raw_size = [100, 80];
        client.interfaces.install_group(0, vec![w])?;
        client.interfaces.root = Some(0);
        let id = if dynamic {
            client.interfaces.create_child(0, 0, 11, true)?
        } else {
            0
        };
        let w = client.interfaces.widgets.get_mut(&id).unwrap();
        w.raw_size = [100, 80];
        w.flags = 1 << 22;
        let mut card = Card::parse(&remote_json::content("{}").unwrap(), None)?;
        card.id = Some("card".into());
        card.endpoint = Some("/go".into());
        card.click_bounds = [0., 0., 1., 1.];
        let rt = client.interfaces.runtime.get_mut(&id).unwrap();
        rt.remote_status = status;
        rt.remote_loader = Some(Box::new(Loader {
            status: 2,
            crm_attached: true,
            card: Some(std::sync::Arc::new(card)),
            ..Default::default()
        }));
        let mut maps = RemoteMappings::default();
        maps.endpoints.insert(Some("/go".into()), 17);
        rt.remote_mappings = Some(Box::new(maps));
        client.input.pointer = point;
        client.input.press_position = point;
        if kind == 0 {
            client.input.pressed_buttons = 1;
        } else {
            client.input.buttons = 1;
        }
        let layout = client.interfaces.layout(client.canvas)?;
        let pending = client.capture_widget_pointer(&layout);
        if binding != 0 {
            if dynamic && binding != 4 {
                client.interfaces.delete_children(0);
                if binding == 1 {
                    client.interfaces.create_child(0, 0, 11, true)?;
                }
            } else {
                close(&mut client)?;
                if !dynamic && binding == 1 {
                    client
                        .interfaces
                        .install_group(0, vec![Widget::empty(0, 3)])?;
                }
            }
        }
        let result = client.dispatch_widget_pointer(pending);
        assert_eq!(result.is_err(), failed, "case{case}");
        result?;
        assert_eq!(
            client.outbound.iter().filter(|p| p.opcode == 91).count(),
            calls as usize,
            "case{case}"
        );
        assert_eq!(
            client.remote.crm.event_kind,
            if calls == 0 { 0 } else { 1 },
            "case{case}"
        );
        if !dynamic && binding == 1 {
            assert_eq!(client.interfaces.widgets[&0].kind, 3);
        }
        if binding == 4 {
            assert!(client.interfaces.widgets.is_empty());
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("160 original ps.nu cases pass ClientState pointer capture/dispatch with callback-time removal/replacement; no URL launches");
    Ok(())
}
