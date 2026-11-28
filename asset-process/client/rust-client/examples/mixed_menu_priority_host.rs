use gielinor_client::{buffer::Reader,client_state::ClientState,context_menu::{Action,Entry},disk_cache::DiskCache,widget::Widget,Result};
fn main()->Result<()> {
 let mut client=ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5",64<<20)?)?;
 client.canvas=[200,100];
 let mut r=Reader::new(include_bytes!("../tests/fixtures/mixed-menu-priority-jvm.bin"));
 for case in 0..r.u32()? {
  let count=r.u32()?;let position=r.u32()?;let n=r.u32()?;let mut world=Vec::new();let mut low=Vec::new();let mut codes=Vec::new();
  let view=if case%2==0 {0}else{7};
  for k in 0..n {
   let code=r.u32()?;codes.push(code);
   let action=match code {
    23=>Action::Walk([k as u16,0]),1003=>Action::ExamineNpc(k as u16),
    2001|9=>Action::Npc{index:k as u16,operation:1,sub:0},
    1001=>Action::Object{id:k as u16,tile:[0,0],operation:5,sub:0},
    1013=>Action::WorldEntity{id:k as u16,operation:6},
    44=>Action::Player{index:k as u16,operation:1},_=>panic!("unknown code")
   }.in_view(view);
   if code==2001 {low.push(action);}
   world.push(Entry{option:format!("World{k}"),target:String::new(),action,submenu:false,children:Vec::new()});
  }
  let mut defs=Vec::new();
  for id in 1..=count {
   let operation=r.u32()?;let priority=r.u32()? as i32;
   let mut w=Widget::empty(id*2,4);w.raw_size=[50,50];w.flags=1<<operation;w.operation_priority=priority;
   w.actions=vec![String::new();operation as usize];w.actions[operation as usize-1]=format!("Op{operation}");defs.push(w);
  }
  let mut viewport=Widget::empty(position*2+1,4);viewport.content=1337;viewport.raw_size=[50,50];defs.push(viewport);
  let expected=(0..r.u32()?).map(|_|Ok((r.u32()?,r.u32()?,r.u32()?))).collect::<Result<Vec<_>>>()?;
  client.interfaces=Default::default();client.interfaces.install_group(0,defs)?;client.interfaces.root=Some(0);
  let actual=client.interface_menu_with_world([10,10],[10,10],&world,&low)?.into_iter().map(|entry| {
   if let Action::Interface{id,operation,..}=entry.action {
    (id/2,operation as u32,if operation as i32-1>client.interfaces.widgets[&id].operation_priority {1007}else{57})
   } else {
    let k=world.iter().position(|e|e.action==entry.action).expect("scene action/view preserved");
    (100+k as u32,k as u32,codes[k])
   }
  }).collect::<Vec<_>>();
  assert_eq!(actual,expected,"case {case} view {view}");
 }
 assert_eq!(r.remaining(),0);println!("512 actual ClientState mixed scene/UI priority menus, alternating root/child scene identities");Ok(())
}
