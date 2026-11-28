#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = gielinor_client::audio_output::Output::open()?;
    for _ in 0..8 {
        output.write(vec![0; 441])?;
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    drop(output);
    println!("Windows PCM output: eight silent buffers submitted and released");
    Ok(())
}
#[cfg(not(windows))]
fn main() {}
