use gielinor_client::{buffer::Reader,client_state::ClientState,disk_cache::DiskCache,widget::Widget,Result};
fn main()->Result<()> {
 let mut client=ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5",64<<20)?)?;client.canvas=[200,100];
 let mut r=Reader::new(include_bytes!("../tests/fixtures/minimap-menu-order-jvm.bin"));
 for case in 0..r.u32()? {
  let pointer=[r.u32()? as i32,20];let press=[r.u32()? as i32,20];let pressed=r.u8()?!=0;
  let mut defs=Vec::new();
  for id in 0..4 {
   let mut w=Widget::empty(id,4);w.raw_position=[10+id as i32*10,10];w.raw_size=[50,20];
   w.content=if id==1 {1338}else{0};w.button=1;w.button_text=format!("Widget{id}");w.if3=r.u8()?!=0;w.hidden=r.u8()?!=0;w.no_click_through=r.u8()?!=0;defs.push(w);
  }
  let expected=(0..r.u32()?).map(|_|r.u32()).collect::<Result<Vec<_>>>()?;
  client.interfaces=Default::default();client.interfaces.install_group(0,defs)?;client.interfaces.root=Some(0);client.input.pointer=pointer;
  let expected=expected.into_iter().filter(|id|*id!=100).collect::<Vec<_>>();
  let point=if pressed {press}else{pointer};let actual=client.interface_menu_at(point,pointer)?.into_iter().map(|e|match e.action {gielinor_client::context_menu::Action::Button{id,..}=>id,_=>panic!("unexpected action")}).collect::<Vec<_>>();
  assert_eq!(actual,expected,"case {case} pointer{pointer:?} press{press:?} pressed{pressed}");
 }
 assert_eq!(r.remaining(),0);println!("Actual ClientState: 1024 minimap/interface traces, interface contribution only");Ok(())
}
