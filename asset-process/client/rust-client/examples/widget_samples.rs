use gielinor_client::{disk_cache::DiskCache, widget::Widget};
use std::io::Write;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    let mut c = DiskCache::open(&a[1], 64 << 20)?;
    let ids = c
        .reference(3)?
        .groups
        .iter()
        .map(|g| g.id)
        .collect::<Vec<_>>();
    let mut samples = Vec::new();
    let mut count = 0;
    let mut legacy = 0;
    let mut kinds = std::collections::BTreeMap::new();
    for id in ids {
        for (file, data) in c.group(3, id, [0; 4])? {
            let w = Widget::decode(id << 16 | file, &data)
                .map_err(|e| format!("widget {id}:{file}: {e}"))?;
            count += 1;
            legacy += (!w.if3) as usize;
            *kinds.entry(w.kind).or_insert(0usize) += 1;
            samples.push((w.id, data));
        }
    }
    let mut out = std::fs::File::create(&a[2])?;
    out.write_all(&(samples.len() as u32).to_be_bytes())?;
    for (id, data) in samples {
        out.write_all(&id.to_be_bytes())?;
        out.write_all(&(data.len() as u32).to_be_bytes())?;
        out.write_all(&data)?;
    }
    println!("Decoded {count} widgets ({legacy} legacy): {kinds:?}");
    Ok(())
}
