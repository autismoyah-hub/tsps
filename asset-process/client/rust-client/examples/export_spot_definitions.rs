use gielinor_client::{disk_cache::DiskCache, spot_animation::Definition, Result};
use std::io::Write;
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let mut cache = DiskCache::open(&args[1], 64 << 20)?;
    let defs = Definition::load_all(&mut cache)?;
    let mut ids: Vec<_> = defs.keys().copied().collect();
    ids.sort_unstable();
    let mut out =
        std::fs::File::create(&args[2]).map_err(|e| gielinor_client::Error::Io(e.to_string()))?;
    let mut bytes = vec![];
    bytes.extend_from_slice(&(ids.len() as u32).to_be_bytes());
    for id in ids {
        let d = &defs[&id];
        let mut data = vec![3];
        data.extend_from_slice(&d.model.to_be_bytes());
        if d.sequence >= 0 {
            data.push(2);
            data.extend_from_slice(&(d.sequence as u16).to_be_bytes());
        }
        for (op, value) in [
            (4, d.scale[0] as u16),
            (5, d.scale[1] as u16),
            (6, d.rotation),
        ] {
            data.push(op);
            data.extend_from_slice(&value.to_be_bytes());
        }
        data.extend_from_slice(&[7, d.ambient as u8, 8, d.contrast as u8]);
        if !d.flag {
            data.push(10);
        }
        for (op, pairs) in [(40, &d.recolor), (41, &d.retexture)] {
            data.push(op);
            data.push(pairs.len() as u8);
            for &(a, b) in pairs {
                data.extend_from_slice(&a.to_be_bytes());
                data.extend_from_slice(&b.to_be_bytes());
            }
        }
        data.push(0);
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&data);
    }
    out.write_all(&bytes)
        .map_err(|e| gielinor_client::Error::Io(e.to_string()))?;
    println!("Exported {} spot animation definitions", defs.len());
    Ok(())
}
