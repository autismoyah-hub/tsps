//! Explicit optional Enhanced launcher; Classic remains the default elsewhere.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use gielinor_client::{authored_content::Content,disk_cache::DiskCache,enhanced::{Settings,Renderer}};
    let args:Vec<_>=std::env::args().skip(1).collect();
    if args.as_slice()==["--adapters"] {for adapter in Renderer::adapters()? {println!("{adapter:?}");} return Ok(());}
    const USAGE:&str="CONTENT CACHE ADDRESS USER PASSWORD_FILE PUBLIC_KEY COMMAND_DIRECTORY MODE ADAPTER_INDEX_OR_AUTO [--headless] [renderer options]\nRenderer options: --scale=0.25..1.5 --shadows=0|256|512|1024|2048|4096 --ao=0..16 --aa=on|off --exposure=0.1..4 --texture-size=64|128 --light=X,Y,Z";
    if args.as_slice()==["--help"] {println!("{USAGE}");return Ok(());}
    if args.len()<9 {return Err(USAGE.into());}
    let headless=args[9..].iter().filter(|arg|arg.as_str()=="--headless").count();
    if headless>1 {return Err("Repeated --headless".into());}
    let options=args[9..].iter().filter(|arg|arg.as_str()!="--headless").cloned().collect::<Vec<_>>();
    let settings=Settings::from_launch_args(&args[7],&args[8],&options)?;
    let content=Content::read(std::path::Path::new(&args[0]))?;
    let mut cache=DiskCache::open_remote(&args[1],64<<20,args[2].parse()?)?;
    cache.wait_for_recovery(true);let cache=content.mount(cache)?;
    gielinor_client::viewport::enhanced_private_authored_test(cache,&args[2],&args[3],&args[4],&args[5],std::path::PathBuf::from(&args[6]),settings,headless==1)?;
    Ok(())
}
#[cfg(not(windows))]
fn main(){eprintln!("Enhanced native launcher requires Windows");std::process::exit(1);}
