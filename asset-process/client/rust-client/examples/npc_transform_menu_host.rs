use gielinor_client::actor_menu;
use gielinor_client::{buffer::Reader,client_state::ClientState,disk_cache::DiskCache,npc::Definition,object::Operation,Result};
fn main()->Result<()> {
 let mut client=ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5",64<<20)?)?;
 client.npcs.insert(123,Definition::decode(123,&[106,255,255,0,0,1,0,200,0,201,0])?);
 for id in 200..202 {let mut def=Definition::decode(id,&[0])?;def.name=format!("Variant{id}");def.operations=vec![Operation{index:0,sub:None,condition:None,text:format!("Talk{id}")}];client.npcs.insert(id,def);}
 let mut r=Reader::new(include_bytes!("../tests/fixtures/npc-transform-menu-jvm.bin"));
 for case in 0..r.u32()? {
  let value=r.u32()? as i32;client.scripts.variables.set_player(0,value);
  let expected=(0..r.u32()?).map(|_|Ok((r.u32()?,r.string()?,r.string()?))).collect::<Result<Vec<_>>>()?;
  let mut actual=Vec::new();
  if let Some(id)=actor_menu::npc_definition(&mut client,123)? {
   let def=&client.npcs[&id];let target=gielinor_client::menu_settings::npc_target(&def.name,def.combat,-1,0);
   let label=gielinor_client::entity_ops::label(&def.operations,0,None,&client.scripts.variables,&mut client.scripts.source)?.unwrap();
   actual.push((9,label.to_owned(),target.clone()));actual.push((1003,"Examine".into(),target));
  }
  assert_eq!(actual,expected,"case {case} varp {value}");
 }
 assert_eq!(r.remaining(),0);println!("128 actual ClientState NPC transform decisions without rendering");Ok(())
}
