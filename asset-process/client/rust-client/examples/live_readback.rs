//! research-live: read assets back from a (packed) cache through the client's own loaders and decoders.
//! usage: live_readback CACHE OUTDIR WHAT...
//! WHAT = model:ID | object:ID | npc:ID | item:ID | seq:ID | map:REGION | counts
//! For each, the raw file as DiskCache::file returns it (which validates CRC and version against the reference
//! table, disk_cache.rs validate_container) is written to OUTDIR, and one line of decoded fields is printed:
//!   model   Model::decode                       vertices, faces
//!   object  object::Definition::load_all        the record the world loader uses
//!   npc     npc::Definition::load_all
//!   item    item::Definitions::load             (all fields: research_items `defs`)
//!   seq     animation::Sequence::load_all, then every frame it names: file, base id, Skeleton::decode, Frame::decode
//!   map     terrain::Region::decode + terrain::locations
//!   counts  how many models / objects / NPCs / items / sequences / map squares the cache lists
use gielinor_client::{animation::{Frame, Sequence, Skeleton}, disk_cache::DiskCache, item::Definitions, model::Model, npc, object, terrain};
type R<T> = Result<T, Box<dyn std::error::Error>>;
fn main() -> R<()> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() < 4 { return Err("CACHE OUTDIR WHAT...".into()); }
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    let out = std::path::Path::new(&a[2]);
    std::fs::create_dir_all(out)?;
    let objects = object::Definition::load_all(&mut cache)?;
    let npcs = npc::Definition::load_all(&mut cache)?;
    let items = Definitions::load(&mut cache)?;
    let seqs = Sequence::load_all(&mut cache)?;
    for what in &a[3..] {
        if what == "counts" {
            println!("counts models={} objects={} npcs={} items={} sequences={} frame_groups={} map_squares={}",
                cache.reference(7)?.groups.len(), objects.len(), npcs.len(), items.items.len(), seqs.len(),
                cache.reference(0)?.groups.len(), cache.reference(5)?.groups.len());
            continue;
        }
        let (kind, id) = what.split_once(':').ok_or("WHAT")?;
        let id: u32 = id.parse()?;
        match kind {
            "model" => {
                let b = cache.file(7, id, 0, [0; 4])?;
                let m = Model::decode(&b)?;
                std::fs::write(out.join(format!("model-{id}.bin")), &b)?;
                println!("model {id} bytes={} vertices={} faces={}", b.len(), m.vertices.len(), m.faces.len());
            }
            "object" => {
                let b = cache.file(2, 6, id, [0; 4])?;
                std::fs::write(out.join(format!("object-{id}.bin")), &b)?;
                let d = objects.get(&id).ok_or("object not loaded")?;
                println!("object {id} name={:?} models={:?} shapes={:?} size={:?} ambient={} contrast={} interact_type={} blocks_projectiles={} interactive={} non_flat_shading={}",
                    d.name, d.models, d.model_shapes, d.size, d.ambient, d.contrast, d.interact_type, d.blocks_projectiles, d.interactive, d.non_flat_shading);
            }
            "npc" => {
                let b = cache.file(2, 9, id, [0; 4])?;
                std::fs::write(out.join(format!("npc-{id}.bin")), &b)?;
                let d = npcs.get(&id).ok_or("npc not loaded")?;
                println!("npc {id} name={:?} models={:?} size={} stand={} walk={} recolor={:?} scale={:?} ambient={} contrast={} z_buffer={} combat={} ops={:?}",
                    d.name, d.models, d.size, d.animations[0], d.animations[3], d.recolor, d.scale, d.ambient, d.contrast, d.z_buffer, d.combat,
                    d.operations.iter().map(|o| (o.index, o.text.clone())).collect::<Vec<_>>());
            }
            "item" => {
                let b = cache.file(2, 10, id, [0; 4])?;
                std::fs::write(out.join(format!("item-{id}.bin")), &b)?;
                println!("item {id} bytes={} loaded={}", b.len(), items.items.contains_key(&id));
            }
            "seq" => {
                let b = cache.file(2, 12, id, [0; 4])?;
                std::fs::write(out.join(format!("seq-{id}.bin")), &b)?;
                let s = seqs.get(&id).ok_or("sequence not loaded")?;
                let mut ok = 0;
                for &f in &s.frames {
                    let (g, n) = (f >> 16, f & 65535);
                    let fb = cache.file(0, g, n, [0; 4])?;
                    std::fs::write(out.join(format!("frame-{g}-{n}.bin")), &fb)?;
                    let base = u16::from_be_bytes([fb[0], fb[1]]);
                    let sk = Skeleton::decode(base, &cache.file(1, base as u32, 0, [0; 4])?)?;
                    Frame::decode(&fb, &sk)?;
                    ok += 1;
                }
                println!("seq {id} bytes={} frames={:?} lengths={:?} priority={} decoded_frames={ok}", b.len(), s.frames, s.lengths, s.priority);
            }
            "map" => {
                let files = cache.group(5, id, [0; 4])?;
                let t = &files.iter().find(|(n, _)| *n == 0).ok_or("terrain")?.1;
                let l = &files.iter().find(|(n, _)| *n == 1).ok_or("locations")?.1;
                std::fs::write(out.join(format!("map-{id}-0.bin")), t)?;
                std::fs::write(out.join(format!("map-{id}-1.bin")), l)?;
                let region = terrain::Region::decode(t, ((id >> 8) * 64) as i32, ((id & 255) * 64) as i32)?;
                let locs = terrain::locations(l, 1_000_000)?;
                let under = region.tiles.iter().filter(|t| t.underlay != 0).count();
                println!("map {id} terrain_bytes={} tiles_with_underlay={under} placements={} first={:?}", t.len(), locs.len(), locs.first());
            }
            _ => return Err("WHAT".into()),
        }
    }
    Ok(())
}
