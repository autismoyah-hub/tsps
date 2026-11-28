use gielinor_client::{client_state::ClientState,disk_cache::DiskCache,widget::Widget,buffer::Reader,context_menu::Action,Result};
fn main()->Result<()> {
 let mut client=ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5",64<<20)?)?;client.canvas=[200,100];
 let mut r=Reader::new(include_bytes!("../tests/fixtures/interface-menu-priority-jvm.bin"));
 for case in 0..r.u32()? {
  let mut defs=Vec::new();
  for id in 1..=r.u32()? {
   let operation=r.u32()?;let priority=r.u32()? as i32;let mut w=Widget::empty(id,4);
   w.raw_size=[50,50];w.flags=1<<operation;w.operation_priority=priority;w.actions=vec![String::new();operation as usize];w.actions[operation as usize-1]=format!("Op{operation}");defs.push(w);
  }
  let expected=(0..r.u32()?).map(|_|Ok((r.u32()?,r.u32()?,r.u32()?))).collect::<Result<Vec<_>>>()?;
  client.interfaces=Default::default();client.interfaces.install_group(0,defs)?;client.interfaces.root=Some(0);
  let actual=client.interface_menu([10,10])?.into_iter().map(|entry|match entry.action {
   Action::Interface{id,operation,..}=>(id,operation as u32,if operation as i32-1>client.interfaces.widgets[&id].operation_priority {1007}else{57}),
   _=>panic!("unexpected action")
  }).collect::<Vec<_>>();
  assert_eq!(actual,expected,"case{case}");
 }
 assert_eq!(r.remaining(),0);println!("Actual ClientState: 512 original multi-widget menus with final global operation priority");Ok(())
}
