//! Asset system tool: dump a skeleton's groups and the decoded transforms of classic sequences as plain text,
//! for the label atlas (tools/label_armour.py and the atlas scripts read it).
//! usage: label_dump CACHE OUT.txt SEQ [SEQ...]
//! Lines:  EQUIP v0 .. v11                  the standard male figure's twelve slot values (kit id + 256)
//!         G index type label label ...     one per group of skeleton 0
//!         SEQ id nframes I m m m ...       interleave mask (without the sentinel) when present
//!         F seq frame group x y z group x y z ...   the frame's transforms in application order (pivots that
//!                                          Frame::decode inserts are included, as the client applies them)
#[allow(dead_code)]
#[path = "asset_pose.rs"]
mod pose;
use gielinor_client::{
    animation::{Frame, Sequence, Skeleton},
    disk_cache::DiskCache,
    identity_kit::Kit,
};
use std::fmt::Write;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() < 4 {
        return Err("CACHE OUT.txt SEQ [SEQ...]".into());
    }
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    let kits = Kit::load_all(&mut cache)?;
    let standard = pose::standard_figure(&kits);
    let mut out = String::new();
    writeln!(out, "EQUIP {}", standard.equipment.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" "))?;
    let skeleton = Skeleton::decode(0, &cache.file(1, 0, 0, [0; 4])?)?;
    for (i, (kind, labels)) in skeleton.transform_types.iter().zip(&skeleton.labels).enumerate() {
        writeln!(out, "G {i} {kind} {}", labels.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" "))?;
    }
    let sequences = Sequence::load_all(&mut cache)?;
    for id in &a[3..] {
        let id: u32 = id.parse()?;
        let Some(s) = sequences.get(&id) else {
            eprintln!("seq {id}: missing");
            continue;
        };
        if s.cached_animation.is_some() || s.frames.is_empty() {
            eprintln!("seq {id}: not a classic frame sequence");
            continue;
        }
        let mask = s.interleave.as_ref().map_or(String::new(), |v| format!(" I {}", v[..v.len() - 1].iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ")));
        let mut lines = String::new();
        let mut ok = true;
        for (n, &fid) in s.frames.iter().enumerate() {
            let files = match cache.group(0, fid >> 16, [0; 4]) {
                Ok(f) => f,
                Err(_) => { ok = false; break; }
            };
            let Some((_, data)) = files.iter().find(|(file, _)| *file == (fid & 65535)) else { ok = false; break; };
            let sid = u16::from_be_bytes([data[0], data[1]]);
            if sid != 0 { ok = false; break; }
            let frame = Frame::decode(data, &skeleton)?;
            write!(lines, "F {id} {n}")?;
            for t in &frame.transforms {
                write!(lines, " {} {} {} {}", t.label, t.values[0], t.values[1], t.values[2])?;
            }
            writeln!(lines)?;
        }
        if !ok {
            eprintln!("seq {id}: not on skeleton 0 or frames missing");
            continue;
        }
        writeln!(out, "SEQ {id} {}{mask}", s.frames.len())?;
        out.push_str(&lines);
    }
    std::fs::write(&a[2], out)?;
    Ok(())
}
