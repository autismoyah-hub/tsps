use gielinor_client::{disk_cache::DiskCache,interface_state::Interfaces,interface_packet::Update,interface_render::Renderer,item::Definitions,inventory::Inventories,raster::Raster};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let a=std::env::args().collect::<Vec<_>>();
 // Native preview evidence is PNG only; a BMP or other path is refused before rendering.
 if a.len()!=4 || !a[3].to_ascii_lowercase().ends_with(".png") {return Err("interface_preview CACHE GROUP OUTPUT.png".into());}
 let mut cache=DiskCache::open(&a[1],64<<20)?;let group=a[2].parse()?;let items=Definitions::load(&mut cache)?;let mut inventories=Inventories::default();let mut state=Interfaces::default();
 // OpenTop aligns fill-parent roots against the stored canvas; size it first (hl.ae).
 state.resize_canvas([765,503],false)?;
 state.apply(&mut cache,&items,&mut inventories,Update::OpenTop(group))?;let layout=state.layout([765,503])?;let mut renderer=Renderer::load(&mut cache,0.8)?;let mut raster=Raster::new(765,503,1<<20)?;raster.clear(0xff29251f);let report=renderer.draw(&mut cache,&items,&state,&layout,&mut raster)?;std::fs::write(&a[3],raster.png()?)?;println!("Interface {group}: {} widgets, {} queued load scripts; {report:?}",layout.widgets.len(),state.scripts.len());Ok(())
}
