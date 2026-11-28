use gielinor_client::{buffer::Reader,client_state::ClientState,context_menu::{Action,Entry},disk_cache::DiskCache,widget::Widget,Result};
fn entry(action:Action)->Entry<Action>{Entry{option:String::new(),target:String::new(),action,submenu:false,children:Vec::new()}}
fn main()->Result<()> {
 let mut client=ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5",64<<20)?)?;client.canvas=[100,100];
 let mut r=Reader::new(include_bytes!("../tests/fixtures/player-menu-batch-jvm.bin"));
 for case in 0..r.u32()? {
  let initial=r.u32()?;
  let mut expected=(0..r.u32()?).map(|_|r.u32()).collect::<Result<Vec<_>>>()?;let target=r.string()?;
  gielinor_client::menu_sort::prioritize(&mut expected,|v|*v as i32);
  let count=initial-2; // Cancel and Walk here account for the other two entries.
  let mut widgets=Vec::new();
  for id in 0..count {let mut w=Widget::empty(id,4);w.button=1;w.button_text="Seed".into();w.raw_size=[50,50];widgets.push(w);}
  let mut viewport=Widget::empty(count,4);viewport.content=1337;viewport.raw_size=[50,50];widgets.push(viewport);
  client.interfaces=Default::default();client.interfaces.install_group(0,widgets)?;client.interfaces.root=Some(0);
  let view=if case%2==0 {0}else{7};let mut world=vec![entry(Action::Walk([50,50]).in_view(view))];let mut low=Vec::new();
  for index in 1..=2 {for operation in (1..=8).rev() {let action=Action::Player{index,operation}.in_view(view);if operation%2==1 {low.push(action);}world.push(entry(action));}}
  let actors=vec![gielinor_client::scene_menu::ActorContribution{entries:1..9,walk_target:Some("<col=ffffff>FixtureA  (skill-1)".into())},gielinor_client::scene_menu::ActorContribution{entries:9..17,walk_target:Some("<col=ffffff>FixtureB  (skill-1)".into())}];
  let actual=client.interface_menu_with_world_groups([10,10],[10,10],&world,&low,&actors)?;
  assert_eq!(actual.iter().find(|e|matches!(e.action.scene().1,Action::Walk(_))).unwrap().target,target);
  let codes=actual.iter().filter_map(|e|if let Action::Player{operation,..}=e.action.scene().1 {Some(43+operation as u32+if operation%2==1 {2000}else{0})}else{None}).collect::<Vec<_>>();
  assert_eq!(codes,expected,"case {case} initial {initial}");
 }
 assert_eq!(r.remaining(),0);println!("128 actual ClientState actor admission cases after interface contributions");Ok(())
}
