//! Private validation executable. Requires an explicit command directory.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use gielinor_client::{authored_content::Content, disk_cache::DiskCache};
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len()<7 || args.len()>8 || (args.len()==8 && args[7]!="--headless") {
        return Err("CONTENT CACHE ADDRESS USER PASSWORD_FILE PUBLIC_KEY COMMAND_DIRECTORY [--headless]".into());
    }
    let content = Content::read(std::path::Path::new(&args[0]))?;
    let mut cache = DiskCache::open_remote(&args[1], 64 << 20, args[2].parse()?)?;
    cache.wait_for_recovery(true);
    let cache = content.mount(cache)?;
    gielinor_client::viewport::private_authored_test(
        cache,
        &args[2],
        &args[3],
        &args[4],
        &args[5],
        std::path::PathBuf::from(&args[6]),
        args.len()==8,
    )?;
    Ok(())
}
#[cfg(not(windows))]
fn main() {
    eprintln!("The native authored launcher currently requires Windows.");
    std::process::exit(1);
}
