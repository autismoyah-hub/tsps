use gielinor_client::actor_menu;
use gielinor_client::{buffer::Reader,client_state::ClientState,disk_cache::DiskCache,npc, npc_info,player_info,world_entity,Result};
fn bits(values:&[(u32,u8)])->Vec<u8>{let mut out=Vec::new();let mut bit=0;for &(v,n) in values {for i in (0..n).rev(){if bit%8==0 {out.push(0);}out[bit/8]|=(((v>>i)&1)as u8)<<(7-bit%8);bit+=1;}}out}
fn main()->Result<()> {
 let mut client=ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5",64<<20)?)?;
 let mut initial=vec![((50<<14)|50,30)];initial.extend(std::iter::repeat_n((0,18),2046));
 let (template,_) = player_info::State::initialize(&bits(&initial),1)?;
 let player=template.players[1].as_ref().unwrap().clone();
 client.npcs.insert(1,npc::Definition::decode(1,&[0])?);
 let mut npcs=npc_info::State::default();npcs.decode(&bits(&[(0,8),(1,16),(0,1),(0,1),(0,6),(0,3),(0,6),(1,1),(1,14)]),false,[50,50],&client.npcs)?;
 let npc=npcs.npcs[&1].clone();
 let mut r=Reader::new(include_bytes!("../tests/fixtures/overlapping-actor-batch-jvm.bin"));
 for case in 0..r.u32()? {
  let picks=(0..r.u32()?).map(|_|{let kind=r.u32()?;let index=r.u32()?;Ok(((if kind==1 {0x4000000000000000}else{0x2000000000000000})|index as u64,0))}).collect::<Result<Vec<_>>>()?;
  let combat=r.u32()? as i32;client.npc_state=Default::default();let mut expected_actors=Vec::new();
  for _ in 0..3 {let index=r.u32()? as u16;let size=r.u32()? as u8;let fine=[r.u32()? as i32,r.u32()? as i32];let mut n=npc.clone();n.id=index;n.path.position=fine;client.npc_state.indices.push(index);client.npc_state.npcs.insert(index,n);let mut def=npc::Definition::decode(index as u32,&[0])?;def.size=size;client.npcs.insert(index as u32,def);expected_actors.push((true,index,size,fine));}
  let mut state=template.clone();state.high.clear();state.players[1]=None;let mut child=std::collections::BTreeMap::new();
  for _ in 0..3 {let index=r.u32()? as u16;let fine=[r.u32()? as i32,r.u32()? as i32];let mut p=player.clone();p.path.position=fine;state.players[index as usize]=Some(p.clone());state.high.push(index);child.insert(index,p);expected_actors.push((false,index,1,fine));}
  state.views.insert(7,player_info::View{base:[0,0],dimensions:[104,104],players:child});client.player_state=Some(state);
  let mut entity=world_entity::Entity::default();entity.npcs=client.npc_state.clone();client.world_entities.entry(0).or_default().entities.insert(7,entity);
  let expected=(0..r.u32()?).map(|_|Ok((r.u32()?,r.u32()?))).collect::<Result<Vec<_>>>()?;
  for view in [0,7] {
   let actors=actor_menu::actors(&client,view);assert_eq!(actors.iter().map(|a|(a.npc,a.index,a.size,a.fine)).collect::<Vec<_>>(),expected_actors,"state order case {case} view {view}");
   let views=std::collections::HashMap::from([(view,actors)]);let picks=picks.iter().map(|&(tag,_)|(tag,view)).collect::<Vec<_>>();let mut actual=vec![(23,0)];
   for (tag,owner) in actor_menu::expand(&picks,&views,combat){assert_eq!(owner,view);let index=tag as u16 as u32;if tag&0x4000000000000000!=0 {actual.extend([(9,index),(1003,index)]);}else{actual.push((44,index));}}
   assert_eq!(actual,expected,"case {case} view {view}");
  }
 }
 assert_eq!(r.remaining(),0);println!("512 actual ClientState overlap snapshots with shuffled NPC/player lists in root and child views");Ok(())
}
