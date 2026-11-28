//! Asset system tool (research-bosses): a SKELETAL (bone-weighted, keyframe-animated) creature, real or OURS FROM
//! FILES, posed at frames of a keyframe animation that comes from the cache (by sequence id, through the client's
//! ordinary loader) or from LOOSE FILES (a frame base with its bone section, and an archive-22 keyframe file).
//! usage: asset_bone CACHE merged ENTRY OUT.txt
//!        asset_bone CACHE grid OUT.bmp [seq=ID] [base=FILE] [anim=FILE] [count=N] [frames=all|every:K|F1,F2,..]
//!                   [cols=N] [zoom=N] [yaw=N] [pitch=N] [cell=WxH] [aim=HEIGHT] [draw=auto|actor|depth] ENTRY
//! ENTRY = npc:ID | def:PATH  (asset_npc's text definition; models= files or cache:ID)
//! merged: the entry's assembled rest mesh as text: `v x y z label nw bone weight bone weight ..` and
//!         `f a b c colour priority flat depth_offset alpha face_label` (asset_npc's `merged` plus the weights).
//! grid:   one cell per frame, `cols` cells to a row. Which posing path is used:
//!         seq=ID alone            the client's Animations::apply_observed (cache loaders; classic or keyframe);
//!         anim=FILE [base=FILE]   Skeleton::decode + CachedAnimation::decode + Rig::new + Model::animate_cached on the
//!                                 files (base from the cache, by the id the keyframe file names, when base= is absent);
//!                                 the frame count is count=N, else seq=ID's range, else the file's own range.
//!         Frame k is time k (apply_observed passes `clock.frame as f32`; one frame = one client cycle).
//! Lighting, scaling and painting are asset_npc's: light(64 + ambient, 850 + contrast, (-30,-50,-30)) ONCE at rest,
//! pose a copy, scale, Mode::Auto unless zbuffer=0. Every cell prints an FNV-1a hash of the posed vertices' f32 bits.
#[allow(dead_code)]
#[path = "asset_pose.rs"]
mod pose;
use gielinor_client::{
    actor_sequence::State, animation::Skeleton, animation_runtime::{Animations, Applied}, cached_animation::CachedAnimation,
    disk_cache::DiskCache, interface_model, item_icon::Icons, item_model::Models, model::Model, npc, npc_model, raster::Raster,
    skeletal::Rig,
};
use pose::actor_draw::{self, Draw};
use pose::R;
use std::fmt::Write as _;
use std::path::PathBuf;

fn pairs(v: &str) -> R<Vec<(u16, u16)>> {
    let mut out = Vec::new();
    for p in v.split(',').filter(|p| !p.trim().is_empty()) {
        let (a, b) = p.trim().split_once(':').ok_or("pairs are FROM:TO")?;
        out.push((a.trim().parse()?, b.trim().parse()?));
    }
    Ok(out)
}
struct Figure { model: Model, ambient: i32, contrast: i32, scale: [u16; 2], z_buffer: bool, label: String }
fn figure(entry: &str, cache: &mut DiskCache, models: &mut Models) -> R<Figure> {
    if let Some(id) = entry.strip_prefix("npc:") {
        let defs = npc::Definition::load_all(cache)?;
        let d = defs.get(&id.parse::<u32>()?).ok_or(format!("no NPC {id}"))?;
        let model = npc_model::mesh(cache, models, d, false, None)?.ok_or("npc has no model")?;
        Ok(Figure { model, ambient: d.ambient, contrast: d.contrast, scale: d.scale, z_buffer: d.z_buffer, label: format!("npc {id} {:?}", d.name) })
    } else if let Some(path) = entry.strip_prefix("def:") {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let base = std::path::Path::new(path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let (mut name, mut files, mut recolor, mut retexture) = (String::from("ours"), Vec::<String>::new(), Vec::new(), Vec::new());
        let (mut sc, mut am, mut co, mut zb) = ([128u16; 2], 0i32, 0i32, true);
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            let (k, v) = line.split_once('=').ok_or(format!("{path}: `{line}` is not key=value"))?;
            let (k, v) = (k.trim(), v.trim());
            match k {
                "name" => name = v.to_string(),
                "models" => files = v.split('|').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                "recolor" => recolor = pairs(v)?,
                "retexture" => retexture = pairs(v)?,
                "scale" => { let (x, y) = v.split_once(',').ok_or("scale=XZ,Y")?; sc = [x.trim().parse()?, y.trim().parse()?]; }
                "ambient" => am = v.parse()?,
                "contrast" => co = v.parse::<i32>()? * 5,
                "zbuffer" => zb = v != "0" && v != "false",
                _ => {}
            }
        }
        let mut parts = Vec::new();
        for file in &files {
            if let Some(id) = file.strip_prefix("cache:") {
                parts.push((*models.raw(cache, id.parse()?)?).clone());
            } else {
                let p = if std::path::Path::new(file).is_absolute() { PathBuf::from(file) } else { base.join(file) };
                parts.push(Model::decode(&std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?)?);
            }
        }
        if parts.is_empty() { return Err(format!("{path}: no models").into()); }
        let mut m = if parts.len() == 1 { parts.remove(0) } else { Model::combine(&parts)? };
        for &(from, to) in &recolor { m.recolor(from, to); }
        for &(from, to) in &retexture { m.retexture(from, to); }
        Ok(Figure { model: m, ambient: am, contrast: co, scale: sc, z_buffer: zb, label: format!("def {name:?}") })
    } else {
        Err(format!("bad entry `{entry}`: use npc:ID or def:PATH").into())
    }
}
fn fnv(m: &Model) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for v in &m.vertices { for c in v { for b in c.to_bits().to_be_bytes() { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); } } }
    h
}
fn main() -> R<()> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() < 5 { return Err("CACHE merged ENTRY OUT.txt | CACHE grid OUT.bmp [seq= base= anim= count= frames= cols= zoom= yaw= pitch= cell= aim= draw=] ENTRY".into()); }
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    let mut models = Models::default();
    if a[2] == "merged" {
        let f = figure(&a[3], &mut cache, &mut models)?;
        let m = &f.model;
        let mut t = String::new();
        writeln!(t, "# {} : {} vertices {} faces; weights {}; v x y z label nw (bone weight).. ; f a b c colour priority flat depth_offset alpha face_label", f.label, m.vertices.len(), m.faces.len(), m.bone_weights.is_some())?;
        for (i, v) in m.vertices.iter().enumerate() {
            write!(t, "v {} {} {} {}", v[0] as i32, v[1] as i32, v[2] as i32, m.vertex_skins.as_ref().map_or(0, |s| s[i]))?;
            let w = m.bone_weights.as_ref().map(|w| w[i].clone()).unwrap_or_default();
            write!(t, " {}", w.len())?;
            for (b, x) in w { write!(t, " {b} {x}")?; }
            writeln!(t)?;
        }
        for (i, face) in m.faces.iter().enumerate() {
            writeln!(t, "f {} {} {} {} {} {} {} {} {}", face[0], face[1], face[2], m.colors[i], m.priorities.as_ref().map_or(m.priority, |p| p[i]),
                m.render_types.as_ref().map_or(0, |r| r[i]), m.face_flags.as_ref().map_or(0, |r| r[i] as i8), m.alpha.as_ref().map_or(0, |r| r[i]), m.face_skins.as_ref().map_or(0, |r| r[i]))?;
        }
        std::fs::write(&a[4], t)?;
        println!("{}: {} vertices {} faces", f.label, m.vertices.len(), m.faces.len());
        return Ok(());
    }
    if a[2] != "grid" { return Err("merged | grid".into()); }
    let (mut zoom, mut yaw, mut pitch) = (520i32, 192i32, 0i32);
    let (mut seq, mut base_file, mut anim_file, mut count_arg): (Option<u32>, Option<String>, Option<String>, Option<usize>) = (None, None, None, None);
    let mut frames_arg = String::from("all");
    let mut cols = 16usize;
    let mut cell = (300usize, 400usize);
    let mut aim: Option<i32> = None;
    let mut draw: Option<Draw> = None;
    let mut entry = String::new();
    for e in &a[4..] {
        if let Some(v) = e.strip_prefix("zoom=") { zoom = v.parse()?; }
        else if let Some(v) = e.strip_prefix("yaw=") { yaw = v.parse()?; }
        else if let Some(v) = e.strip_prefix("pitch=") { pitch = v.parse()?; }
        else if let Some(v) = e.strip_prefix("seq=") { seq = Some(v.parse()?); }
        else if let Some(v) = e.strip_prefix("base=") { base_file = Some(v.to_string()); }
        else if let Some(v) = e.strip_prefix("anim=") { anim_file = Some(v.to_string()); }
        else if let Some(v) = e.strip_prefix("count=") { count_arg = Some(v.parse()?); }
        else if let Some(v) = e.strip_prefix("frames=") { frames_arg = v.to_string(); }
        else if let Some(v) = e.strip_prefix("cols=") { cols = v.parse()?; }
        else if let Some(v) = e.strip_prefix("cell=") { let (w, h) = v.split_once('x').ok_or("cell=WxH")?; cell = (w.parse()?, h.parse()?); }
        else if let Some(v) = e.strip_prefix("aim=") { aim = Some(v.parse()?); }
        else if let Some(v) = e.strip_prefix("draw=") { draw = Some(Draw::parse(v)?); }
        else { entry = e.clone(); }
    }
    let f = figure(&entry, &mut cache, &mut models)?;
    let model = &f.model;
    let rest_height = {
        let mut m = model.clone();
        m.scale([f.scale[0] as i32, f.scale[1] as i32, f.scale[0] as i32]);
        m.vertices.iter().map(|v| -(v[1] as i32)).max().unwrap_or(0).max(0)
    };
    println!("{entry}: {}; {} vertices {} faces; weights {}; rest height {rest_height}; scale {:?} ambient {} contrast {} z_buffer {}", f.label, model.vertices.len(), model.faces.len(), model.bone_weights.is_some(), f.scale, f.ambient, f.contrast, f.z_buffer);
    let mut animations = Animations::load(&mut cache)?;
    let sequence = match seq { Some(id) => Some(animations.sequences.get(&id).ok_or(format!("no sequence {id}"))?.clone()), None => None };
    // loose files
    let loose: Option<(Skeleton, CachedAnimation, Rig)> = match &anim_file {
        None => None,
        Some(p) => {
            let data = std::fs::read(p).map_err(|e| format!("{p}: {e}"))?;
            let sid = u16::from_be_bytes(data.get(1..3).ok_or("short keyframe file")?.try_into().unwrap());
            let bytes = match &base_file { Some(b) => std::fs::read(b).map_err(|e| format!("{b}: {e}"))?, None => cache.file(1, sid as u32, 0, [0; 4])? };
            let sk = Skeleton::decode(sid, &bytes)?;
            let an = CachedAnimation::decode(&data, &sk)?;
            let rig = Rig::new(&sk, an.pose)?;
            println!("loose keyframe file {p}: version {} base {sid} ({}) range {:?} pose {} channels kept {}; base has {} groups {} bones", an.version, if base_file.is_some() { "file" } else { "cache" }, an.range, an.pose, an.channels.len(), sk.transform_types.len(), sk.bones.len());
            Some((sk, an, rig))
        }
    };
    let count = match (count_arg, &sequence, &loose) {
        (Some(n), _, _) => n,
        (None, Some(s), _) => if s.cached_animation.is_some() { s.cached_range[1].saturating_sub(s.cached_range[0]) as usize } else { s.frames.len() },
        (None, None, Some((_, an, _))) => an.range[1].saturating_sub(an.range[0]) as usize,
        _ => 0,
    };
    if let Some(s) = &sequence { println!("seq {}: cached {:?} range {:?} classic frames {} loop {} max_loops {} => {count} frames", s.id, s.cached_animation, s.cached_range, s.frames.len(), s.loop_frames, s.max_loops); }
    let frames: Vec<Option<usize>> = if count == 0 { vec![None] }
        else if frames_arg == "all" { (0..count).map(Some).collect() }
        else if let Some(k) = frames_arg.strip_prefix("every:") { let k: usize = k.parse()?; (0..count).step_by(k.max(1)).map(Some).collect() }
        else { let mut v = Vec::new(); for x in frames_arg.split(',') { v.push(if x == "bind" { None } else { Some(x.parse::<usize>()?.min(count - 1)) }); } v };
    let icons = Icons::load(&mut cache, 0.8)?;
    let mut painter = actor_draw::Painter::load(&mut cache)?;
    let colors = model.light(64 + f.ambient, 850 + f.contrast, [-30, -50, -30])?;
    let mode = draw.unwrap_or(if f.z_buffer { Draw::Auto } else { Draw::Actor });
    let rest_half = aim.unwrap_or(rest_height / 2);
    let (cw, ch) = cell;
    let cols = cols.min(frames.len()).max(1);
    let rows = (frames.len() + cols - 1) / cols;
    let mut raster = Raster::new(cw * cols, ch * rows, 1 << 28)?;
    raster.clear(0xff302b25);
    for (k, frame) in frames.iter().enumerate() {
        let mut m = model.clone();
        let mut note = String::from("rest pose");
        if let Some(fr) = frame {
            if let Some((sk, an, rig)) = &loose {
                m.animate_cached(sk, rig, an, *fr as f32)?;
                note = format!("frame {fr}: LOOSE keyframe file, base {}", sk.id);
            } else if let Some(s) = &sequence {
                let state = State { id: s.id as i32, frame: *fr as i32, ..Default::default() };
                note = match animations.apply_observed(&mut cache, &mut m, s.id, &state.clock())? {
                    Applied::Classic { frame_id, skeleton_id, .. } => format!("frame {fr}: classic {}:{} base {skeleton_id}", frame_id >> 16, frame_id & 65535),
                    Applied::Cached { animation_id, skeleton_id, .. } => format!("frame {fr}: cache keyframe file {}:{} base {skeleton_id}", animation_id >> 16, animation_id & 65535),
                    other => format!("frame {fr}: NOT APPLIED {other:?}"),
                };
            }
        }
        m.scale([f.scale[0] as i32, f.scale[1] as i32, f.scale[0] as i32]);
        let (mut lo, mut hi) = ([i32::MAX; 3], [i32::MIN; 3]);
        for v in &m.vertices { for x in 0..3 { lo[x] = lo[x].min(v[x] as i32); hi[x] = hi[x].max(v[x] as i32); } }
        println!("cell {k}: {note}; vertices {:016x}; box min {lo:?} max {hi:?}", fnv(&m));
        let mut placed = m.clone();
        for v in &mut placed.vertices { v[1] += rest_half as f32; }
        let vertices = interface_model::vertices(&placed, [pitch, yaw, 0], [0, 0], zoom, false)?;
        let origin = actor_draw::origin_depth(rest_half, pitch, zoom);
        let (sprite, warning) = painter.draw(&mut cache, &icons, &m, &colors, &vertices, origin, [cw, ch], mode)?;
        if let Some(w) = warning { println!("cell {k}: WARNING {w}"); }
        sprite.draw(&mut raster, ((k % cols) * cw) as i32, ((k / cols) * ch) as i32, 256)?;
    }
    std::fs::write(&a[3], raster.bmp()?)?;
    Ok(())
}
