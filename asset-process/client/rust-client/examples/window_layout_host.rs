use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, interface_state::Interfaces,
    vm::{Host, Stacks, Value}, widget::Widget, Result};
fn mode(client: &mut ClientState, value: i32) -> Result<()> {
    let mut stack=Stacks::default(); stack.push_int(value)?;
    client.extended(5307, false, &mut stack)?; assert!(stack.ints().is_empty()); Ok(())
}
fn packet(client: &mut ClientState, mode: u8, canvas: [u16;2]) {
    let p=client.outbound.pop_front().expect("window status");
    let mut expected=vec![mode];for value in canvas { expected.extend(value.to_be_bytes()); }
    assert_eq!(p.opcode,10);assert_eq!(p.payload,expected);assert!(client.outbound.is_empty());
}
fn main()->Result<()> {
    let mut cache=DiskCache::open_remote("work/kotlin-world/.data/cache/js5",64<<20,"127.0.0.1:43617".parse().unwrap())?;
    cache.wait_for_recovery(true);let mut c=ClientState::load(cache)?;
    c.interfaces=Interfaces::default();let mut root=Widget::empty(0,0);
    root.if3=true;root.raw_size=[10,20];root.size_alignment=[1,1];
    c.interfaces.install_group(0,vec![root])?;c.interfaces.root=Some(0);
    c.interfaces.runtime.get_mut(&0).unwrap().listeners.insert(1427,(vec![Value::Int(42)],None));
    c.native_layout([765,503],0,30)?;
    assert_eq!(c.interfaces.layout(c.canvas)?.widgets[&0].geometry.size,[755,483]);
    c.interfaces.resize_events.borrow_mut().clear();
    mode(&mut c,2)?;packet(&mut c,2,[765,503]);
    let fit=c.native_layout([1024,768],100,30)?;
    assert_eq!(fit.origin,[0,0]);assert_eq!(c.canvas,[1024,768]);
    assert_eq!(c.interfaces.layout(c.canvas)?.widgets[&0].geometry.size,[1014,748]);
    assert_eq!(c.interfaces.resize_events.borrow().len(),1);
    assert_eq!(c.interfaces.resize_events.borrow()[0].event.context.resize_depth,0);
    c.interfaces.resize_events.borrow_mut().clear();
    c.native_layout([1024,768],600,30)?;assert!(c.outbound.is_empty());
    c.native_layout([1024,768],601,30)?;packet(&mut c,2,[1024,768]);
    c.native_layout([960,640],700,30)?;
    let deadline=c.window.deadline;mode(&mut c,3)?;assert_eq!(c.window.deadline,deadline);assert!(c.outbound.is_empty());
    mode(&mut c,2)?;packet(&mut c,2,[960,640]);assert_eq!(c.window.deadline,None);
    mode(&mut c,1)?;packet(&mut c,1,[960,640]);
    let fit=c.native_layout([1024,768],1000,30)?;
    assert_eq!(c.canvas,[765,503]);assert_eq!(fit.origin,[129,0]);
    assert_eq!(fit.to_logical(fit.to_physical([700,450])),[700,450]);
    assert_eq!(c.interfaces.layout(c.canvas)?.widgets[&0].geometry.size,[755,483]);
    c.native_layout([1024,768],1500,30)?;assert!(c.outbound.is_empty());
    c.native_layout([1024,768],1501,30)?;packet(&mut c,1,[765,503]);
    println!("Cache-backed ClientState5307: mode packets before layout, root callbacks/depth, invalid-mode preservation, strict500ms reporting and coordinate mapping pass");Ok(())
}
