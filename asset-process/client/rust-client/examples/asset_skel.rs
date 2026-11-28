//! Asset system tool (research-skeleton): one NPC, real or OURS FROM FILES, posed at frames of a classic sequence that
//! may come from the cache or from a LOOSE directory holding its own frame base. This is how a creature with a new
//! skeleton and its own animations is seen before anything is packed.
//! usage: asset_skel CACHE OUT.bmp [anim=DIR | seq=ID] [frames=F1,F2,..|all] [zoom=N] [yaw=N] [pitch=N] [cell=WxH]
//!                   [aim=HEIGHT] [draw=auto|actor|depth] [track=L1,L2,..] [dump=PATH] ENTRY
//! ENTRY = npc:ID     a creature of the cache (npc_model::mesh)
//!       | def:PATH   one of ours: asset_npc's text definition (name=, models=A|B (files or cache:ID), recolor=,
//!                    scale=XZ,Y, ambient=, contrast= (as stored), zbuffer=); other keys are ignored here
//! anim=DIR  DIR/seq.dat, DIR/frame_G_F.dat, DIR/base_B.dat (the frame base; from the cache when absent)
//! The strip is one row: one cell per frame. Camera, lighting, scaling and painting are asset_npc's `draw`
//! (light(64 + ambient, 850 + contrast, (-30,-50,-30)) at rest, pose, scale, Mode::Auto unless zbuffer=0), so a cell
//! here and a cell of asset_npc with the same zoom / yaw / pitch / cell / aim are the same picture when the bytes agree.
//! Posing: the client's Sequence::decode, Skeleton::decode, Frame::decode and Model::animate_classic.
//! track= prints per frame the centroid (x, height, z; after scale) of the posed vertices of each listed label;
//! every frame also prints the posed box. dump= writes the rest mesh (x y z label per line, cache space).
#[allow(dead_code)]
#[path = "asset_pose.rs"]
mod pose;
use gielinor_client::{
    animation::{Frame, Sequence, Skeleton}, disk_cache::DiskCache, interface_model, item_icon::Icons, item_model::Models,
    model::Model, npc, npc_model, raster::Raster,
};
use pose::actor_draw::{self, Draw};
use pose::R;
use std::collections::HashMap;
use std::path::PathBuf;

fn pairs(v: &str) -> R<Vec<(u16, u16)>> {
    let mut out = Vec::new();
    for p in v.split(',').filter(|p| !p.trim().is_empty()) {
        let (a, b) = p.trim().split_once(':').ok_or("pairs are FROM:TO")?;
        out.push((a.trim().parse()?, b.trim().parse()?));
    }
    Ok(out)
}
fn main() -> R<()> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() < 4 { return Err("CACHE OUT.bmp [anim=DIR | seq=ID] [frames=..] [zoom= yaw= pitch= cell= aim= draw= track= dump=] ENTRY".into()); }
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    let (mut zoom, mut yaw, mut pitch) = (520i32, 192i32, 0i32);
    let mut seq: Option<u32> = None;
    let mut dir: Option<PathBuf> = None;
    let mut frames_arg = String::new();
    let mut cell = (300usize, 400usize);
    let mut aim: Option<i32> = None;
    let mut draw: Option<Draw> = None;
    let mut track: Vec<u8> = Vec::new();
    let mut dump: Option<String> = None;
    let mut entry = String::new();
    for e in &a[3..] {
        if let Some(v) = e.strip_prefix("zoom=") { zoom = v.parse()?; }
        else if let Some(v) = e.strip_prefix("yaw=") { yaw = v.parse()?; }
        else if let Some(v) = e.strip_prefix("pitch=") { pitch = v.parse()?; }
        else if let Some(v) = e.strip_prefix("seq=") { seq = Some(v.parse()?); }
        else if let Some(v) = e.strip_prefix("anim=") { dir = Some(PathBuf::from(v)); }
        else if let Some(v) = e.strip_prefix("frames=") { frames_arg = v.to_string(); }
        else if let Some(v) = e.strip_prefix("cell=") {
            let (w, h) = v.split_once('x').ok_or("cell=WxH")?;
            cell = (w.parse()?, h.parse()?);
        }
        else if let Some(v) = e.strip_prefix("aim=") { aim = Some(v.parse()?); }
        else if let Some(v) = e.strip_prefix("draw=") { draw = Some(Draw::parse(v)?); }
        else if let Some(v) = e.strip_prefix("track=") { for l in v.split(',') { track.push(l.parse()?); } }
        else if let Some(v) = e.strip_prefix("dump=") { dump = Some(v.to_string()); }
        else { entry = e.clone(); }
    }
    let mut models = Models::default();
    // ---- the figure: rest mesh, light, scale, painting mode
    let (model, ambient, contrast, scale, z_buffer, label);
    if let Some(id) = entry.strip_prefix("npc:") {
        let defs = npc::Definition::load_all(&mut cache)?;
        let d = defs.get(&id.parse::<u32>()?).ok_or(format!("no NPC {id}"))?;
        model = npc_model::mesh(&mut cache, &mut models, d, false, None)?.ok_or("npc has no model")?;
        ambient = d.ambient; contrast = d.contrast; scale = d.scale; z_buffer = d.z_buffer;
        label = format!("npc {id} {:?} animations {:?}", d.name, d.animations);
    } else if let Some(path) = entry.strip_prefix("def:") {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let base = std::path::Path::new(path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let (mut name, mut files, mut recolor) = (String::from("ours"), Vec::<String>::new(), Vec::new());
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
                parts.push((*models.raw(&mut cache, id.parse()?)?).clone());
            } else {
                let p = if std::path::Path::new(file).is_absolute() { PathBuf::from(file) } else { base.join(file) };
                parts.push(Model::decode(&std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?)?);
            }
        }
        if parts.is_empty() { return Err(format!("{path}: no models").into()); }
        let mut m = if parts.len() == 1 { parts.remove(0) } else { Model::combine(&parts)? };
        for &(from, to) in &recolor { m.recolor(from, to); }
        model = m; ambient = am; contrast = co; scale = sc; z_buffer = zb;
        label = format!("def {name:?}");
    } else {
        return Err(format!("bad entry `{entry}`: use npc:ID or def:PATH").into());
    }
    if let Some(path) = &dump {
        let skins = model.vertex_skins.clone().unwrap_or_else(|| vec![0; model.vertices.len()]);
        let text: String = model.vertices.iter().zip(&skins).map(|(v, s)| format!("{} {} {} {}\n", v[0] as i32, v[1] as i32, v[2] as i32, s)).collect();
        std::fs::write(path, text)?;
    }
    let rest_height = {
        let mut m = model.clone();
        m.scale([scale[0] as i32, scale[1] as i32, scale[0] as i32]);
        m.vertices.iter().map(|v| -(v[1] as i32)).max().unwrap_or(0).max(0)
    };
    // ---- the sequence
    let sequence: Option<Sequence> = match (&dir, seq) {
        (Some(d), _) => Some(Sequence::decode(seq.unwrap_or(65535), &std::fs::read(d.join("seq.dat")).map_err(|e| format!("{}: {e}", d.join("seq.dat").display()))?)?),
        (None, Some(id)) => Some(Sequence::load_all(&mut cache)?.get(&id).ok_or(format!("no sequence {id}"))?.clone()),
        (None, None) => None,
    };
    let frames: Vec<Option<usize>> = match &sequence {
        None => vec![None],
        Some(s) => {
            if s.cached_animation.is_some() { return Err("this tool poses classic sequences only".into()); }
            if frames_arg.is_empty() || frames_arg == "all" { (0..s.frames.len()).map(Some).collect() }
            else { let mut v = Vec::new(); for f in frames_arg.split(',') { v.push(if f == "bind" { None } else { Some(f.parse::<usize>()?.min(s.frames.len() - 1)) }); } v }
        }
    };
    println!("{entry}: {label}; {} vertices {} faces; rest height {rest_height} (after scale {scale:?}); ambient {ambient} contrast {contrast} z_buffer {z_buffer}", model.vertices.len(), model.faces.len());
    if let Some(s) = &sequence {
        println!("seq {} ({}): frames={} lengths={:?} total={} loop={} priority={} max_loops={}", s.id, if dir.is_some() { "loose files" } else { "cache" },
            s.frames.len(), s.lengths, s.total_length, s.loop_frames, s.priority, s.max_loops);
    }
    let mut bases: HashMap<u16, Skeleton> = HashMap::new();
    let icons = Icons::load(&mut cache, 0.8)?;
    let mut painter = actor_draw::Painter::load(&mut cache)?;
    let colors = model.light(64 + ambient, 850 + contrast, [-30, -50, -30])?;
    let mode = draw.unwrap_or(if z_buffer { Draw::Auto } else { Draw::Actor });
    let rest_half = aim.unwrap_or(rest_height / 2);
    let (cw, ch) = cell;
    let mut raster = Raster::new(cw * frames.len(), ch, 1 << 27)?;
    raster.clear(0xff302b25);
    for (k, frame) in frames.iter().enumerate() {
        let mut m = model.clone();
        let mut note = String::from("rest pose");
        if let (Some(s), Some(f)) = (&sequence, frame) {
            let id = s.frames[*f];
            let (group, file) = (id >> 16, id & 65535);
            let loose = dir.as_ref().map(|d| d.join(format!("frame_{group}_{file}.dat"))).filter(|p| p.exists());
            let (data, from) = match &loose {
                Some(p) => (std::fs::read(p)?, "file"),
                None => {
                    let files = cache.group(0, group, [0; 4])?;
                    (files.iter().find(|(x, _)| *x == file).ok_or("animation frame missing")?.1.clone(), "cache")
                }
            };
            let sid = u16::from_be_bytes(data.get(..2).ok_or("short frame")?.try_into().unwrap());
            if !bases.contains_key(&sid) {
                let lb = dir.as_ref().map(|d| d.join(format!("base_{sid}.dat"))).filter(|p| p.exists());
                let bytes = match &lb {
                    Some(p) => { println!("base {sid}: from {}", p.display()); std::fs::read(p)? }
                    None => cache.file(1, sid as u32, 0, [0; 4])?,
                };
                bases.insert(sid, Skeleton::decode(sid, &bytes)?);
            }
            let base = &bases[&sid];
            let fr = Frame::decode(&data, base)?;
            m.animate_classic(base, &fr, false)?;
            note = format!("frame {f} len {}: classic {group}:{file} from {from}, base {sid}, {} transforms", s.lengths[*f], fr.transforms.len());
        }
        m.scale([scale[0] as i32, scale[1] as i32, scale[0] as i32]);
        let (mut lo, mut hi) = ([i32::MAX; 3], [i32::MIN; 3]);
        for v in &m.vertices { for x in 0..3 { lo[x] = lo[x].min(v[x] as i32); hi[x] = hi[x].max(v[x] as i32); } }
        println!("cell {k}: {note}; box min {lo:?} max {hi:?}");
        if !track.is_empty() {
            if let Some(skins) = &m.vertex_skins {
                for &l in &track {
                    let (mut n, mut sum) = (0usize, [0f64; 3]);
                    for (i, &s) in skins.iter().enumerate() {
                        if s == l { n += 1; for x in 0..3 { sum[x] += m.vertices[i][x] as f64; } }
                    }
                    if n > 0 { println!("track cell {k} label {l} n {n} x {:.1} h {:.1} z {:.1}", sum[0] / n as f64, -sum[1] / n as f64, sum[2] / n as f64); }
                }
            }
        }
        let mut placed = m.clone();
        for v in &mut placed.vertices { v[1] += rest_half as f32; }
        let vertices = interface_model::vertices(&placed, [pitch, yaw, 0], [0, 0], zoom, false)?;
        let origin = actor_draw::origin_depth(rest_half, pitch, zoom);
        let (sprite, warning) = painter.draw(&mut cache, &icons, &m, &colors, &vertices, origin, [cw, ch], mode)?;
        if let Some(w) = warning { println!("cell {k}: WARNING {w}"); }
        sprite.draw(&mut raster, (k * cw) as i32, 0, 256)?;
    }
    std::fs::write(&a[2], raster.bmp()?)?;
    Ok(())
}
