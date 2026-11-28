//! Explicit optional authored-content launcher; no test-action adapter.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use gielinor_client::{authored_content::Content, disk_cache::DiskCache};
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(args.len() == 6 || args.len() == 7 && args[6] == "--smoke") {
        return Err("CONTENT CACHE ADDRESS USER PASSWORD_FILE PUBLIC_KEY [--smoke]".into());
    }
    let content = Content::read(std::path::Path::new(&args[0]))?;
    let mut cache = DiskCache::open_remote(&args[1], 64 << 20, args[2].parse()?)?;
    cache.wait_for_recovery(true);
    let cache = content.mount(cache)?;
    gielinor_client::viewport::live_view_with_cache(
        cache,
        &args[2],
        &args[3],
        &args[4],
        &args[5],
        args.len() == 7,
    )?;
    Ok(())
}
#[cfg(not(windows))]
fn main() {
    eprintln!("The native authored launcher currently requires Windows.");
    std::process::exit(1);
}
