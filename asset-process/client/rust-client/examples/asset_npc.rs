//! Asset system tool for NPCs (research-npc): census of the cache's NPC definitions, and a harness that draws a
//! real NPC, an NPC of ours given as model FILES plus a small definition, or a player figure, in the rest pose
//! and at chosen frames of stand / walk / attack / death sequences, lit and painted as the live client does.
//!
//! usage: asset_npc CACHE census OUTDIR
//!        asset_npc CACHE dump ID OUTDIR            the real NPC's raw model files + OUTDIR/npc-ID.npcdef
//!        asset_npc CACHE merged ENTRY OUT.txt      the entry's assembled rest mesh as text (v/f lines)
//!        asset_npc CACHE draw OUT.bmp [rows=R1,R2..] [zoom=N] [yaw=N] [pitch=N] [cell=WxH] [aim=H|own]
//!                                      [draw=def|auto|actor|depth] ENTRY [ENTRY...]
//! ENTRY = npc:ID[;attack=SEQ][;death=SEQ][;NAME=SEQ]   a real NPC, built by the client's own npc_model::mesh
//!       | def:PATH[;NAME=SEQ]                           one of ours: a text definition (see below)
//!       | player:OUTFIT[;stand=SEQ][;walk=SEQ][;NAME=SEQ]  a player figure; OUTFIT as in asset_pose (`none`, item:..)
//! ROW   = bind | NAME:FRAME (stand, walk, attack, death, or any NAME the entry defines) | seq:ID:FRAME
//!         An entry that has no sequence of that NAME is drawn in the rest pose and reported.
//!
//! Definition file (one `key=value` per line; `#` comments):
//!   name=TEXT   models=A|B|C (each a model FILE path, or cache:ID)   recolor=FROM:TO,FROM:TO   retexture=..
//!   scale=XZ,Y (128 = natural)   ambient=N   contrast=N (as stored: the client multiplies by 5)   size=N
//!   zbuffer=1|0 (0 = opcode 147: painted by priority like a player)   stand=SEQ walk=SEQ attack=SEQ death=SEQ
//!
//! What the live client does with an NPC (src/world_actors.rs npc_triangles, the ordinary branch):
//!   npc_model::mesh  -> every model id decoded, Model::combine (one model; equal coordinates merged), then the
//!                       definition's recolours and retextures;
//!   model.light(64 + ambient, 850 + contrast, [-30, -50, -30])   once, in the rest pose (cached);
//!   Animations::apply_actor on a copy; npc_model::resize (scale) AFTER animation;
//!   place_actor(.., Mode::actor(false, def.z_buffer, use_player)): Mode::Auto unless the definition carries
//!   opcode 147 (z_buffer = false), in which case Mode::Priority, the player's painting.
//! This file repeats exactly those steps; for `npc:ID` it calls npc_model::mesh itself, for `def:` it decodes the
//! files and calls the same Model::combine / recolor / light / apply / scale. Painting is examples/support/
//! actor_draw.rs (the client's Frame::draw with the same Triangle and Submission the world path builds).
#[allow(dead_code)]
#[path = "asset_pose.rs"]
mod pose;
use gielinor_client::{
    actor_sequence::State, animation::{Sequence, Skeleton}, animation_runtime::{Animations, Applied}, disk_cache::DiskCache,
    identity_kit::Kit, interface_model, item::Definitions, item_icon::Icons, item_model::Models, model::Model,
    npc::Definition, npc_model, raster::Raster,
};
use pose::actor_draw::{self, Draw};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
type R<T> = Result<T, Box<dyn std::error::Error>>;

fn js(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if c.is_ascii() && !c.is_ascii_control() => o.push(c),
            _ => o.push('?'),
        }
    }
    o.push('"');
    o
}
fn list<T: std::fmt::Display>(v: impl IntoIterator<Item = T>) -> String {
    format!("[{}]", v.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
}
fn model_stats(id: u32, m: &Model) -> String {
    let (mut lo, mut hi) = ([i32::MAX; 3], [i32::MIN; 3]);
    for v in &m.vertices {
        for x in 0..3 { lo[x] = lo[x].min(v[x] as i32); hi[x] = hi[x].max(v[x] as i32); }
    }
    if m.vertices.is_empty() { lo = [0; 3]; hi = [0; 3]; }
    let labels: BTreeSet<u8> = m.vertex_skins.as_ref().map_or(BTreeSet::new(), |s| s.iter().copied().collect());
    let mut prio = BTreeMap::<u8, usize>::new();
    match &m.priorities {
        Some(p) => for &x in p { *prio.entry(x).or_default() += 1; },
        None => { prio.insert(m.priority, m.faces.len()); }
    }
    let colours: BTreeSet<u16> = m.colors.iter().copied().collect();
    format!(
        "{{\"id\":{id},\"verts\":{},\"faces\":{},\"labels\":{},\"has_labels\":{},\"weights\":{},\"prio\":{{{}}},\"depth_offsets\":{},\"alpha\":{},\"textures\":{},\"flat\":{},\"min\":{},\"max\":{},\"ncolours\":{}}}",
        m.vertices.len(), m.faces.len(), list(labels.iter()), m.vertex_skins.is_some(), m.bone_weights.is_some(),
        prio.iter().map(|(k, v)| format!("\"{k}\":{v}")).collect::<Vec<_>>().join(","),
        m.face_flags.as_ref().map_or(0, |f| f.iter().filter(|&&x| x != 0).count()),
        m.alpha.as_ref().map_or(0, |f| f.iter().filter(|&&x| x != 0).count()),
        m.face_textures.as_ref().map_or(0, |f| f.iter().filter(|&&x| x != -1).count()),
        m.render_types.as_ref().map_or(0, |f| f.iter().filter(|&&x| x & 1 != 0).count()),
        list(lo), list(hi), colours.len()
    )
}
fn census(cache: &mut DiskCache, dir: &str) -> R<()> {
    std::fs::create_dir_all(dir)?;
    let defs = Definition::load_all(cache)?;
    let mut ids = defs.keys().copied().collect::<Vec<_>>();
    ids.sort();
    let mut out = String::new();
    let mut wanted = BTreeSet::<u32>::new();
    for id in &ids {
        let d = &defs[id];
        wanted.extend(d.models.iter().copied().filter(|&m| m != u32::MAX));
        writeln!(out, "{{\"id\":{id},\"name\":{},\"size\":{},\"combat\":{},\"models\":{},\"heads\":{},\"anims\":{},\"recolor\":{},\"retexture\":{},\"scale\":{},\"ambient\":{},\"contrast\":{},\"zbuffer\":{},\"priority\":{},\"transforms\":{},\"follower\":{},\"interactable\":{},\"ops\":{}}}",
            js(&d.name), d.size, d.combat, list(d.models.iter().map(|&m| m as i64)), list(d.head_models.iter().map(|&m| m as i64)), list(d.animations),
            list(d.recolor.iter().map(|(a, b)| format!("[{a},{b}]"))), d.retexture.len(), list(d.scale), d.ambient, d.contrast, d.z_buffer, d.priority,
            d.transforms.is_some(), d.follower, d.interactable,
            list(d.operations.iter().map(|o| js(&o.text))))?;
    }
    std::fs::write(format!("{dir}/npcs.jsonl"), &out)?;
    println!("npcs {}", ids.len());
    // kits and worn item models
    let kits = Kit::load_all(cache)?;
    let mut out = String::new();
    let mut kids = kits.keys().copied().collect::<Vec<_>>();
    kids.sort();
    for k in kids {
        let kit = &kits[&k];
        wanted.extend(kit.models.iter().copied());
        writeln!(out, "{{\"id\":{k},\"part\":{},\"disabled\":{},\"models\":{},\"heads\":{}}}", kit.body_part, kit.disabled, list(kit.models.iter()), list(kit.heads))?;
    }
    std::fs::write(format!("{dir}/kits.jsonl"), &out)?;
    let items = Definitions::load(cache)?;
    let mut out = String::new();
    let mut iids = items.items.keys().copied().collect::<Vec<_>>();
    iids.sort();
    for i in iids {
        let d = &items.items[&i];
        if d.wear_models[0][0] == -1 && d.wear_models[1][0] == -1 { continue; }
        writeln!(out, "{{\"id\":{i},\"name\":{},\"wear\":{},\"male\":{},\"female\":{}}}", js(&d.name), list(d.wear_positions), list(d.wear_models[0]), list(d.wear_models[1]))?;
    }
    std::fs::write(format!("{dir}/items.jsonl"), &out)?;
    // models
    let mut models = Models::default();
    let mut out = String::new();
    let mut missing = 0;
    for &m in &wanted {
        match models.raw(cache, m) {
            Ok(model) => writeln!(out, "{}", model_stats(m, &model))?,
            Err(_) => missing += 1,
        }
    }
    std::fs::write(format!("{dir}/models.jsonl"), &out)?;
    println!("models {} missing {missing}", wanted.len());
    // sequences -> frame base (skeleton)
    let sequences = Sequence::load_all(cache)?;
    let mut sids = sequences.keys().copied().collect::<Vec<_>>();
    sids.sort();
    let mut out = String::new();
    let mut group_skeleton = HashMap::<u32, i32>::new();
    let mut skeletons = BTreeSet::<u16>::new();
    for s in sids {
        let q = &sequences[&s];
        let (kind, sk, n) = if let Some(a) = q.cached_animation {
            let sk = cache.file(22, a >> 16, a & 65535, [0; 4]).ok().and_then(|d| d.get(1..3).map(|b| u16::from_be_bytes([b[0], b[1]]) as i32)).unwrap_or(-1);
            ("cached", sk, q.cached_range[1].saturating_sub(q.cached_range[0]) as usize)
        } else if let Some(&f) = q.frames.first() {
            let g = f >> 16;
            let sk = *group_skeleton.entry(g).or_insert_with(|| {
                cache.group(0, g, [0; 4]).ok().and_then(|files| files.first().and_then(|(_, d)| d.get(..2).map(|b| u16::from_be_bytes([b[0], b[1]]) as i32))).unwrap_or(-1)
            });
            ("classic", sk, q.frames.len())
        } else { ("empty", -1, 0) };
        if sk >= 0 { skeletons.insert(sk as u16); }
        writeln!(out, "{{\"id\":{s},\"kind\":\"{kind}\",\"skeleton\":{sk},\"frames\":{n},\"length\":{},\"weapon\":{},\"shield\":{},\"interleave\":{},\"priority\":{}}}", q.total_length, q.weapon, q.shield, q.interleave.is_some(), q.priority)?;
    }
    std::fs::write(format!("{dir}/sequences.jsonl"), &out)?;
    let mut out = String::new();
    for sk in skeletons {
        if let Ok(data) = cache.file(1, sk as u32, 0, [0; 4]) {
            if let Ok(s) = Skeleton::decode(sk, &data) {
                let labels: BTreeSet<u8> = s.labels.iter().flatten().copied().collect();
                writeln!(out, "{{\"id\":{sk},\"groups\":{},\"bones\":{},\"labels\":{}}}", s.transform_types.len(), s.bones.len(), list(labels.iter()))?;
            }
        }
    }
    std::fs::write(format!("{dir}/skeletons.jsonl"), &out)?;
    println!("sequences {}", sequences.len());
    Ok(())
}

/// One figure to draw: its rest mesh and how to light, scale and paint it.
struct Figure {
    label: String,
    model: Model,
    ambient: i32,
    contrast: i32,
    scale: [u16; 2],
    z_buffer: bool,
    player: bool,
    seqs: HashMap<String, u32>,
    notes: Vec<String>,
}
fn parse_pairs(v: &str) -> R<Vec<(u16, u16)>> {
    let mut out = Vec::new();
    for p in v.split(',').filter(|p| !p.trim().is_empty()) {
        let (a, b) = p.trim().split_once(':').ok_or("pairs are FROM:TO")?;
        out.push((a.trim().parse()?, b.trim().parse()?));
    }
    Ok(out)
}
fn figure(entry: &str, cache: &mut DiskCache, models: &mut Models, defs: &HashMap<u32, Definition>, kits: &HashMap<u32, Kit>, items: &Definitions) -> R<Figure> {
    let mut parts = entry.split(';');
    let head = parts.next().unwrap();
    let mut seqs = HashMap::new();
    let mut extra = Vec::new();
    for p in parts {
        let (k, v) = p.split_once('=').ok_or(format!("`{p}` in `{entry}`: use NAME=SEQ"))?;
        extra.push((k.to_string(), v.parse::<u32>()?));
    }
    let mut f = if let Some(id) = head.strip_prefix("npc:") {
        let id: u32 = id.parse()?;
        let d = defs.get(&id).ok_or(format!("no NPC {id}"))?;
        let model = npc_model::mesh(cache, models, d, false, None)?.ok_or(format!("NPC {id} has no models"))?;
        for (name, slot) in [("stand", 0usize), ("walk", 3)] {
            if d.animations[slot] >= 0 { seqs.insert(name.to_string(), d.animations[slot] as u32); }
        }
        Figure { label: format!("npc {id} {:?}", d.name), model, ambient: d.ambient, contrast: d.contrast, scale: d.scale, z_buffer: d.z_buffer, player: false, seqs,
                 notes: vec![format!("models {:?} recolor {:?} scale {:?} ambient {} contrast {} size {} z_buffer {}", d.models, d.recolor, d.scale, d.ambient, d.contrast, d.size, d.z_buffer)] }
    } else if let Some(path) = head.strip_prefix("def:") {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let base = std::path::Path::new(path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let (mut name, mut files, mut recolor, mut retexture) = (String::from("ours"), Vec::<String>::new(), Vec::new(), Vec::new());
        let (mut scale, mut ambient, mut contrast, mut z_buffer) = ([128u16; 2], 0i32, 0i32, true);
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            let (k, v) = line.split_once('=').ok_or(format!("{path}: `{line}` is not key=value"))?;
            let (k, v) = (k.trim(), v.trim());
            match k {
                "name" => name = v.to_string(),
                "models" => files = v.split('|').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                "recolor" => recolor = parse_pairs(v)?,
                "retexture" => retexture = parse_pairs(v)?,
                "scale" => { let (a, b) = v.split_once(',').ok_or("scale=XZ,Y")?; scale = [a.trim().parse()?, b.trim().parse()?]; }
                "ambient" => ambient = v.parse()?,
                "contrast" => contrast = v.parse::<i32>()? * 5,
                "zbuffer" => z_buffer = v != "0" && v != "false",
                "size" | "combat" | "note" | "heads" => {}
                _ => { seqs.insert(k.to_string(), v.parse::<u32>().map_err(|_| format!("{path}: `{line}`: unknown key, or a sequence name without a number"))?); }
            }
        }
        let mut parts = Vec::new();
        for file in &files {
            if let Some(id) = file.strip_prefix("cache:") {
                parts.push((*models.raw(cache, id.parse()?)?).clone());
            } else {
                let p = if std::path::Path::new(file).is_absolute() { std::path::PathBuf::from(file) } else { base.join(file) };
                parts.push(Model::decode(&std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?)?);
            }
        }
        if parts.is_empty() { return Err(format!("{path}: no models").into()); }
        // npc_model::mesh: a single part is used as it is, several are combined
        let mut model = if parts.len() == 1 { parts.remove(0) } else { Model::combine(&parts)? };
        for &(from, to) in &recolor { model.recolor(from, to); }
        for &(from, to) in &retexture { model.retexture(from, to); }
        Figure { label: format!("def {name:?}"), model, ambient, contrast, scale, z_buffer, player: false, seqs,
                 notes: vec![format!("models {files:?} recolor {recolor:?} scale {scale:?} ambient {ambient} contrast {contrast} z_buffer {z_buffer}")] }
    } else if let Some(spec) = head.strip_prefix("player:") {
        let standard = pose::standard_figure(kits);
        let o = pose::outfit(spec, &standard, items, models, cache)?;
        let mut player = standard.clone();
        player.equipment = o.equipment;
        let model = pose::assemble(&o, &player, cache, models, kits, items)?;
        seqs.insert("stand".into(), 808);
        seqs.insert("walk".into(), 819);
        Figure { label: format!("player {spec}"), model, ambient: 0, contrast: 0, scale: [128; 2], z_buffer: false, player: true, seqs, notes: o.notes }
    } else {
        return Err(format!("bad entry `{entry}`: use npc:ID, def:PATH or player:OUTFIT").into());
    };
    for (k, v) in extra { f.seqs.insert(k, v); }
    Ok(f)
}
fn rest_height(f: &Figure) -> i32 {
    let mut m = f.model.clone();
    m.scale([f.scale[0] as i32, f.scale[1] as i32, f.scale[0] as i32]);
    m.vertices.iter().map(|v| -(v[1] as i32)).max().unwrap_or(0).max(0)
}
fn dump(cache: &mut DiskCache, id: u32, dir: &str) -> R<()> {
    std::fs::create_dir_all(dir)?;
    let defs = Definition::load_all(cache)?;
    let d = defs.get(&id).ok_or("no such NPC")?;
    let mut names = Vec::new();
    for &m in &d.models {
        if m == u32::MAX { continue; }
        let data = cache.file(7, m, 0, [0; 4])?;
        std::fs::write(format!("{dir}/model-{m}.bin"), &data)?;
        names.push(format!("model-{m}.bin"));
    }
    let mut t = String::new();
    writeln!(t, "# real NPC {id} {:?}, written by asset_npc dump: its own model files and definition values", d.name)?;
    writeln!(t, "name={}", d.name)?;
    writeln!(t, "models={}", names.join("|"))?;
    writeln!(t, "recolor={}", d.recolor.iter().map(|(a, b)| format!("{a}:{b}")).collect::<Vec<_>>().join(","))?;
    writeln!(t, "retexture={}", d.retexture.iter().map(|(a, b)| format!("{a}:{b}")).collect::<Vec<_>>().join(","))?;
    writeln!(t, "scale={},{}", d.scale[0], d.scale[1])?;
    writeln!(t, "ambient={}", d.ambient)?;
    writeln!(t, "contrast={}", d.contrast / 5)?;
    writeln!(t, "size={}", d.size)?;
    writeln!(t, "zbuffer={}", u8::from(d.z_buffer))?;
    if d.animations[0] >= 0 { writeln!(t, "stand={}", d.animations[0])?; }
    if d.animations[3] >= 0 { writeln!(t, "walk={}", d.animations[3])?; }
    std::fs::write(format!("{dir}/npc-{id}.npcdef"), t)?;
    println!("{dir}/npc-{id}.npcdef: {} model files", names.len());
    Ok(())
}
fn main() -> R<()> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() < 4 {
        return Err("CACHE census OUTDIR | CACHE dump ID OUTDIR | CACHE merged ENTRY OUT.txt | CACHE draw OUT.bmp [options] ENTRY...".into());
    }
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    match a[2].as_str() {
        "census" => return census(&mut cache, &a[3]),
        "dump" => return dump(&mut cache, a[3].parse()?, &a[4]),
        _ => {}
    }
    let kits = Kit::load_all(&mut cache)?;
    let items = Definitions::load(&mut cache)?;
    let defs = Definition::load_all(&mut cache)?;
    let mut models = Models::default();
    if a[2] == "merged" {
        let f = figure(&a[3], &mut cache, &mut models, &defs, &kits, &items)?;
        let m = &f.model;
        let mut t = String::new();
        writeln!(t, "# {} : {} vertices {} faces; v x y z label ; f a b c colour priority flat depth_offset alpha face_label", f.label, m.vertices.len(), m.faces.len())?;
        for (i, v) in m.vertices.iter().enumerate() {
            writeln!(t, "v {} {} {} {}", v[0] as i32, v[1] as i32, v[2] as i32, m.vertex_skins.as_ref().map_or(0, |s| s[i]))?;
        }
        for (i, face) in m.faces.iter().enumerate() {
            writeln!(t, "f {} {} {} {} {} {} {} {} {}", face[0], face[1], face[2], m.colors[i], m.priorities.as_ref().map_or(m.priority, |p| p[i]),
                m.render_types.as_ref().map_or(0, |r| r[i]), m.face_flags.as_ref().map_or(0, |r| r[i] as i8), m.alpha.as_ref().map_or(0, |r| r[i]), m.face_skins.as_ref().map_or(0, |r| r[i]))?;
        }
        std::fs::write(&a[4], t)?;
        println!("{}: {} vertices {} faces", f.label, m.vertices.len(), m.faces.len());
        return Ok(());
    }
    if a[2] != "draw" { return Err("census | dump | merged | draw".into()); }
    let icons = Icons::load(&mut cache, 0.8)?;
    let mut animations = Animations::load(&mut cache)?;
    let (mut zoom, mut yaw, mut pitch) = (520i32, 192i32, 0i32);
    let mut cell: Option<(usize, usize)> = None;
    let mut aim: Option<String> = None;
    let mut draw: Option<Draw> = None;
    let mut rows: Vec<String> = Vec::new();
    let mut entries = Vec::new();
    for e in &a[4..] {
        if let Some(v) = e.strip_prefix("zoom=") { zoom = v.parse()?; }
        else if let Some(v) = e.strip_prefix("yaw=") { yaw = v.parse()?; }
        else if let Some(v) = e.strip_prefix("pitch=") { pitch = v.parse()?; }
        else if let Some(v) = e.strip_prefix("rows=") { rows = v.split(',').map(|s| s.to_string()).collect(); }
        else if let Some(v) = e.strip_prefix("cell=") {
            let (w, h) = v.split_once('x').ok_or("cell=WxH")?;
            cell = Some((w.parse()?, h.parse()?));
        }
        else if let Some(v) = e.strip_prefix("aim=") { aim = Some(v.to_string()); }
        else if let Some(v) = e.strip_prefix("draw=") { draw = if v == "def" { None } else { Some(Draw::parse(v)?) }; }
        else { entries.push(e.clone()); }
    }
    if entries.is_empty() { return Err("no entries".into()); }
    if rows.is_empty() { rows.push("bind".into()); }
    let (cw, ch) = cell.unwrap_or((300, 400));
    let mut painter = actor_draw::Painter::load(&mut cache)?;
    let mut figures = Vec::new();
    for e in &entries {
        let f = figure(e, &mut cache, &mut models, &defs, &kits, &items)?;
        println!("{e}: {}; {} vertices {} faces; rest height {} (after scale); sequences {:?}", f.label, f.model.vertices.len(), f.model.faces.len(), rest_height(&f), {
            let mut s = f.seqs.iter().collect::<Vec<_>>(); s.sort(); s });
        for n in &f.notes { println!("{e}: {n}"); }
        figures.push(f);
    }
    // one ground line for every cell: by default the camera aims at half the tallest figure's rest height
    let shared = match aim.as_deref() {
        Some("own") => None,
        Some(v) => Some(v.parse::<i32>()?),
        None => Some(figures.iter().map(rest_height).max().unwrap_or(0) / 2),
    };
    let mut raster = Raster::new(cw * entries.len(), ch * rows.len(), 1 << 27)?;
    raster.clear(0xff302b25);
    for (k, f) in figures.iter().enumerate() {
        let e = &entries[k];
        // lit once in the rest pose, before animation and before the definition's scale (world_actors.rs)
        let colors = f.model.light(64 + f.ambient, 850 + f.contrast, [-30, -50, -30])?;
        let mode = draw.unwrap_or(if f.player || !f.z_buffer { Draw::Actor } else { Draw::Auto });
        let rest_half = shared.unwrap_or_else(|| rest_height(f) / 2);
        for (r, row) in rows.iter().enumerate() {
            let mut m = f.model.clone();
            let mut note = String::from("rest pose");
            let wanted: Option<(u32, usize)> = if row == "bind" { None } else {
                let bits = row.split(':').collect::<Vec<_>>();
                match bits.as_slice() {
                    ["seq", id, fr] => Some((id.parse()?, fr.parse()?)),
                    [name, fr] => match f.seqs.get(*name) {
                        Some(&id) => Some((id, fr.parse()?)),
                        None => { note = format!("NO `{name}` sequence for this entry: rest pose"); None }
                    },
                    _ => return Err(format!("row `{row}`: use bind, NAME:FRAME or seq:ID:FRAME").into()),
                }
            };
            if let Some((id, fr)) = wanted {
                let s = animations.sequences.get(&id).ok_or(format!("no sequence {id}"))?.clone();
                let count = if s.cached_animation.is_some() { (s.cached_range[1].saturating_sub(s.cached_range[0])) as usize } else { s.frames.len() };
                let fr = if count == 0 { 0 } else { fr.min(count - 1) };
                let state = State { id: id as i32, frame: fr as i32, ..Default::default() };
                note = match animations.apply_observed(&mut cache, &mut m, id, &state.clock())? {
                    Applied::Classic { frame_id, skeleton_id, .. } => format!("seq {id} frame {fr} of {count}: classic {}:{} skeleton {skeleton_id}", frame_id >> 16, frame_id & 65535),
                    Applied::Cached { animation_id, skeleton_id, .. } => format!("seq {id} frame {fr} of {count}: cached animation {animation_id} skeleton {skeleton_id}"),
                    other => format!("seq {id} frame {fr}: NOT APPLIED {other:?}"),
                };
            }
            // npc_model::resize after animation; a player is never resized
            m.scale([f.scale[0] as i32, f.scale[1] as i32, f.scale[0] as i32]);
            let mut placed = m.clone();
            for v in &mut placed.vertices { v[1] += rest_half as f32; }
            let vertices = interface_model::vertices(&placed, [pitch, yaw, 0], [0, 0], zoom, false)?;
            let origin = actor_draw::origin_depth(rest_half, pitch, zoom);
            let (sprite, warning) = painter.draw(&mut cache, &icons, &m, &colors, &vertices, origin, [cw, ch], mode)?;
            let (mut lo, mut hi) = ([i32::MAX; 3], [i32::MIN; 3]);
            for v in &m.vertices {
                for x in 0..3 { lo[x] = lo[x].min(v[x] as i32); hi[x] = hi[x].max(v[x] as i32); }
            }
            println!("{e} row {row}: {note}; painted {mode:?}; box min {lo:?} max {hi:?}");
            if let Some(w) = warning { println!("{e} row {row}: WARNING {w}"); }
            sprite.draw(&mut raster, (k * cw) as i32, (r * ch) as i32, 256)?;
        }
    }
    std::fs::write(&a[3], raster.bmp()?)?;
    Ok(())
}
