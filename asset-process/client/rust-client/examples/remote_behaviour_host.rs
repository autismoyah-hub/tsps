use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_state::Interfaces,
    remote_card::{Parameter, ParameterValue},
    remote_crm::Transport,
    remote_request::Requester,
    remote_url::Context,
    vm::{Host, Object, Stacks},
    widget::Widget,
    Result,
};
fn text(r: &mut Reader<'_>) -> Result<String> {
    let n = r.u16()? as usize;
    Ok(String::from_utf8(r.bytes(n)?.to_vec()).unwrap())
}
fn main() -> Result<()> {
    let mut r = Reader::new(include_bytes!("../tests/fixtures/remote-behaviour-jvm.bin"));
    assert_eq!(r.u32()?, 232);
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    for case in 0..232 {
        let input = text(&mut r)?;
        assert_eq!(r.u8()?, 0);
        for _ in 0..r.u32()? {
            text(&mut r)?;
        }
        let n = r.u32()?;
        let mut expected = Vec::new();
        for _ in 0..n {
            let name = text(&mut r)?;
            let kind = r.u32()?;
            let value = if kind == 0 {
                ParameterValue::Int(r.u32()? as i32)
            } else {
                assert_eq!(r.u8()?, 1);
                ParameterValue::Text(text(&mut r)?)
            };
            expected.push(Parameter { name, value });
        }
        client.interfaces = Interfaces::default();
        let mut widget = Widget::empty(0, 11);
        widget.raw_size = [100, 80];
        client.interfaces.install_group(0, vec![widget])?;
        client.interfaces.root = Some(0);
        client.canvas = [100, 80];
        client.active_widgets = [Some(0); 2];
        client.remote = Context {
            requester: Some(Requester::manual()),
            ..Default::default()
        };
        client.remote.crm.transport = Transport::manual();
        let mut stack = Stacks::default();
        for value in [
            "https://content.invalid/card",
            "https://crm.invalid",
            "card",
        ] {
            stack.push_object(Object::String(value.into()))?;
        }
        client.extended(1151, false, &mut stack)?;
        let fields = input.strip_prefix('{').unwrap().strip_suffix('}').unwrap();
        let response=format!("{{version:2,lastfullsyncat:0,lastcardupdatedat:0,crmcomponents:[{{game:'osrs',removed:false,platform:'desktop',location:'card',{fields}}}]}}");
        client.remote.requester.as_ref().unwrap().queued()[0].complete(Some(response.into_bytes()));
        client.tick_interfaces(false)?;
        let runtime = &client.interfaces.runtime[&0];
        assert_eq!(runtime.remote_status, 2, "case{case}");
        assert_eq!(
            runtime
                .remote_loader
                .as_ref()
                .unwrap()
                .card
                .as_ref()
                .unwrap()
                .parameters,
            expected,
            "case{case}"
        );
        assert_eq!(client.remote.crm.event_kind, 3);
    }
    assert_eq!(r.remaining(), 0);
    println!("232 original behaviour-parameter cases pass actual authenticated ClientState loading and impression dispatch");
    Ok(())
}
