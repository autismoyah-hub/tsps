use gielinor_client::{client_state::ClientState,disk_cache::DiskCache,vm::{Host,Stacks},widget::Widget,Result};
fn main()->Result<()>{
 let mut c=ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5",64<<20)?)?;
 assert!(c.world_map.renderer.is_none());
 let mut s=Stacks::default();c.extended(6605,false,&mut s)?;assert_eq!(s.ints(),&[1]);assert!(c.world_map.renderer.is_none());
 for id in [0,1,25,0] {let mut s=Stacks::default();s.push_int(id)?;c.extended(6602,false,&mut s)?;c.extended(6605,false,&mut s)?;assert_eq!(s.ints(),&[1]);assert_eq!(c.world_map.loading.area,Some(id));}
 c.interfaces.install_group(0,vec![Widget::empty(0,0)])?;
 let child=c.interfaces.create_child(0,0,5,false)?;c.active_widgets=[Some(child),Some(0)];
 let mut s=Stacks::default();s.push_int(239)?;c.extended(3139,false,&mut s)?;assert_eq!(c.interfaces.shortcut_scope.mode,1);c.extended(3138,false,&mut s)?;assert_eq!(c.interfaces.shortcut_scope.mode,0);
 c.extended(3140,false,&mut s)?;assert_eq!(c.interfaces.shortcut_scope.target,0);assert_eq!(c.interfaces.shortcut_scope.mode,3);assert_eq!(s.ints(),&[239]);
 let area=c.world_map.current;c.prepare_login()?;assert_eq!(c.world_map.current,area);let mut s=Stacks::default();c.extended(6605,false,&mut s)?;assert_eq!(s.ints(),&[1]);
 println!("Map archive readiness before rendering, area switches/login preservation, scope3138/3139 and dynamic logical-widget3140 passed");Ok(())
}
