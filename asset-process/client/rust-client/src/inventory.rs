//! Revision 239 item-container packets and client-side inventory state.
use crate::{buffer::{Reader,Transform},protocol::Packet,Error,Result};
use std::collections::HashMap;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Item {pub id:i32,pub count:i32}
impl Default for Item {fn default()->Self{Self{id:-1,count:0}}}
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Update {
    Full{widget:i32,container:u16,items:Vec<Item>},
    Partial{widget:i32,container:u16,items:Vec<(u16,Item)>},
    Stop(u16),
}
pub fn decode(packet:&Packet)->Result<Option<Update>> {
    let mut r=Reader::new(&packet.payload);
    let update=match packet.opcode {
        22=>{let widget=r.u32()? as i32;let container=r.u16()?;let n=r.u16()?;let mut items=Vec::with_capacity(n as usize);for _ in 0..n {let mut count=r.transformed(Transform::Sub)? as i32;if count==255{count=r.u32_middle()? as i32;}items.push(Item{id:r.u16_le()? as i32-1,count});}Update::Full{widget,container,items}},
        18=>{let widget=r.u32()? as i32;let container=r.u16()?;let mut items=Vec::new();while r.remaining()!=0 {let slot=r.unsigned_smart()?;let id=r.u16()? as i32-1;let count=if id==-1{0}else{let v=r.u8()?;if v==255{r.u32()? as i32}else{v as i32}};items.push((slot,Item{id,count}));}Update::Partial{widget,container,items}},
        45=>Update::Stop(r.u16_add(false)?),
        _=>return Ok(None),
    };if r.remaining()!=0{return Err(Error::Invalid("inventory packet trailing bytes"));}Ok(Some(update))
}
#[derive(Default)]
pub struct Inventories {
    pub containers:HashMap<u32,Vec<Item>>,
    pub widgets:HashMap<u32,Vec<Item>>,
    pub changed:Vec<u16>,
}
impl Inventories {
    pub fn apply(&mut self,update:Update){
        match update {
            Update::Stop(id)=>{self.containers.remove(&(id as u32));self.changed.push(id);},
            Update::Full{widget,container,items}=>{let id=container as u32+if widget < -70000{32768}else{0};if widget>=0{self.widgets.insert(widget as u32,items.clone());}self.containers.insert(id,items);self.changed.push(container&32767);},
            Update::Partial{widget,container,items}=>{let id=container as u32+if widget < -70000{32768}else{0};let dest=self.containers.entry(id).or_default();for &(slot,item) in &items{let slot=slot as usize;if dest.len()<=slot{dest.resize(slot+1,Item::default());}dest[slot]=item;}if widget>=0{let dest=self.widgets.entry(widget as u32).or_default();for (slot,item) in items{let slot=slot as usize;if dest.len()<=slot{dest.resize(slot+1,Item::default());}dest[slot]=item;}}self.changed.push(container&32767);},
        }
        if self.changed.len()>32{self.changed.drain(..self.changed.len()-32);}
    }
    pub fn item(&self,id:u32,slot:i32)->Item{self.containers.get(&id).and_then(|v|usize::try_from(slot).ok().and_then(|i|v.get(i))).copied().unwrap_or_default()}
    pub fn total(&self,id:u32,item:i32)->i32{if item<0{return 0;}self.containers.get(&id).map_or(0,|v|v.iter().filter(|i|i.id==item).fold(0i32,|a,i|a.wrapping_add(i.count)))}
    pub fn clear_widget(&mut self,id:u32){if let Some(items)=self.widgets.get_mut(&id){items.fill(Item::default());}}
}
