//! Normal-world original map package with the maintained native profiler.
#[global_allocator]
static ALLOCATOR:gielinor_client::client_metrics::TrackedAllocator=gielinor_client::client_metrics::TrackedAllocator;
#[cfg(windows)]
fn main()->Result<(),Box<dyn std::error::Error>>{
 use gielinor_client::{asset_package::Package,disk_cache::DiskCache,enhanced::Settings,ui_profile::Benchmark};
 let a=std::env::args().skip(1).collect::<Vec<_>>();if a.len()<10{return Err("PACKAGE SHA CACHE ADDRESS USER PASSWORD KEY OUTPUT MODE ADAPTER [profiling/render options] [--smoke]".into());}
 if a[1].len()!=64{return Err("SHA256 length".into());}let mut pin=[0;32];for(i,v)in pin.iter_mut().enumerate(){*v=u8::from_str_radix(&a[1][i*2..i*2+2],16)?;}
 let smoke=a.last().is_some_and(|v|v=="--smoke");let(benchmark,options)=Benchmark::from_options(&a[10..a.len()-usize::from(smoke)])?;let settings=Settings::from_launch_args(&a[8],&a[9],&options)?;
 let mut cache=DiskCache::open_remote(&a[2],64<<20,a[3].parse()?)?;cache.wait_for_recovery(true);cache.mount_package(Package::read(std::path::Path::new(&a[0]))?,pin)?;
 gielinor_client::viewport::profiled_enhanced_authored_live(cache,&a[3],&a[4],&a[5],&a[6],a[7].clone().into(),settings,benchmark,smoke)?;Ok(())
}
#[cfg(not(windows))]fn main(){panic!("Windows required");}
