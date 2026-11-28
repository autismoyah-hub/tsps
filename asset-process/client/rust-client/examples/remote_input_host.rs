use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_state::Interfaces,
    remote_crm::Transport,
    remote_request::Requester,
    vm::{Host, Object, Stacks},
    widget::Widget,
    Result,
};
fn op(c: &mut ClientState, code: u16, ints: &[i32], strings: &[&str]) -> Result<()> {
    let mut s = Stacks::default();
    for v in ints {
        s.push_int(*v)?;
    }
    for v in strings {
        s.push_object(Object::String((*v).into()))?;
    }
    c.extended(code, false, &mut s)?;
    Ok(())
}
fn optional(r: &mut Reader<'_>) -> Result<()> {
    if r.u8()? != 0 {
        let n = r.u16()?;
        r.bytes(n as usize)?;
    }
    Ok(())
}
fn expected() -> Result<Vec<u8>> {
    let mut r = Reader::new(include_bytes!("../tests/fixtures/remote-click-jvm.bin"));
    let n = r.u32()?;
    let mut result = None;
    for _ in 0..n {
        let kind = r.u32()?;
        let loader = r.u32()?;
        let attached = r.u8()?;
        let mask = r.u8()?;
        let mapped = r.u8()?;
        let params = r.u32()?;
        let id = r.u32()?;
        let child = r.u32()?;
        let failed = r.u8()?;
        r.u32()?;
        optional(&mut r)?;
        if r.u8()? != 0 {
            let opcode = r.u32()?;
            let n = r.u32()?;
            let bytes = r.bytes(n as usize)?;
            if (kind, loader, attached, mask, mapped, params, id, child)
                == (11, 2, 1, 1, 1, 3, 0, u32::MAX)
            {
                assert_eq!(failed, 0);
                assert_eq!(opcode, 91);
                result = Some(bytes.to_vec());
            }
        }
    }
    Ok(result.unwrap())
}
fn main() -> Result<()> {
    let expected = expected()?;
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut c = ClientState::load(cache)?;
    c.interfaces = Interfaces::default();
    let mut w = Widget::empty(0, 11);
    w.raw_size = [100, 80];
    w.flags = 1 << 22;
    c.interfaces.install_group(0, vec![w])?;
    c.interfaces.root = Some(0);
    c.canvas = [100, 80];
    c.active_widgets = [Some(0); 2];
    c.remote.requester = Some(Requester::manual());
    c.remote.crm.transport = Transport::manual();
    op(&mut c, 1132, &[0x87654321u32 as i32], &["/go"])?;
    op(
        &mut c,
        1151,
        &[],
        &[
            "https://content.invalid/card",
            "https://crm.invalid",
            "card",
        ],
    )?;
    c.remote.requester.as_ref().unwrap().queued()[0].complete(Some(br#"{version:2,lastfullsyncat:0,lastcardupdatedat:0,crmcomponents:[{game:'osrs',removed:false,platform:'desktop',location:'card',id:'card',behaviour:{endpoint:'/go',p0:305419896,p1:-1,p2:-2147483648}}]}"#.to_vec()));
    c.tick_interfaces(false)?;
    assert_eq!(c.remote.crm.event_kind, 3);
    c.outbound.clear();
    c.input.pointer = [50, 40];
    c.input.press_position = [50, 40];
    c.input.pressed_buttons = 1;
    c.input.buttons = 1;
    c.tick_interfaces(false)?;
    let packets = c
        .outbound
        .iter()
        .filter(|p| p.opcode == 91)
        .collect::<Vec<_>>();
    assert_eq!(packets.len(), 1);
    assert_eq!(packets[0].payload, expected);
    assert_eq!(c.remote.crm.event_kind, 1);
    c.outbound.clear();
    c.input.begin_cycle();
    c.tick_interfaces(false)?;
    assert!(!c.outbound.iter().any(|p| p.opcode == 91));
    c.input.pointer = [100, 80];
    c.input.press_position = [100, 80];
    c.input.pressed_buttons = 1;
    c.remote.crm.event(2, Some("outside".into()));
    c.tick_interfaces(false)?;
    assert_eq!(c.remote.crm.event_kind, 2);
    c.input.pointer = [50, 40];
    c.input.press_position = [50, 40];
    c.interfaces.widgets.get_mut(&0).unwrap().flags = 0;
    c.tick_interfaces(false)?;
    assert_eq!(c.remote.crm.event_kind, 1);
    assert!(!c.outbound.iter().any(|p| p.opcode == 91));
    c.interfaces
        .runtime
        .get_mut(&0)
        .unwrap()
        .remote_mappings
        .as_mut()
        .unwrap()
        .endpoints
        .clear();
    c.tick_interfaces(false)?;
    assert_eq!(c.remote.pending_urls.pop_front().as_deref(), Some("/go"));
    assert!(c.remote.pending_urls.is_empty());
    println!("Actual ClientState authenticated load -> ready impression -> native input -> exact original packet91; held/outside/flag/URL queue cases pass, no URL launch");
    Ok(())
}
