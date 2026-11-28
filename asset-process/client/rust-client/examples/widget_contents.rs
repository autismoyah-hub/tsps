use gielinor_client::{disk_cache::DiskCache,widget::Widget};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let mut c=DiskCache::open(&std::env::args().nth(1).unwrap(),64<<20)?;
 for (id,bytes) in c.group(3,548,[0;4])? {let w=Widget::decode((548<<16)|id,&bytes)?;if w.content!=0{println!("widget {} content {} type {} text {:?}",w.id,w.content,w.kind,w.text);}}
 Ok(())
}
