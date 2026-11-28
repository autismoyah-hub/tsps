use gielinor_client::{disk_cache::DiskCache, identity_kit::Kit};
use std::io::Write;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    let mut c = DiskCache::open(&a[1], 64 << 20)?;
    let records = c.group(2, 3, [0; 4])?;
    let mut stock = vec![];
    for (id, data) in &records {
        Kit::decode(*id, data, true)?;
        if Kit::decode(*id, data, false).is_ok() {
            stock.push((*id, data));
        }
    }
    let mut o = std::fs::File::create(&a[2])?;
    o.write_all(&(stock.len() as u32).to_be_bytes())?;
    for (id, data) in &stock {
        o.write_all(&id.to_be_bytes())?;
        o.write_all(&(data.len() as u32).to_be_bytes())?;
        o.write_all(data)?;
    }
    println!(
        "Decoded {} identity kits, {} stock-compatible",
        records.len(),
        stock.len()
    );
    Ok(())
}
