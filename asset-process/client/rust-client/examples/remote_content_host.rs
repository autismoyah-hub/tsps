use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_render::Renderer,
    interface_state::Interfaces,
    raster::Raster,
    remote_crm::Transport,
    remote_request::Requester,
    vm::{Host, Object, Stacks},
    widget::Widget,
    Result,
};
fn opcode(client: &mut ClientState, op: u16, ints: &[i32], strings: &[&str]) -> Result<()> {
    let mut stack = Stacks::default();
    for value in ints {
        stack.push_int(*value)?;
    }
    for value in strings {
        stack.push_object(Object::String((*value).into()))?;
    }
    client.extended(op, false, &mut stack)?;
    Ok(())
}
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let font = cache
        .reference(13)?
        .groups
        .iter()
        .find(|g| g.name_hash == Some(gielinor_client::disk_cache::name_hash("b12_full")))
        .unwrap()
        .id;
    let mut renderer = Renderer::load(&mut cache, 0.8)?;
    let mut client = ClientState::load(cache)?;
    client.interfaces = Interfaces::default();
    let mut parent = Widget::empty(0, 11);
    parent.raw_size = [120, 80];
    client.interfaces.install_group(0, vec![parent])?;
    client.interfaces.root = Some(0);
    client.active_widgets = [Some(0); 2];
    client.canvas = [120, 80];
    client.remote.user_id = 17;
    client.remote.user_hash = 31;
    client.remote.requester = Some(Requester::manual());
    client.remote.crm.transport = Transport::manual();
    opcode(&mut client, 1131, &[17, font as i32], &[])?;
    opcode(
        &mut client,
        1151,
        &[],
        &[
            "https://content.invalid/%userid%/%userhash%/%deviceid%",
            "https://crm.invalid",
            "card",
        ],
    )?;
    let root=br#"{"version":2,"lastfullsyncat":0,"lastcardupdatedat":0,"crmcomponents":[{"game":"osrs","removed":false,"platform":"desktop","location":"card","image":{"src":"https://content.invalid/image","placement":[0,0,0.5,1]},"label":{"text":"Ready","font":17,"placement":[0.5,0,0.5,1]},"behaviour":{"endpoint":"/go"}}]}"#;
    client.remote.requester.as_ref().unwrap().queued()[0].complete(Some(root.to_vec()));
    client.tick_interfaces(false)?;
    assert_eq!(client.interfaces.runtime[&0].remote_status, 1);
    assert_eq!(client.interfaces.child_array_length(0), 0);
    let pixels = vec![
        0xff112233, 0xff445566, 0xff778899, 0xffabcdef, 0xff123456, 0xfffedcba,
    ];
    let bitmap = Raster::from_pixels(3, 2, pixels.clone())?.bmp()?;
    client.remote.requester.as_ref().unwrap().queued()[1].complete(Some(bitmap));
    client.remote.crm.event(2, Some("before-ready".into()));
    client.tick_interfaces(false)?;
    assert_eq!(client.interfaces.runtime[&0].remote_status, 2);
    assert_eq!(client.interfaces.child_array_length(0), 2);
    assert_eq!(client.remote.crm.event_kind, 3);
    assert_eq!(client.remote.crm.event_id, None);
    assert!(client.interfaces.widgets[&0].no_click_through);
    let image = client.interfaces.script_child(0, 0).unwrap();
    let label = client.interfaces.script_child(0, 1).unwrap();
    assert_eq!(client.interfaces.widgets[&image].raw_size, [60, 80]);
    assert_eq!(client.interfaces.widgets[&label].font, font as i32);
    let layout = client.interfaces.layout(client.canvas)?;
    assert_eq!(layout.order, vec![0, image, label]);
    let mut raster = Raster::new(120, 80, 120 * 80)?;
    raster.clear(0xffeeeeee);
    let report = renderer.draw(
        &mut client.scripts.source.cache,
        &client.items,
        &client.interfaces,
        &layout,
        &mut raster,
    )?;
    assert!(report.pending.is_empty(), "{report:?}");
    assert_eq!(report.drawn, 2);
    assert_eq!(raster.pixels()[121], pixels[0]);
    assert_eq!(raster.pixels()[41], pixels[2]);
    assert_eq!(raster.pixels()[61 * 120 + 1], pixels[3]);
    assert!(raster
        .pixels()
        .chunks_exact(120)
        .any(|row| row[60..].iter().any(|&p| p != 0xffeeeeee)));
    std::fs::write("work/w06-remote-content-host.bmp", raster.bmp()?)?;
    // An item assigned later by a script must take precedence over the remote image.
    client.active_widgets = [Some(image); 2];
    opcode(&mut client, 1200, &[1279, 1], &[])?;
    let mut with_remote = Raster::new(120, 80, 120 * 80)?;
    with_remote.clear(0xffeeeeee);
    renderer.draw(
        &mut client.scripts.source.cache,
        &client.items,
        &client.interfaces,
        &layout,
        &mut with_remote,
    )?;
    let request = client
        .interfaces
        .runtime
        .get_mut(&image)
        .unwrap()
        .remote_image
        .take();
    let mut ordinary_item = Raster::new(120, 80, 120 * 80)?;
    ordinary_item.clear(0xffeeeeee);
    renderer.draw(
        &mut client.scripts.source.cache,
        &client.items,
        &client.interfaces,
        &layout,
        &mut ordinary_item,
    )?;
    assert_eq!(
        with_remote.pixels(),
        ordinary_item.pixels(),
        "remote URL incorrectly overrides item icon"
    );
    assert_ne!(with_remote.pixels()[121], pixels[0]);
    client
        .interfaces
        .runtime
        .get_mut(&image)
        .unwrap()
        .remote_image = request;
    client.tick_interfaces(false)?;
    assert_eq!(client.interfaces.script_child(0, 0), Some(image));
    assert_eq!(client.interfaces.script_child(0, 1), Some(label));
    println!("Actual-cache authenticated dispatch, delayed image readiness, generated image/label children, native image decoding and interface raster checks passed");
    Ok(())
}
