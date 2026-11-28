use gielinor_client::{client_state::ClientState,disk_cache::DiskCache,widget::Widget,buffer::Reader,interface_packet::SubInterface,context_menu::{Entry,Action},Result};
fn root()->Widget {let mut w=Widget::empty(0,0);w.if3=true;w.raw_size=[500,500];w}
fn source()->Widget {let mut w=Widget::empty(65536,4);w.if3=true;w.raw_position=[20,20];w.raw_size=[50,50];w}
fn main()->Result<()> {
 let mut client=ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5",64<<20)?)?;
 let mut r=Reader::new(include_bytes!("../tests/fixtures/widget-drag-identity-jvm.bin"));
 for _ in 0..r.u32()? {
  let case=r.u32()?;client.interfaces=Default::default();client.widget_drag=None;
  client.interfaces.install_group(0,vec![root()])?;client.interfaces.install_group(1,vec![source()])?;
  client.interfaces.root=Some(0);client.interfaces.children.insert(0,SubInterface{parent:0,group:1,kind:0});
  client.interfaces.runtime.get_mut(&65536).unwrap().drag_parent=Some(0);
  let entries=vec![Entry{option:"Test".into(),target:String::new(),action:Action::Cancel,submenu:false,children:vec![]}];
  assert!(client.begin_widget_drag([25,25],&entries)?);
  if case&1!=0 {
   client.interfaces.close(0)?;client.interfaces.install_group(1,vec![source()])?;
   client.interfaces.children.insert(0,SubInterface{parent:0,group:1,kind:0});
   client.interfaces.runtime.get_mut(&65536).unwrap().drag_parent=Some(0);
  }
  if case&2!=0 {
   client.interfaces.children.insert(987,SubInterface{parent:987,group:0,kind:0});client.interfaces.close(987)?;
   client.interfaces.install_group(0,vec![root()])?;
  }
  client.input.buttons=1;client.input.pointer=[45,25];
  for step in 0..2 {
   let active=r.u8()?!=0;let dragging=r.u8()?!=0;let outcome=r.u32()?;let mouse=[r.u32()? as i32,r.u32()? as i32];
   let actual=client.tick_widget_drag()?;
   assert_eq!(client.widget_drag.is_some(),active,"active case{case} step{step}");
   if let Some(drag)=&client.widget_drag {assert_eq!(drag.motion.dragging,dragging,"dragging case{case} step{step}");}
   assert_eq!(actual.is_some(),outcome!=0,"release case{case} step{step}");
   if let Some((point,_))=actual {assert_eq!(point,mouse,"point case{case} step{step}");}
  }
 }
 assert_eq!(r.remaining(),0);println!("Actual ClientState: source/parent group reload preserves original drag identity and cancellation timing");Ok(())
}
