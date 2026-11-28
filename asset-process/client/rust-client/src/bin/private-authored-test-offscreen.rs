//! Private validation executable, offscreen build only (#233). The same reviewed command grammar as
//! `private-authored-test`, with `--headless` required; parsing and dispatch live in the library's
//! `offscreen_cli`, so this binary has no parser of its own. No window, OS input or presentation code is
//! compiled into it.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args = gielinor_client::offscreen_cli::parse(&args)?;
    gielinor_client::offscreen_cli::run(args)?;
    Ok(())
}
#[cfg(not(windows))]
fn main() {
    eprintln!("The native authored launcher currently requires Windows.");
    std::process::exit(1);
}
