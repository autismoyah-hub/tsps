use gielinor_client::{client_state::ClientState,disk_cache::DiskCache,interface_state::Interfaces,
    vm::{Host,Object,Stacks},widget::Widget,Result};
fn main()->Result<()> {
    let mut cache=DiskCache::open_remote("work/kotlin-world/.data/cache/js5",64<<20,"127.0.0.1:43617".parse().unwrap())?;
    cache.wait_for_recovery(true);let mut c=ClientState::load(cache)?;
    c.interfaces=Interfaces::default();c.interfaces.install_group(0,vec![Widget::empty(0,11),Widget::empty(1,0)])?;
    c.active_widgets=[Some(0),Some(0)];
    for (op,dot) in [(1131,false),(1131,true),(2131,false),(2131,true)] {
        let mut s=Stacks::default();s.push_int(17)?;s.push_int(op as i32)?;if op>=2000{s.push_int(0)?;}
        c.extended(op,dot,&mut s)?;assert!(s.ints().is_empty());
        assert_eq!(c.interfaces.runtime[&0].remote_mappings.as_ref().unwrap().fonts.borrow()[&17],op as i32);
    }
    for (op,key) in [(1132,None),(2132,Some("endpoint"))] {
        let mut s=Stacks::default();s.push_int(73)?;if op>=2000{s.push_int(0)?;}
        s.push_object(key.map_or(Object::Null,|v|Object::String(v.into())))?;
        c.extended(op,false,&mut s)?;assert!(s.ints().is_empty());assert!(s.objects().is_empty());
        assert_eq!(c.interfaces.runtime[&0].remote_mappings.as_ref().unwrap().endpoints[&key.map(str::to_owned)],73);
    }
    let mut s=Stacks::default();s.push_int(9)?;s.push_int(10)?;s.push_int(1)?;
    c.extended(2131,false,&mut s)?;assert!(s.ints().is_empty());assert!(c.interfaces.runtime[&1].remote_mappings.is_none());
    println!("Actual-cache ClientState1131/1132 and2131/2132 retain font/endpoint/null mappings, both selection slots and non-type11 no-op pass");Ok(())
}
