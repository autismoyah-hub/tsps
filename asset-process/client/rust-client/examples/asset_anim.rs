//! Asset system tool (research-animation): a figure or an NPC posed at chosen frames of a classic sequence that is
//! read FROM LOOSE FILES instead of the cache, so an authored or re-encoded animation can be seen without packing.
//! usage: asset_anim CACHE OUT.bmp [anim=DIR | seq=ID] [frames=F1,F2,..] [under=SEQ:FRAME] [zoom=N] [yaw=N]
//!                   [pitch=N] [cell=WxH] [aim=HEIGHT|head|body|legs|feet|hands] [draw=actor|depth|auto|widget]
//!                   [track=L1,L2,..] ENTRY [ENTRY...]
//! ENTRY = everything asset_pose accepts (none, item:ID, model:PATH@SLOT, outfits joined with +)
//!       | npc:ID            a creature from the cache, assembled, lit and resized as world_actors.rs does it
//!                           (npc_model::mesh, Model::light(64 + ambient, 850 + contrast), npc_model::resize after
//!                           posing); drawn with draw=auto unless draw= says otherwise
//! anim=DIR  a loose sequence directory:
//!     DIR/seq.dat              the sequence definition, the bytes of archive 2 group 12 file ID
//!     DIR/frame_G_F.dat        classic frame F of frame group G (archive 0 group G file F); seq.dat names it G<<16|F
//!     DIR/base_B.dat           optional frame base B (archive 1 group B file 0); taken from the cache when absent
//!   Frames the directory does not hold are read from the cache, so a sequence may mix new and real frames.
//! seq=ID    the same code path with every byte taken from the cache (the control for the round-trip proof).
//! dump=PATH writes the assembled REST mesh of the first entry as text, one vertex per line: x y z label (cache
//!           space, y down), for tools that pose it themselves (animlib.apply_frame, ik in author.py).
//! track=    prints, per cell, the centroid (x, height, z in worn space: height = -y) of the posed vertices
//!           carrying each listed vertex label, e.g. track=27,50 for the right hand and the weapon.
//!
//! What runs: the client's own Sequence::decode, Skeleton::decode, Frame::decode and Model::animate_classic /
//! animate_classic_mask (deform.rs), called in the order animation_runtime.rs apply_observed / apply_blended
//! call them for two classic sequences. Only the lookup of the bytes differs (a file instead of
//! DiskCache::group). The picture code is asset_pose's. A frame's base id is its own first two bytes for a
//! loose frame and the first two bytes of the first file of its group for a cache frame (as the client does).
#[allow(dead_code)]
#[path = "asset_pose.rs"]
mod pose;
use gielinor_client::{
    animation::{Frame, Sequence, Skeleton}, disk_cache::DiskCache, identity_kit::Kit, interface_model,
    item::Definitions, item_icon::Icons, item_model::Models, model::Model, npc, npc_model, raster::Raster,
};
use pose::{actor_draw, R};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

struct Source {
    dir: Option<PathBuf>,
    sequences: HashMap<u32, Sequence>,
    bases: HashMap<u16, Skeleton>,
}
impl Source {
    fn base(&mut self, cache: &mut DiskCache, id: u16) -> R<Skeleton> {
        if let Some(b) = self.bases.get(&id) { return Ok(b.clone()); }
        let loose = self.dir.as_ref().map(|d| d.join(format!("base_{id}.dat"))).filter(|p| p.exists());
        let data = match &loose {
            Some(p) => { println!("base {id}: from {}", p.display()); std::fs::read(p)? }
            None => cache.file(1, id as u32, 0, [0; 4])?,
        };
        let b = Skeleton::decode(id, &data)?;
        self.bases.insert(id, b.clone());
        Ok(b)
    }
    /// The decoded frame and its base. `loose` allows the directory to supply it.
    fn frame(&mut self, cache: &mut DiskCache, id: u32, loose: bool) -> R<(Skeleton, Frame, &'static str)> {
        let (group, file) = (id >> 16, id & 65535);
        if loose {
            if let Some(p) = self.dir.as_ref().map(|d| d.join(format!("frame_{group}_{file}.dat"))).filter(|p| p.exists()) {
                let data = std::fs::read(&p)?;
                let sid = u16::from_be_bytes(data.get(..2).ok_or("short frame file")?.try_into().unwrap());
                let base = self.base(cache, sid)?;
                let frame = Frame::decode(&data, &base)?;
                return Ok((base, frame, "file"));
            }
        }
        let files = cache.group(0, group, [0; 4])?;
        let first = &files.first().ok_or("empty frame group")?.1;
        let sid = u16::from_be_bytes(first.get(..2).ok_or("short frame")?.try_into().unwrap());
        let base = self.base(cache, sid)?;
        let data = &files.iter().find(|(f, _)| *f == file).ok_or("animation frame missing")?.1;
        let frame = Frame::decode(data, &base)?;
        Ok((base, frame, "cache"))
    }
}
fn aim_height(v: &str) -> R<i32> {
    Ok(match v {
        "head" => 180, "body" | "torso" | "chest" => 140, "hands" => 100, "legs" => 65, "feet" => 10,
        _ => v.parse::<i32>().map_err(|_| format!("aim={v}"))?,
    })
}
fn half_height(m: &Model) -> i32 {
    m.vertices.iter().map(|v| -(v[1] as i32)).max().unwrap_or(0).max(0) / 2
}
fn main() -> R<()> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() < 4 {
        return Err("CACHE OUT.bmp [anim=DIR | seq=ID] [frames=..] [under=SEQ:FRAME] [zoom= yaw= pitch= cell= aim= draw= track=] ENTRY...".into());
    }
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    let kits = Kit::load_all(&mut cache)?;
    let items = Definitions::load(&mut cache)?;
    let mut models = Models::default();
    let icons = Icons::load(&mut cache, 0.8)?;
    let (mut zoom, mut yaw, mut pitch) = (520i32, 192i32, 0i32);
    let mut seq: Option<u32> = None;
    let mut dir: Option<PathBuf> = None;
    let mut frames: Vec<Option<usize>> = Vec::new();
    let mut under: Option<(u32, usize)> = None;
    let mut cell: Option<(usize, usize)> = None;
    let mut aim: Option<i32> = None;
    let mut draw: Option<actor_draw::Draw> = None;
    let mut track: Vec<u8> = Vec::new();
    let mut dump: Option<String> = None;
    let mut entries = Vec::new();
    for e in &a[3..] {
        if let Some(v) = e.strip_prefix("zoom=") { zoom = v.parse()?; }
        else if let Some(v) = e.strip_prefix("yaw=") { yaw = v.parse()?; }
        else if let Some(v) = e.strip_prefix("pitch=") { pitch = v.parse()?; }
        else if let Some(v) = e.strip_prefix("seq=") { seq = Some(v.parse()?); }
        else if let Some(v) = e.strip_prefix("anim=") { dir = Some(PathBuf::from(v)); }
        else if let Some(v) = e.strip_prefix("frames=") {
            for f in v.split(',') { frames.push(if f == "bind" { None } else { Some(f.parse()?) }); }
        }
        else if let Some(v) = e.strip_prefix("under=") {
            let (s, f) = v.split_once(':').ok_or("under=SEQ:FRAME")?;
            under = Some((s.parse()?, f.parse()?));
        }
        else if let Some(v) = e.strip_prefix("cell=") {
            let (w, h) = v.split_once('x').ok_or("cell=WxH")?;
            cell = Some((w.parse()?, h.parse()?));
        }
        else if let Some(v) = e.strip_prefix("aim=") { aim = Some(aim_height(v)?); }
        else if let Some(v) = e.strip_prefix("draw=") { draw = Some(actor_draw::Draw::parse(v)?); }
        else if let Some(v) = e.strip_prefix("dump=") { dump = Some(v.to_string()); }
        else if let Some(v) = e.strip_prefix("track=") { for l in v.split(',') { track.push(l.parse()?); } }
        else { entries.push(e.clone()); }
    }
    if entries.is_empty() { return Err("no entries".into()); }
    let mut source = Source { dir: dir.clone(), sequences: Sequence::load_all(&mut cache)?, bases: HashMap::new() };
    // the action sequence: decoded by the client's decoder from the loose file, or taken from the cache
    let sequence: Option<Sequence> = match (&dir, seq) {
        (Some(d), _) => {
            let data = std::fs::read(d.join("seq.dat")).map_err(|e| format!("{}: {e}", d.join("seq.dat").display()))?;
            Some(Sequence::decode(seq.unwrap_or(65535), &data)?)
        }
        (None, Some(id)) => Some(source.sequences.get(&id).ok_or(format!("no sequence {id}"))?.clone()),
        (None, None) => None,
    };
    if frames.is_empty() { frames.push(if sequence.is_some() { Some(0) } else { None }); }
    if sequence.is_none() && frames.iter().any(|f| f.is_some()) { return Err("frames need seq= or anim=".into()); }
    if let Some(s) = &sequence {
        println!("seq {} ({}): frames={} lengths={:?} total={} loop={} priority={} weapon={} shield={} interleave={} max_loops={} precedence={} priority_moving={} restart={}",
            s.id, if dir.is_some() { "loose files" } else { "cache" }, s.frames.len(), s.lengths, s.total_length, s.loop_frames, s.priority,
            s.weapon, s.shield, s.interleave.as_ref().map_or("no".to_string(), |v| format!("{:?}", &v[..v.len() - 1])),
            s.max_loops, s.precedence, s.priority_moving, s.restart);
        if s.cached_animation.is_some() { return Err("this tool poses classic sequences only".into()); }
    }
    let mut painter = actor_draw::Painter::load(&mut cache)?;
    let (cw, ch) = cell.unwrap_or(if zoom < 400 { (600, 800) } else { (300, 400) });
    let standard = pose::standard_figure(&kits);
    let bare = standard.mesh(&mut cache, &mut models, &kits, &items, false)?;
    let npcs = if entries.iter().any(|e| e.starts_with("npc:")) { npc::Definition::load_all(&mut cache)? } else { HashMap::new() };
    let mut raster = Raster::new(cw * entries.len(), ch * frames.len(), 1 << 26)?;
    raster.clear(0xff302b25);
    for (k, e) in entries.iter().enumerate() {
        for (r, frame) in frames.iter().enumerate() {
            // ---- assemble and light in the rest pose
            let npc_def = match e.strip_prefix("npc:") {
                Some(id) => Some(npcs.get(&id.parse::<u32>()?).ok_or(format!("no npc {id}"))?),
                None => None,
            };
            let (mut m, colors, rest_half, mode);
            if let Some(d) = npc_def {
                m = npc_model::mesh(&mut cache, &mut models, d, false, None)?.ok_or("npc has no model")?;
                colors = m.light(64 + d.ambient, 850 + d.contrast, [-30, -50, -30])?;
                let mut rest = m.clone();
                npc_model::resize(&mut rest, d);
                rest_half = aim.unwrap_or_else(|| half_height(&rest));
                mode = draw.unwrap_or(actor_draw::Draw::Auto);
                if r == 0 {
                    let mut h = BTreeMap::<u8, usize>::new();
                    if let Some(s) = &m.vertex_skins { for &x in s { *h.entry(x).or_default() += 1; } }
                    println!("{e}: {:?} models {:?} size {} scale {:?} animations {:?} combat {}; {} vertices {} faces; rest height {}; bone weights {}; vertex labels {h:?}",
                        d.name, d.models, d.size, d.scale, d.animations, d.combat, m.vertices.len(), m.faces.len(), half_height(&rest) * 2, m.bone_weights.is_some());
                }
            } else {
                let mut o = pose::outfit(e, &standard, &items, &mut models, &mut cache)?;
                if r == 0 { for n in &o.notes { println!("{e}: {n}"); } }
                let mut player = standard.clone();
                player.equipment = o.equipment;
                if let (Some(s), Some(_)) = (&sequence, frame) {
                    let swapped = player.sequence_equipment(s);
                    if swapped.equipment != player.equipment {
                        for slot in [3, 5] {
                            if swapped.equipment[slot] != player.equipment[slot] { o.slot_models[slot] = None; }
                        }
                        player = swapped;
                    }
                }
                m = pose::assemble(&o, &player, &mut cache, &mut models, &kits, &items)?;
                rest_half = aim.unwrap_or_else(|| half_height(&bare));
                colors = m.light(64, 850, [-30, -50, -30])?;
                mode = draw.unwrap_or(actor_draw::Draw::Actor);
            }
            if let (Some(path), 0, 0) = (&dump, k, r) {
                // the assembled rest mesh of the first entry: one line per vertex, cache space, with its label
                let skins = m.vertex_skins.clone().unwrap_or_else(|| vec![0; m.vertices.len()]);
                let text: String = m.vertices.iter().zip(&skins).map(|(v, s)| format!("{} {} {} {}
", v[0] as i32, v[1] as i32, v[2] as i32, s)).collect();
                std::fs::write(path, text)?;
            }
            // ---- pose
            let mut note = String::from("bind");
            if let (Some(s), Some(f)) = (&sequence, frame) {
                let id = *s.frames.get(*f).ok_or(format!("sequence has {} frames, asked for {f}", s.frames.len()))?;
                let (base, fr, from) = source.frame(&mut cache, id, true)?;
                note = format!("classic frame {}:{} from {from}, base {}, {} transforms", id >> 16, id & 65535, base.id, fr.transforms.len());
                match (under, &s.interleave) {
                    (Some((us, uf)), Some(mask)) => {
                        // animation_runtime.rs apply_blended, two classic sequences
                        let b = source.sequences.get(&us).ok_or(format!("no sequence {us}"))?.clone();
                        let bid = *b.frames.get(uf).ok_or("under frame")?;
                        let (ubase, ufr, _) = source.frame(&mut cache, bid, false)?;
                        m.animate_classic_mask(&base, &fr, Some(mask), false, false)?;
                        m.animate_classic_mask(&ubase, &ufr, Some(mask), true, false)?;
                        note += &format!("; blended over {us}:{uf}");
                    }
                    (Some(_), None) => {
                        m.animate_classic(&base, &fr, false)?;
                        note += "; no interleave list, so the under sequence is ignored (as the client does)";
                    }
                    (None, _) => m.animate_classic(&base, &fr, false)?,
                }
            }
            if let Some(d) = npc_def { npc_model::resize(&mut m, d); }
            let label = frame.map_or("bind".to_string(), |f| f.to_string());
            println!("{e} frame {label}: {note}; {} vertices {} faces", m.vertices.len(), m.faces.len());
            if !track.is_empty() {
                if let Some(skins) = &m.vertex_skins {
                    for &l in &track {
                        let (mut n, mut sum) = (0usize, [0f64; 3]);
                        for (i, &s) in skins.iter().enumerate() {
                            if s == l { n += 1; for x in 0..3 { sum[x] += m.vertices[i][x] as f64; } }
                        }
                        if n > 0 {
                            println!("track entry {k} frame {label} label {l} n {n} x {:.1} h {:.1} z {:.1}", sum[0] / n as f64, -sum[1] / n as f64, sum[2] / n as f64);
                        }
                    }
                }
            }
            let mut placed = m.clone();
            for v in &mut placed.vertices { v[1] += rest_half as f32; }
            let vertices = interface_model::vertices(&placed, [pitch, yaw, 0], [0, 0], zoom, false)?;
            let origin = actor_draw::origin_depth(rest_half, pitch, zoom);
            let (sprite, warning) = painter.draw(&mut cache, &icons, &m, &colors, &vertices, origin, [cw, ch], mode)?;
            if let Some(w) = warning { println!("{e} frame {label}: WARNING {w}"); }
            sprite.draw(&mut raster, (k * cw) as i32, (r * ch) as i32, 256)?;
        }
    }
    std::fs::write(&a[2], raster.bmp()?)?;
    Ok(())
}
