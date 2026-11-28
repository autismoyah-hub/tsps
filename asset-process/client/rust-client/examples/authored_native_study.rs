//! Explicit admitted authored-content navigation; no game login.
#[cfg(not(all(windows,feature="custom-assets")))]
fn main(){eprintln!("requires Windows and custom-assets");std::process::exit(1);}
#[cfg(all(windows,feature="custom-assets"))]
fn main()->Result<(),Box<dyn std::error::Error>> {
 use gielinor_client::{authored_content::Content,authored_floor::ORIGIN,disk_cache::DiskCache,scene::Scene,viewport::{StudyOptions,StudyViewpoint,study_view}};
 let a=std::env::args().skip(1).collect::<Vec<_>>();
 if a.len()<4{return Err("CONTENT REFERENCE_CACHE MODE ADAPTER [--js5=ADDRESS] [--smoke | --benchmark=NEW_DIRECTORY (native-profiling)]".into());}
 let mut options=StudyOptions::default();let mut remote=None;
 for arg in &a[4..]{
 #[cfg(feature="native-profiling")]
 if let Some(value)=arg.strip_prefix("--benchmark="){if value.is_empty()||options.benchmark.is_some(){return Err("empty or duplicate benchmark".into());}options.benchmark=Some(value.into());continue;}
 if arg=="--smoke"&&!options.smoke{options.smoke=true;}else if let Some(value)=arg.strip_prefix("--js5="){if remote.is_some(){return Err("duplicate JS5".into());}remote=Some(value.parse::<std::net::SocketAddr>()?);}else{return Err("unknown or duplicate study option".into());}}
    #[cfg(feature="native-profiling")]
    if options.benchmark.as_ref().is_some_and(|p|options.smoke||p.exists()){return Err("benchmark requires a new output directory and excludes smoke".into());}
 #[cfg(feature="enhanced-renderer")]{options.renderer=Some(gielinor_client::enhanced::Settings::from_launch_args(&a[2],&a[3],&[])?);}
 #[cfg(not(feature="enhanced-renderer"))]{if a[2]!="classic"||a[3]!="auto"{return Err("default study requires classic auto".into());}}
 let content=Content::read(std::path::Path::new(&a[0]))?;
 let mut cache=if let Some(address)=remote{let mut c=DiskCache::open_remote(&a[1],64<<20,address)?;c.wait_for_recovery(true);c}else{DiskCache::open(&a[1],64<<20)?};
 options.start=Some(StudyViewpoint{world:[ORIGIN[0]+44,ORIGIN[1]+38],distance:2600.});
 options.viewpoints=[(53,46,2200.),(64,64,2200.),(81,56,2600.),(59,56,10500.)].into_iter().map(|(x,z,distance)|StudyViewpoint{world:[ORIGIN[0]+x,ORIGIN[1]+z],distance}).collect();
 options.far=Some(30000);options.cutaway=true;
 let crcs=cache.login_crcs()?;let mut cache=content.mount(cache)?;
 let scene=Scene::authored_placement_study_at(&mut cache,ORIGIN)?;
 if crcs!=cache.login_crcs()?{return Err("original reference CRCs changed".into());}
 println!("Admitted declared-placement offline study; no game login; placements={}",scene.objects.len());
 study_view(cache,scene,0,options)?;Ok(())
}
