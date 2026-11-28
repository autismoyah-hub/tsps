//! Explicit combined Classic/Enhanced native profiling; no private action adapter.
#[global_allocator]
static ALLOCATOR:gielinor_client::client_metrics::TrackedAllocator=gielinor_client::client_metrics::TrackedAllocator;

const USAGE:&str="CONTENT CACHE ADDRESS USER PASSWORD_FILE PUBLIC_KEY OUTPUT_DIRECTORY MODE ADAPTER_INDEX_OR_AUTO [renderer options] [--orbit-step=-512..512] [--zoom=X,Y] [--window=WIDTHxHEIGHT] [--zero-area-frames=1..120] [--headless] [--sample-frames=60..480] [--warmup-frames=1..4096] [--scene-filter=full|essential|no-furnishings|no-floor-finish|no-upper-architecture] [--smoke]\nRenderer options: --scale=0.25..1.5 --shadows=0|256|512|1024|2048|4096 --ao=0..16 --aa=on|off --exposure=0.1..4 --texture-size=64|128 --light=X,Y,Z";

fn settings(args:&[String])->Result<(gielinor_client::enhanced::Settings,bool,gielinor_client::ui_profile::Benchmark),String> {
    if args.len()<9 {return Err(USAGE.into());}
    let smoke=args.len()>9 && args.last().is_some_and(|arg|arg=="--smoke");
    let (benchmark,renderer)=gielinor_client::ui_profile::Benchmark::from_options(&args[9..args.len()-usize::from(smoke)])?;
    if smoke && benchmark.run().is_some() {return Err("--smoke cannot shorten a measured frame window".into());}
    let settings=gielinor_client::enhanced::Settings::from_launch_args(&args[7],&args[8],&renderer)?;
    Ok((settings,smoke,benchmark))
}

#[cfg(windows)]
fn main()->Result<(),Box<dyn std::error::Error>> {
    use gielinor_client::{authored_content::Content,disk_cache::DiskCache,enhanced::Renderer};
    let args:Vec<_>=std::env::args().skip(1).collect();
    if args.as_slice()==["--help"] {println!("{USAGE}");return Ok(());}
    if args.as_slice()==["--adapters"] {for adapter in Renderer::adapters()? {println!("{adapter:?}");}return Ok(());}
    let (settings,smoke,benchmark)=settings(&args)?;
    let content=Content::read(std::path::Path::new(&args[0]))?;
    let mut cache=DiskCache::open_remote(&args[1],64<<20,args[2].parse()?)?;
    cache.wait_for_recovery(true);let cache=content.mount(cache)?;
    gielinor_client::viewport::profiled_enhanced_authored_live(cache,&args[2],&args[3],&args[4],&args[5],args[6].clone().into(),settings,benchmark,smoke)?;
    Ok(())
}
#[cfg(not(windows))]
fn main(){eprintln!("Enhanced native profiling requires Windows");std::process::exit(1);}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(extra:&[&str])->Vec<String> {
        ["content","cache","address","user","password-file","key","output","lite","auto"].into_iter().chain(extra.iter().copied()).map(String::from).collect()
    }
    #[test]
    fn malformed_launch_options_fail_before_content_or_network_access() {
        for extra in [["--smoke","--scale=1"],["--scale=NaN","--smoke"],["--ao=2","--ao=3"]] {
            assert!(settings(&args(&extra)).is_err());
        }
        let mut missing_adapter=args(&[]);missing_adapter[8]="--smoke".into();
        assert!(settings(&missing_adapter).is_err());
        assert!(settings(&missing_adapter[..8]).is_err());
    }
    #[test]
    fn explicit_smoke_and_renderer_options_are_preserved() {
        let (s,smoke,_)=settings(&args(&["--scale=1.25","--ao=4","--smoke"])).unwrap();
        assert!(smoke);assert_eq!(s.scale,1.25);assert_eq!(s.ao,4);
        assert!(!settings(&args(&[])).unwrap().1);
        assert!(settings(&args(&["--sample-frames=300","--smoke"])).is_err());
        assert!(settings(&args(&["--sample-frames=300","--headless"])).unwrap().2.headless());
    }
    #[test]
    fn benchmark_options_are_explicit_and_separate_from_renderer_settings() {
        let (s,smoke,b)=settings(&args(&["--orbit-step=64","--zoom=256,320","--window=960x640","--ao=4","--smoke"])).unwrap();
        assert!(smoke);assert_eq!(s.ao,4);assert_eq!(b.yaw(16350),30);
        assert_eq!(b.zoom(),Some([256,320]));assert_eq!(b.window(),Some([960,640]));
        assert_eq!(settings(&args(&[])).unwrap().2,gielinor_client::ui_profile::Benchmark::default());
        assert!(settings(&args(&["--orbit-step=513"])).is_err());
        assert!(settings(&args(&["--window=960x640","--window=765x503"])).is_err());
    }
}
