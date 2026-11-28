use crate::{buffer::Reader,disk_cache::DiskCache,interface_state::Interfaces,item::Definitions,widget::Widget,widget_layout::Geometry,Result};
#[test]
fn position_packets_match_original_geometry_updates()->Result<()> {
 let mut input=Reader::new(include_bytes!("../tests/fixtures/widget-position-packet-jvm.bin"));
 let mut cache=DiskCache::open("tests/fixtures/event-script-cache",1<<20)?;
 let items=Definitions{items:Default::default()};
 for case in 0..input.u32()? {
  let kind=input.u32()? as u8;let parent=input.u32()? as i32;let mode=input.u32()? as i8;let x=input.u32()? as i32;let y=input.u32()? as i32;
  let expected=[input.u32()? as i32,input.u32()? as i32,input.u32()? as i32,input.u32()? as i32];
  let mut state=Interfaces::default();state.geometry_canvas.set([765,503]);
  state.install_group(0,(0..3).map(|id|{let mut w=Widget::empty(id,if id==0 {0}else if id==1 {kind}else{4});w.parent=if id==2{Some(1)}else if id==1&&parent>=0{Some(parent as u32)}else{None};w.raw_size=[if id==0{300}else if id==1{40}else{10},0];w.size_alignment=[i8::from(id!=0),0];if id==1{w.raw_position=[10,20];w.position_alignment=[mode,mode];}w}).collect())?;
  for (&id,r) in &state.runtime {r.aligned.set(Some(Geometry{position:if id==1{[7,8]}else{[0,0]},size:[if id==0{300}else if id==1{100}else{77},200]}));}
  let payload=vec![y.wrapping_add(128) as u8,(y>>8)as u8,0,1,0,0,x.wrapping_add(128)as u8,(x>>8)as u8];
  let update=crate::interface_packet::decode(&crate::protocol::Packet{opcode:102,payload})?.unwrap();
  state.apply(&mut cache,&items,&mut Default::default(),update)?;
  let geometry=state.runtime[&1].aligned.get().expect("stored geometry");
  let actual=[geometry.position[0],geometry.position[1],geometry.size[0],state.runtime[&2].aligned.get().unwrap().size[0]];
  assert_eq!(actual,expected,"case{case},kind{kind},parent{parent},mode{mode},position{x},{y}");
 }
 assert_eq!(input.remaining(),0);Ok(())
}
