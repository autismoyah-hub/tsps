use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_state::{Interfaces, RemoteMappings},
    remote_crm::{Session, Transport},
    remote_request::Requester,
    remote_url::Context,
    remote_widget::Loader,
    vm::Host,
    vm::{Object, Stacks},
    widget::Widget,
    Result,
};
fn text(r: &mut Reader<'_>) -> Result<String> {
    let n = r.u16()? as usize;
    Ok(String::from_utf8(r.bytes(n)?.to_vec()).unwrap())
}
fn optional(r: &mut Reader<'_>) -> Result<Option<String>> {
    if r.u8()? != 0 {
        Ok(Some(text(r)?))
    } else {
        Ok(None)
    }
}
fn main() -> Result<()> {
    let mut r = Reader::new(include_bytes!("../tests/fixtures/remote-auth-jvm.bin"));
    assert_eq!(r.u32()?, 1408);
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    for case in 0..1408 {
        let op = r.u32()? as u16;
        let dot = r.u8()? != 0;
        let kind = r.u32()? as u8;
        let seeded = r.u8()? != 0;
        let available = r.u8()? != 0;
        let user_id = r.u64()? as i64;
        let user_hash = r.u64()? as i64;
        let inputs = [optional(&mut r)?, optional(&mut r)?, optional(&mut r)?];
        let failed = r.u8()? != 0;
        let present = r.u8()? != 0;
        let status = r.u32()? as i32;
        let version = r.u32()? as i32;
        let raw = optional(&mut r)?;
        let location = optional(&mut r)?;
        let base = optional(&mut r)?;
        let attached = r.u8()? != 0;
        let mapped = r.u8()? != 0;
        let font_count = r.u32()? as usize;
        let n = r.u32()?;
        let mut urls = Vec::new();
        for _ in 0..n {
            urls.push(text(&mut r)?)
        }
        let n = r.u32()?;
        let mut rest = Vec::new();
        for _ in 0..n {
            rest.push(text(&mut r)?)
        }
        let crm_base = optional(&mut r)?;
        let device = optional(&mut r)?;
        let hash = optional(&mut r)?;
        let pending = r.u8()? != 0;
        let ints = r.u32()? as usize;
        let objects = r.u32()? as usize;
        client.interfaces = Interfaces::default();
        client
            .interfaces
            .install_group(0, vec![Widget::empty(0, kind)])?;
        if seeded {
            let runtime = client.interfaces.runtime.get_mut(&0).unwrap();
            runtime.remote_loader = Some(Box::new(Loader {
                status: 2,
                version: 7,
                ..Default::default()
            }));
            let maps = RemoteMappings::default();
            maps.fonts.borrow_mut().insert(17, 29);
            runtime.remote_mappings = Some(Box::new(maps));
        }
        client.remote = Context {
            user_id,
            user_hash,
            requester: available.then(Requester::manual),
            crm: Session {
                base: Some("previous/".into()),
                device: Some("old-device".into()),
                user_hash: Some("old-hash".into()),
                transport: Transport::manual(),
                ..Default::default()
            },
            ..Default::default()
        };
        let mut stack = Stacks::default();
        for input in inputs {
            stack.push_object(input.map_or(Object::Null, Object::String))?;
        }
        if op >= 2000 {
            stack.push_int(0)?;
        }
        client.active_widgets = [Some(0); 2];
        let result = client.extended(op, dot, &mut stack);
        assert_eq!(result.is_err(), failed, "case{case} op{op}");
        let runtime = &client.interfaces.runtime[&0];
        let loader = runtime.remote_loader.as_ref();
        assert_eq!(loader.is_some(), present, "case{case}");
        assert_eq!(loader.map_or(-99, |l| l.status), status, "case{case}");
        assert_eq!(loader.map_or(-99, |l| l.version), version);
        assert_eq!(loader.and_then(|l| l.raw_url.clone()), raw, "case{case}");
        assert_eq!(loader.and_then(|l| l.location.clone()), location);
        assert_eq!(loader.and_then(|l| l.crm_base.clone()), base);
        assert_eq!(loader.is_some_and(|l| l.crm_attached), attached);
        assert_eq!(runtime.remote_mappings.is_some(), mapped);
        assert_eq!(
            runtime
                .remote_mappings
                .as_ref()
                .map_or(0, |m| m.fonts.borrow().len()),
            font_count
        );
        assert_eq!(
            client.remote.requester.as_ref().map_or_else(Vec::new, |r| r
                .queued()
                .iter()
                .map(|r| r.url.clone())
                .collect()),
            urls,
            "case{case}"
        );
        assert_eq!(
            client
                .remote
                .crm
                .transport
                .queued()
                .iter()
                .map(|r| r.url.clone())
                .collect::<Vec<_>>(),
            rest,
            "case{case}"
        );
        assert_eq!(client.remote.crm.base, crm_base, "case{case}");
        assert_eq!(client.remote.crm.device, device);
        assert_eq!(client.remote.crm.user_hash, hash);
        assert_eq!(client.remote.crm.pending.is_some(), pending);
        assert_eq!(stack.ints().len(), ints);
        assert_eq!(stack.objects().len(), objects);
    }
    assert_eq!(r.remaining(), 0);
    client.remote.user_id = 17;
    client.remote.user_hash = 31;
    client.remote.crm = Session {
        transport: Transport::manual(),
        ..Default::default()
    };
    client.remote.crm.start(
        Some("https://crm.invalid".into()),
        Some("fixture-device".into()),
        Some("31".into()),
        10,
    )?;
    let pending = client.remote.crm.pending.clone().unwrap();
    client.prepare_login()?;
    assert_eq!(client.remote.user_hash, 31);
    pending.complete(gielinor_client::world_http::Response {
        status: 200,
        body: b"session".to_vec(),
    });
    assert_eq!(
        client
            .remote
            .crm
            .pending
            .as_ref()
            .unwrap()
            .result()
            .unwrap()
            .status,
        200
    );
    println!("1408 authenticated remote-widget cases pass through actual ClientState dispatch without network submission");
    Ok(())
}
