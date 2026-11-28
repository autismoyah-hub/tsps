use gielinor_client::{disk_cache::DiskCache, npc::Definition};
use std::io::Write;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    let records = cache.group(2, 9, [0; 4])?;
    let mut stock = vec![];
    let mut models = 0;
    let mut errors = vec![];
    for (id, data) in &records {
        match Definition::decode_local(*id, data) {
            Ok(d) => models += d.models.len() + d.head_models.len(),
            Err(e) => errors.push(format!("{id}: {e}")),
        };
        if Definition::decode(*id, data).is_ok() {
            stock.push((*id, data));
        }
    }
    let mut out = std::fs::File::create(&a[2])?;
    out.write_all(&(stock.len() as u32).to_be_bytes())?;
    for (id, data) in &stock {
        out.write_all(&id.to_be_bytes())?;
        out.write_all(&(data.len() as u32).to_be_bytes())?;
        out.write_all(data)?;
    }
    println!(
        "NPCs={}, model references={models}, stock-compatible={}, errors={}",
        records.len(),
        stock.len(),
        errors.len()
    );
    for error in errors.iter().take(30) {
        println!("{error}");
    }
    if !errors.is_empty() {
        return Err("NPC decode failed".into());
    }
    Ok(())
}
