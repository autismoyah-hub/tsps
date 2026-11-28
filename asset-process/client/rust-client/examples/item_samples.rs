use gielinor_client::{disk_cache::DiskCache,item::{Definition,Definitions}};
use std::io::Write;
fn main()->Result<(),Box<dyn std::error::Error>> {
 let a=std::env::args().collect::<Vec<_>>();let mut c=DiskCache::open(&a[1],64<<20)?;let records=c.group(2,10,[0;4])?;
 let mut out=std::fs::File::create(&a[2])?;out.write_all(&(records.len() as u32).to_be_bytes())?;
 let mut trailing=0;for (id,data) in &records {let d=Definition::decode_local(*id,data).map_err(|e|format!("item {id}: {e}"))?;trailing+=usize::from(!d.trailing.is_empty());out.write_all(&id.to_be_bytes())?;out.write_all(&(data.len() as u32).to_be_bytes())?;out.write_all(data)?;}
 let defs=Definitions::from_records(records,true)?;println!("Decoded and resolved {} items; {trailing} records with trailing bytes",defs.items.len());Ok(())
}
