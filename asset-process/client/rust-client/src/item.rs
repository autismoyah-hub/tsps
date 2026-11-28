//! Revision 239 item definitions and certificate, bought-item and placeholder resolution.
use crate::{buffer::Reader, disk_cache::DiskCache, object::{Operation, Param}, Error, Result};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct Definition {
    pub id: u32,
    pub model: i32,
    pub name: String,
    pub examine: String,
    pub zoom: i32,
    pub angles: [i32; 3],
    pub offset: [i32; 2],
    pub stackable: i32,
    pub price: i32,
    pub wear_positions: [i32; 3],
    pub members: bool,
    pub ground_operations: Vec<Operation>,
    pub inventory_operations: [Option<String>; 5],
    pub sub_operations: [Option<Vec<Option<String>>>; 5],
    pub recolor: Vec<(u16, u16)>,
    pub retexture: Vec<(u16, u16)>,
    pub shift_click: i32,
    pub wear_models: [[i32; 3]; 2],
    pub wear_offset: [i32; 2],
    pub head_models: [[i32; 2]; 2],
    pub category: i32,
    pub weight: i32,
    pub note: [i32; 2],
    pub bought: [i32; 2],
    pub placeholder: [i32; 2],
    pub counts: [(i32, i32); 10],
    pub scale: [i32; 3],
    pub ambient: i32,
    pub contrast: i32,
    pub team: i32,
    pub tradeable: bool,
    pub stockmarket: bool,
    pub searchable: bool,
    pub params: HashMap<u32, Param>,
    pub trailing: Vec<u8>,
}
impl Definition {
    pub fn decode(id: u32, bytes: &[u8]) -> Result<Self> { Self::decode_mode(id, bytes, false) }
    pub fn decode_local(id: u32, bytes: &[u8]) -> Result<Self> { Self::decode_mode(id, bytes, true) }
    fn decode_mode(id: u32, bytes: &[u8], local: bool) -> Result<Self> {
        let mut d = Self { id, model: 0, name: "null".into(), examine: "null".into(), zoom: 2000,
            angles: [0;3], offset:[0;2], stackable:0, price:1, wear_positions:[-1;3], members:false,
            ground_operations:vec![Operation { index:2, sub:None, condition:None, text:"Take".into() }],
            inventory_operations:[None,None,None,None,Some("Drop".into())], sub_operations:Default::default(),
            recolor:vec![], retexture:vec![], shift_click:-2, wear_models:[[-1;3];2], wear_offset:[0;2],
            head_models:[[-1;2];2], category:-1, weight:0, note:[-1;2], bought:[-1;2], placeholder:[-1;2],
            counts:[(0,0);10], scale:[128;3], ambient:0, contrast:0, team:0, tradeable:true,
            stockmarket:false, searchable:false, params:HashMap::new(), trailing:vec![] };
        let mut r=Reader::new(bytes);
        loop { let op=r.u8()?; match op {
            0=>break,
            1 if local=>d.model=r.u16()? as i32,
            23|25 if local=>{let g=usize::from(op==25);d.wear_models[g][0]=r.u16()? as i32;d.wear_offset[g]=r.u8()? as i32;},
            24|26|78|79 if local=>{let (g,n)=match op {24=>(0,1),26=>(1,1),78=>(0,2),_=>(1,2)};d.wear_models[g][n]=r.u16()? as i32;},
            90..=93 if local=>{let (g,n)=match op {90=>(0,0),91=>(1,0),92=>(0,1),_=>(1,1)};d.head_models[g][n]=r.u16()? as i32;},
            200 if local=>{for _ in 0..r.u8()? {r.u16()?;}},
            201..=203 if local=>{r.u32()?;},
            204|205|207|209..=211 if local=>{r.u16()?;},
            206 if local=>d.tradeable=false,
            208|212 if local=>{r.u8()?;},
            2=>d.name=r.string()?,3=>d.examine=r.string()?,4=>d.zoom=r.u16()? as i32,
            5|6=>d.angles[(op-5) as usize]=r.u16()? as i32,
            7|8=>d.offset[(op-7) as usize]=r.u16()? as i16 as i32,
            9=>{r.string()?;},11=>d.stackable=1,12=>d.price=r.u32()? as i32,
            13|14=>d.wear_positions[(op-13) as usize]=r.u8()? as i32,
            15=>d.tradeable=false,16=>d.members=true,27=>d.wear_positions[2]=r.u8()? as i32,
            30..=34=>{let index=op-30;let text=r.string()?;if !text.eq_ignore_ascii_case("hidden") {d.ground_operations.retain(|o|o.index!=index);d.ground_operations.push(Operation{index,sub:None,condition:None,text});}},
            200=>{let index=r.u8()?;let sub=Some(r.u8()?);let text=r.string()?;d.ground_operations.push(Operation{index,sub,condition:None,text});},
            201|202=>{let index=r.u8()?;let a=r.u16()?;let b=r.u16()?;let c=if op==202 {Some(r.u16()?)}else{None};let min=r.u32()? as i32;let max=r.u32()? as i32;let text=r.string()?;d.ground_operations.push(Operation{index,sub:None,condition:Some((a,b,c,min,max)),text});},
            35..=39=>d.inventory_operations[(op-35) as usize]=Some(r.string()?),
            40|41=>{let pairs=if op==40 {&mut d.recolor}else{&mut d.retexture};pairs.clear();for _ in 0..r.u8()? {pairs.push((r.u16()?,r.u16()?));}},
            42=>d.shift_click=r.u8()? as i8 as i32,
            43=>{let parent=r.u8()? as usize;if parent<5&&d.sub_operations[parent].is_none(){d.sub_operations[parent]=Some(vec![None;20]);}loop{let sub=r.u8()?;if sub==0{break;}let text=r.string()?;if parent<5&&sub<=20{d.sub_operations[parent].as_mut().unwrap()[sub as usize-1]=Some(text);}}},
            44=>d.model=r.u32()? as i32,
            45|48=>{let g=usize::from(op==48);d.wear_models[g][0]=r.u32()? as i32;d.wear_offset[g]=r.u8()? as i32;},
            46|47|49|50=>{let (g,n)=match op {46=>(0,1),47=>(0,2),49=>(1,1),_=>(1,2)};d.wear_models[g][n]=r.u32()? as i32;},
            51..=54=>{let n=(op-51) as usize;d.head_models[n/2][n%2]=r.u32()? as i32;},
            65=>{d.stockmarket=true;d.searchable=true;},75=>d.weight=r.u16()? as i16 as i32,
            94=>d.category=r.u16()? as i32,95=>d.angles[2]=r.u16()? as i32,
            97|98=>d.note[(op-97) as usize]=r.u16()? as i32,
            100..=109=>d.counts[(op-100) as usize]=(r.u16()? as i32,r.u16()? as i32),
            110..=112=>d.scale[(op-110) as usize]=r.u16()? as i32,
            113=>d.ambient=r.u8()? as i8 as i32,114=>d.contrast=r.u8()? as i8 as i32*5,
            115=>d.team=r.u8()? as i32,139|140=>d.bought[(op-139) as usize]=r.u16()? as i32,
            148|149=>d.placeholder[(op-148) as usize]=r.u16()? as i32,160=>d.stackable=2,
            249=>{for _ in 0..r.u8()? {let kind=r.u8()?;let key=r.u24()?;let value=match kind{1=>Param::String(r.string()?),2=>Param::Long(r.u64()? as i64),_=>Param::Int(r.u32()? as i32)};d.params.entry(key).or_insert(value);}},
            _=>{},
        }}
        if d.stackable==1 {d.weight=0;}
        d.trailing=r.bytes(r.remaining())?.to_vec();Ok(d)
    }
    pub fn shift_click_index(&self)->Option<usize>{if self.shift_click>=0 {let i=self.shift_click as usize;self.inventory_operations.get(i)?.as_ref().map(|_|i)}else if self.shift_click != -1 &&self.inventory_operations[4].as_ref().is_some_and(|s|s.eq_ignore_ascii_case("Drop")){Some(4)}else{None}}
    pub fn count_variant(&self, quantity:i32)->u32 {let mut id=self.id;if quantity>1 {for &(item,threshold) in &self.counts {if threshold!=0&&quantity>=threshold{id=item as u32;}}}id}
    fn derive(&mut self, base:&Self, template:&Self) {
        self.model=template.model;self.zoom=template.zoom;self.angles=template.angles;self.offset=template.offset;
        let colors=if self.bought[1]!=-1 {base}else{template};self.recolor=colors.recolor.clone();self.retexture=colors.retexture.clone();
        if self.note[1]!=-1 {self.name=base.name.clone();self.examine="Swap this note at any bank for the equivalent item".into();self.members=base.members;self.price=base.price;self.tradeable=base.tradeable;self.stackable=1;}
        if self.bought[1]!=-1 {self.name=base.name.clone();self.examine=base.examine.clone();self.members=base.members;self.stackable=base.stackable;self.category=base.category;self.wear_positions=base.wear_positions;self.wear_models=base.wear_models;self.head_models=base.head_models;self.team=base.team;self.ground_operations=base.ground_operations.clone();self.weight=base.weight;self.inventory_operations=base.inventory_operations.clone();self.inventory_operations[4]=Some("Discard".into());self.sub_operations=base.sub_operations.clone();self.sub_operations[4]=None;self.price=0;self.tradeable=false;self.stockmarket=false;self.searchable=false;}
        if self.placeholder[1]!=-1 {self.stackable=template.stackable;self.category=template.category;self.name=base.name.clone();self.examine=base.examine.clone();self.price=0;self.members=false;self.tradeable=false;self.stockmarket=false;self.searchable=false;}
    }
}
pub struct Definitions { pub items:HashMap<u32,Definition> }
impl Definitions {
    pub fn load(cache:&mut DiskCache)->Result<Self>{let raw=cache.group(2,10,[0;4])?;Self::from_records(raw,true)}
    pub fn load_for_world(cache:&mut DiskCache,members:bool)->Result<Self> {
        if members { return Self::load(cache); }
        let mut enabled=HashSet::new();
        for (id,bytes) in cache.group(2,11,[0;4])? {
            if !crate::param_def::Definition::decode(&bytes)?.auto_disable { enabled.insert(id); }
        }
        Self::from_records_for_world(cache.group(2,10,[0;4])?,true,false,&enabled)
    }
    pub fn from_records(records:Vec<(u32,Vec<u8>)>,local:bool)->Result<Self>{
        Self::from_records_for_world(records,local,true,&HashSet::new())
    }
    /// Resolve each linked definition in its world context, as eq.aa does recursively.
    pub fn from_records_for_world(records:Vec<(u32,Vec<u8>)>,local:bool,members:bool,enabled_params:&HashSet<u32>)->Result<Self>{
        let mut raw=HashMap::new();let mut links=HashMap::new();
        for (id,bytes) in records {if matches!(bytes.first(),Some(97|139|148)) {let mut r=Reader::new(&bytes);r.u8()?;let base=r.u16()?;r.u8()?;let template=r.u16()?;links.insert(id,(base as u32,template as u32));}raw.insert(id,Definition::decode_mode(id,&bytes,local).map_err(|e|Error::Io(format!("item {id}: {e}")))?);}
        fn resolve(id:u32,raw:&HashMap<u32,Definition>,links:&HashMap<u32,(u32,u32)>,done:&mut HashMap<u32,Definition>,visiting:&mut HashSet<u32>,members:bool,enabled_params:&HashSet<u32>)->Result<()> {
            if done.contains_key(&id){return Ok(());}if !visiting.insert(id){return Err(Error::Invalid("cyclic item templates"));}
            let mut d=raw.get(&id).ok_or(Error::Invalid("missing item template"))?.clone();
            if let Some(&(base,template))=links.get(&id){resolve(base,raw,links,done,visiting,members,enabled_params)?;resolve(template,raw,links,done,visiting,members,enabled_params)?;d.derive(&done[&base],&done[&template]);}
            if !members && d.members {
                if !links.contains_key(&id) { d.name.push_str(" (Members)"); }
                d.examine="Login to a members' server to use this object.".into();
                d.searchable=false;d.tradeable=false;
                d.ground_operations=vec![Operation{index:2,sub:None,condition:None,text:"Take".into()}];
                for i in 0..4 {d.inventory_operations[i]=None;d.sub_operations[i]=None;}
                d.shift_click=-2;d.category=-1;d.team=0;
                d.params.retain(|key,_|enabled_params.contains(key));
            }
            visiting.remove(&id);done.insert(id,d);Ok(())
        }
        let mut items=HashMap::new();let mut visiting=HashSet::new();for &id in raw.keys(){resolve(id,&raw,&links,&mut items,&mut visiting,members,enabled_params)?;}Ok(Self{items})
    }
}
