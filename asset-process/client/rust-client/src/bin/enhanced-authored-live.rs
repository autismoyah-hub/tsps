//! Explicit optional Enhanced launcher; Classic remains the default elsewhere.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use gielinor_client::{authored_content::Content,disk_cache::DiskCache,enhanced::{Settings,Renderer}};
    let args:Vec<_>=std::env::args().skip(1).collect();
    if args.as_slice()==["--adapters"] {for adapter in Renderer::adapters()? {println!("{adapter:?}");} return Ok(());}
    const USAGE:&str="CONTENT CACHE ADDRESS USER PASSWORD_FILE PUBLIC_KEY MODE ADAPTER_INDEX_OR_AUTO [renderer options] [--smoke]\nRenderer options: --scale=0.25..1.5 --shadows=0|256|512|1024|2048|4096 --ao=0..16 --aa=on|off --exposure=0.1..4 --texture-size=64|128 --light=X,Y,Z";
    if args.as_slice()==["--help"] {println!("{USAGE}");return Ok(());}
    if args.len()<8 {return Err(USAGE.into());}
    let smoke=args.len()>8 && args.last().is_some_and(|arg|arg=="--smoke");
    let options=&args[8..args.len()-usize::from(smoke)];
    let settings=Settings::from_launch_args(&args[6],&args[7],options)?;
    let content=Content::read(std::path::Path::new(&args[0]))?;
    let mut cache=DiskCache::open_remote(&args[1],64<<20,args[2].parse()?)?;
    cache.wait_for_recovery(true);let cache=content.mount(cache)?;
    gielinor_client::viewport::enhanced_authored_live(cache,&args[2],&args[3],&args[4],&args[5],settings,smoke)?;
    Ok(())
}
#[cfg(not(windows))]
fn main(){eprintln!("Enhanced native launcher requires Windows");std::process::exit(1);}
