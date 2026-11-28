//! research-effects2: asset_effect.rs + ANIMATED OBJECTS (locations with a sequence), by id through the client's own path
//! (Scene::replace_location / Scene::animate_location / WorldMesh::build + retain_clocks + advance, the calls live_runtime.rs
//! makes for LOC_ADD_CHANGE, LOC_DEL and LOC_ANIM) and from loose files (the same steps repeated on files).
//! Extra SCRIPT lines:
//!   loc ID X Y SHAPE ROT START [END|-] [sync]    spawn object ID at world tile at cycle START (LOC_ADD_CHANGE), remove at END
//!                                               (LOC_DEL). `sync`: follow the spawn with LOC_ANIM of the definition's own
//!                                               sequence, which starts it at frame 0 instead of the random start.
//!   locanim X Y SHAPE ROT SEQ CYCLE              LOC_ANIM: play SEQ (-1 = none) on the object on that tile
//!   floc DEF X Y SHAPE ROT START [END|-] [sync | draws=A,B]     ours from files (.loc: see load_loc); draws = the two random
//!                                               numbers [0,1) the client would draw for a random start
//!   flocanim X Y ANIMDIR|seq:ID|none CYCLE       LOC_ANIM on a file object
//!   pin                                          give every animated object already in the map a LOC_ANIM of its own
//!                                               sequence at cycle 0 (frame 0 start: a deterministic stage)
//!   shake AXIS AMPLITUDE START END               a crude camera shake for films (NOT the client's CAM_SHAKE code): the
//!                                               camera target moves by a deterministic +/-AMPLITUDE on AXIS 0 x, 1 height, 2 z
//! research-effects: spot animations (graphics) and projectiles in the WORLD, as the live client assembles them, without a
//! server: a real map location loaded by the client's map loader, NPCs and players put into ClientState by hand
//! (as examples/live_world.rs), and effects started the way the three server packets start them. Time runs in client
//! cycles (20 ms); any set of cycles is written as pictures.
//!
//! usage: asset_effect CACHE SCRIPT.txt
//! SCRIPT lines (blank and `#` lines ignored; coordinates are world tiles and may be fractional only for the camera):
//!   size W H                                                    (the picture; the lens scales with W so a small picture is not a crop)
//!   focal F                                                     (lens in pixels; default 1000 * W / 1280, live_world's view)
//!   camera WORLD_X WORLD_Y YAW DISTANCE ELEVATION TARGET_UP       (as live_world; yaw in 16384ths)
//!   npc ID X Y ANGLE [seq=SEQ@CYCLE ...]                           actor n1, n2, .. in order of appearance
//!   player X Y GENDER ANGLE ITEM,ITEM|- [stand=SEQ] [walk=SEQ] [seq=SEQ@CYCLE ...]     actor p1, p2, ..
//!   -- by id, THE CLIENT'S OWN PATH (ClientState + world_actors::Actors::triangles):
//!   spot ID X Y HEIGHT START                      a graphic on a tile          (world_spot::Request, zone record 5)
//!   aspot ACTOR ID HEIGHT START [SLOT]            a graphic on an actor        (actor_spot::Effects::apply, opcode 72 / NPC)
//!   proj ID SRC DST H0 H1 START END SLOPE OFFSET  a projectile                 (projectile::Request); SRC, DST = X,Y | n1 | p1
//!   -- ours as FILES (this file repeats the client's steps on loose files; proven equal by pixels, see EFFECTS.md):
//!   fspot DEF X Y HEIGHT START
//!   faspot ACTOR DEF HEIGHT START [SLOT]
//!   fproj DEF SRC DST H0 H1 START END SLOPE OFFSET
//!   render CYCLES OUTPREFIX                       CYCLES = a,b,c | A:B:STEP ; writes OUTPREFIX-CCCC.bmp
//! DEF file (key=value lines): name=  model=PATH|cache:ID  anim=DIR (seq.dat, frame_G_F.dat, base_B.dat) | seq=ID (cache)
//!   scale=XZ,Y  rotation=0|90|180|270  ambient=N  contrast=N (as stored; a spot's contrast is NOT multiplied)  recolor=FROM:TO,..
//!   noflag=1 (opcode 10)
//!
//! What the client does with a spot animation (src/world_actors.rs place_spot; the projectile branch of `triangles`):
//!   Definition::base_model: the model, recoloured and retextured;
//!   model.light(64 + ambient, 850 + contrast, [-30, -50, -30])  ONCE, in the rest pose, cached per id;
//!   on a copy: Animations::apply(sequence, clock) -> Model::animate_classic; model.scale([sx, sy, sx]); rotate_quarter
//!   for 90/180/270 only; then
//!     spot:        translate(0, -height, 0); place at (x, ground elevation, z), turned by the actor's angle (0 on a tile);
//!                  an actor's own transparency override is applied on top (apply_alpha);
//!     projectile:  every vertex turned about X by the flight pitch (16.16 fixed point), placed at the flight position
//!                  turned by the flight yaw; Motion::advance (src/projectile.rs) moves it every cycle.
//!   Clocks: actor spots tick in tick_actor_sequences, world spots in draw_root_graphics, projectiles in
//!   draw_projectiles_in_views; each is removed at its first sequence wrap (flags & 1) or, a projectile, after END.
//! For the FILE entries this program runs those same calls on the same structs (actor_spot::Effect, projectile::Motion,
//! animation_runtime::Clock) and builds the Triangle exactly as world_actors.rs `place` does.
use gielinor_client::{
    actor_spot::Effect, animation::{Frame as AFrame, Sequence, Skeleton}, animation_runtime::{Animations, Clock}, client_state::ClientState,
    disk_cache::DiskCache, identity_kit::Kit, model::Model, model_bounds::Bounds, model_order::Submission,
    npc_info::{Appearance as NpcAppearance, Npc}, player_appearance, player_info::{Player, State}, player_model::Appearance,
    projectile::{Motion, Request}, rebuild::Normal, render3d::{texture_uv, Frame, TextureMapping, Triangle}, scene::Scene, texture,
    world_actors::Actors, world_mesh::WorldMesh,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
type R<T> = Result<T, Box<dyn std::error::Error>>;

fn appearance_block(base: &Appearance, items: &gielinor_client::item::Definitions, ids: &[u32], name: &str, stand: u16, walk: u16) -> R<Vec<u8>> {
    let mut eq = base.equipment;
    for &id in ids {
        let d = items.items.get(&id).ok_or(format!("item {id} is not in the cache"))?;
        let slot = d.wear_positions[0];
        if slot < 0 { return Err(format!("item {id} declares no wear position").into()); }
        eq[slot as usize] = id + 2048;
        for &h in &d.wear_positions[1..] { if h >= 0 { eq[h as usize] = 0; } }
    }
    let mut b = vec![base.gender, 255, 255];
    let put = |b: &mut Vec<u8>, v: u32| { if v == 0 { b.push(0) } else { b.extend((v as u16).to_be_bytes()) } };
    for v in eq { put(&mut b, v); }
    for v in base.equipment { put(&mut b, v); }
    b.extend(base.colors);
    for s in [stand, 823, walk, 820, 821, 822, 824] { b.extend(s.to_be_bytes()); }
    b.extend(name.bytes()); b.push(0);
    b.push(3); b.extend(0u16.to_be_bytes()); b.push(0); b.extend(0u16.to_be_bytes());
    b.extend([0, 0, 0]); b.push(0);
    Ok(b)
}

/// One of ours, from files: everything a spot animation definition names, loaded loose.
struct FileSpot {
    name: String,
    model: Model,
    colors: Vec<[i32; 3]>,
    sequence: Option<Sequence>,
    frames: HashMap<u32, AFrame>,
    bases: HashMap<u16, Skeleton>,
    scale: [i32; 2],
    rotation: u16,
    flag: bool,
}
fn load_def(path: &str, cache: &mut DiskCache) -> R<FileSpot> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let dir = Path::new(path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let rel = |v: &str| if Path::new(v).is_absolute() { PathBuf::from(v) } else { dir.join(v) };
    let (mut name, mut model, mut anim, mut seq) = (String::from("ours"), None::<Model>, None::<PathBuf>, None::<u32>);
    let (mut scale, mut rotation, mut ambient, mut contrast, mut flag) = ([128i32; 2], 0u16, 0i32, 0i32, true);
    let mut recolor = Vec::<(u16, u16)>::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let (k, v) = line.split_once('=').ok_or(format!("{path}: `{line}` is not key=value"))?;
        let (k, v) = (k.trim(), v.trim());
        match k {
            "name" => name = v.to_string(),
            "model" => model = Some(if let Some(id) = v.strip_prefix("cache:") { Model::decode(&cache.file(7, id.parse()?, 0, [0; 4])?)? }
                                    else { Model::decode(&std::fs::read(rel(v)).map_err(|e| format!("{v}: {e}"))?)? }),
            "anim" => anim = Some(rel(v)),
            "seq" => seq = Some(v.parse()?),
            "scale" => { let (a, b) = v.split_once(',').ok_or("scale=XZ,Y")?; scale = [a.trim().parse()?, b.trim().parse()?]; }
            "rotation" => rotation = v.parse()?,
            "ambient" => ambient = v.parse()?,
            "contrast" => contrast = v.parse()?,
            "noflag" => flag = v == "0",
            "recolor" => for p in v.split(',').filter(|p| !p.trim().is_empty()) {
                let (a, b) = p.trim().split_once(':').ok_or("recolor=FROM:TO,..")?;
                recolor.push((a.trim().parse()?, b.trim().parse()?));
            },
            "note" => {}
            _ => return Err(format!("{path}: unknown key `{k}`").into()),
        }
    }
    let mut model = model.ok_or(format!("{path}: no model"))?;
    for &(from, to) in &recolor { model.recolor(from, to); }
    let colors = model.light(64 + ambient, 850 + contrast, [-30, -50, -30])?;
    let mut frames = HashMap::new();
    let mut bases: HashMap<u16, Skeleton> = HashMap::new();
    let sequence = match (&anim, seq) {
        (Some(d), _) => Some(Sequence::decode(seq.unwrap_or(65535), &std::fs::read(d.join("seq.dat")).map_err(|e| format!("{}: {e}", d.join("seq.dat").display()))?)?),
        (None, Some(id)) => Some(Sequence::load_all(cache)?.get(&id).ok_or(format!("no sequence {id}"))?.clone()),
        (None, None) => None,
    };
    if let Some(s) = &sequence {
        if s.cached_animation.is_some() { return Err(format!("{path}: skeletal (cached) sequences are not supported for file effects").into()); }
        for &id in &s.frames {
            if frames.contains_key(&id) { continue; }
            let (group, file) = (id >> 16, id & 65535);
            let loose = anim.as_ref().map(|d| d.join(format!("frame_{group}_{file}.dat"))).filter(|p| p.exists());
            let data = match &loose {
                Some(p) => std::fs::read(p)?,
                None => cache.group(0, group, [0; 4])?.iter().find(|(x, _)| *x == file).ok_or("animation frame missing")?.1.clone(),
            };
            let sid = u16::from_be_bytes(data.get(..2).ok_or("short frame")?.try_into().unwrap());
            if !bases.contains_key(&sid) {
                let lb = anim.as_ref().map(|d| d.join(format!("base_{sid}.dat"))).filter(|p| p.exists());
                let bytes = match &lb { Some(p) => std::fs::read(p)?, None => cache.file(1, sid as u32, 0, [0; 4])? };
                bases.insert(sid, Skeleton::decode(sid, &bytes)?);
            }
            frames.insert(id, AFrame::decode(&data, &bases[&sid])?);
        }
    }
    println!("def {path}: {name:?} {} vertices {} faces; sequence {:?} frames {} total {} loop {}; scale {scale:?} rotation {rotation} ambient {ambient} contrast {contrast}",
        model.vertices.len(), model.faces.len(), sequence.as_ref().map(|s| s.id), sequence.as_ref().map_or(0, |s| s.frames.len()),
        sequence.as_ref().map_or(0, |s| s.total_length), sequence.as_ref().map_or(0, |s| s.loop_frames));
    Ok(FileSpot { name, model, colors, sequence, frames, bases, scale, rotation, flag })
}
/// One of our OBJECTS from files: what an object definition names, loaded loose.
/// .loc keys: name=  model=PATH|cache:ID[,..]  anim=DIR | seq=ID  ambient=N contrast=N (as stored: the client multiplies
/// contrast by 5)  scale=X,Y,Z  offset=X,Y,Z  recolor=FROM:TO,..  size=X,Y  mirror=1 (opcode 62)  randomstart=0 (opcode 89)
/// contour=0 (opcode 21: the posed mesh follows the ground) | contour=N (opcode 81: N = byte x 256)
struct FileLoc {
    name: String,
    model: Model,
    colors: Vec<[i32; 3]>,
    sequence: Option<Sequence>,
    frames: HashMap<u32, AFrame>,
    bases: HashMap<u16, Skeleton>,
    size: [u8; 2],
    random: bool,
    rotation: u8,
    contour: i32,
}
fn load_anim(anim: &Option<PathBuf>, seq: Option<u32>, cache: &mut DiskCache) -> R<(Option<Sequence>, HashMap<u32, AFrame>, HashMap<u16, Skeleton>)> {
    let mut frames = HashMap::new();
    let mut bases: HashMap<u16, Skeleton> = HashMap::new();
    let sequence = match (anim, seq) {
        (Some(d), _) => Some(Sequence::decode(seq.unwrap_or(65535), &std::fs::read(d.join("seq.dat")).map_err(|e| format!("{}: {e}", d.join("seq.dat").display()))?)?),
        (None, Some(id)) => Some(Sequence::load_all(cache)?.get(&id).ok_or(format!("no sequence {id}"))?.clone()),
        (None, None) => None,
    };
    if let Some(s) = &sequence {
        if s.cached_animation.is_some() { return Err("skeletal (cached) sequences are not supported for file objects".into()); }
        for &id in &s.frames {
            if frames.contains_key(&id) { continue; }
            let (group, file) = (id >> 16, id & 65535);
            let loose = anim.as_ref().map(|d| d.join(format!("frame_{group}_{file}.dat"))).filter(|p| p.exists());
            let data = match &loose {
                Some(p) => std::fs::read(p)?,
                None => cache.group(0, group, [0; 4])?.iter().find(|(x, _)| *x == file).ok_or("animation frame missing")?.1.clone(),
            };
            let sid = u16::from_be_bytes(data.get(..2).ok_or("short frame")?.try_into().unwrap());
            if !bases.contains_key(&sid) {
                let lb = anim.as_ref().map(|d| d.join(format!("base_{sid}.dat"))).filter(|p| p.exists());
                let bytes = match &lb { Some(p) => std::fs::read(p)?, None => cache.file(1, sid as u32, 0, [0; 4])? };
                bases.insert(sid, Skeleton::decode(sid, &bytes)?);
            }
            frames.insert(id, AFrame::decode(&data, &bases[&sid])?);
        }
    }
    Ok((sequence, frames, bases))
}
fn load_loc(path: &str, rotation: u8, cache: &mut DiskCache) -> R<FileLoc> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let dir = Path::new(path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let rel = |v: &str| if Path::new(v).is_absolute() { PathBuf::from(v) } else { dir.join(v) };
    let (mut name, mut parts, mut anim, mut seq) = (String::from("ours"), Vec::<Model>::new(), None::<PathBuf>, None::<u32>);
    let (mut scale, mut offset, mut ambient, mut contrast, mut size, mut mirror, mut random, mut contour) = ([128i32; 3], [0i32; 3], 0i32, 0i32, [1u8; 2], false, true, -1i32);
    let mut recolor = Vec::<(u16, u16)>::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let (k, v) = line.split_once('=').ok_or(format!("{path}: `{line}` is not key=value"))?;
        let (k, v) = (k.trim(), v.trim());
        let three = |v: &str| -> R<[i32; 3]> { let q: Vec<i32> = v.split(',').map(|x| x.trim().parse()).collect::<Result<_, _>>()?; if q.len() != 3 { return Err("X,Y,Z".into()); } Ok([q[0], q[1], q[2]]) };
        match k {
            "name" => name = v.to_string(),
            "model" => for m in v.split(',') { let m = m.trim();
                parts.push(if let Some(id) = m.strip_prefix("cache:") { Model::decode(&cache.file(7, id.parse()?, 0, [0; 4])?)? }
                           else { Model::decode(&std::fs::read(rel(m)).map_err(|e| format!("{m}: {e}"))?)? }); },
            "anim" => anim = Some(rel(v)),
            "seq" => seq = Some(v.parse()?),
            "scale" => scale = three(v)?,
            "offset" => offset = three(v)?,
            "ambient" => ambient = v.parse::<i32>()? as i8 as i32,
            "contrast" => contrast = v.parse::<i32>()? as i8 as i32 * 5,
            "size" => { let (a, b) = v.split_once(',').ok_or("size=X,Y")?; size = [a.trim().parse()?, b.trim().parse()?]; }
            "mirror" => mirror = v == "1",
            "randomstart" => random = v != "0",
            "contour" => contour = v.parse()?,
            "recolor" => for p in v.split(',').filter(|p| !p.trim().is_empty()) {
                let (a, b) = p.trim().split_once(':').ok_or("recolor=FROM:TO,..")?;
                recolor.push((a.trim().parse()?, b.trim().parse()?));
            },
            "note" | "solid" | "interactive" | "ops" | "id" | "blocks" => {}
            _ => return Err(format!("{path}: unknown key `{k}`").into()),
        }
    }
    if parts.is_empty() { return Err(format!("{path}: no model").into()); }
    // object_model.rs Models::object, shape 10: mirror, combine, quarter turn, recolour, scale, offset
    if mirror { for m in &mut parts { m.mirror(); } }
    let mut model = if parts.len() == 1 { parts.remove(0) } else { Model::combine(&parts)? };
    model.rotate_quarter(rotation);
    for &(from, to) in &recolor { model.recolor(from, to); }
    model.scale(scale);
    model.translate(offset);
    // world_mesh.rs: an object is lit from the side, once, in its rest pose
    let colors = model.light(64 + ambient, 768 + contrast, [-50, -10, -50])?;
    let (sequence, frames, bases) = load_anim(&anim, seq, cache)?;
    println!("loc {path}: {name:?} {} vertices {} faces; sequence frames {} total {} loop {}; ambient {ambient} contrast {contrast} (x5 applied) size {size:?} random start {random}",
        model.vertices.len(), model.faces.len(), sequence.as_ref().map_or(0, |s| s.frames.len()), sequence.as_ref().map_or(0, |s| s.total_length), sequence.as_ref().map_or(0, |s| s.loop_frames));
    Ok(FileLoc { name, model, colors, sequence, frames, bases, size, random, rotation, contour })
}
struct FLoc { def: FileLoc, tile: [i32; 2], shape: u8, start: i32, end: Option<i32>, clock: Clock, started: bool, sync: bool, draws: [f64; 2], swaps: Vec<(i32, Option<PathBuf>, Option<u32>, bool)> }
enum LocEvent { Add { id: u32, tile: [i32; 2], shape: u8, rotation: u8, sync: bool }, Remove { tile: [i32; 2], shape: u8 }, Anim { tile: [i32; 2], shape: u8, rotation: u8, sequence: i32 } }

impl FileSpot {
    /// place_spot / the projectile branch up to placement: animate a copy, scale, quarter turn.
    fn posed(&self, clock: &Clock) -> R<Model> {
        let mut model = self.model.clone();
        if let Some(s) = &self.sequence {
            if !clock.stopped {
                if let Some(id) = s.frames.get(clock.frame) {
                    let f = &self.frames[id];
                    model.animate_classic(&self.bases[&f.skeleton], f, false)?;
                }
            }
        }
        model.scale([self.scale[0], self.scale[1], self.scale[0]]);
        if matches!(self.rotation, 90 | 180 | 270) { model.rotate_quarter((self.rotation / 90) as u8); }
        Ok(model)
    }
}
/// world_actors.rs `place`, repeated (it is private there). single_tile = false, as every spot and projectile passes.
fn place(out: &mut Vec<Triangle>, model: &Model, colors: &[[i32; 3]], position: [i32; 3], angle: u16, tag: u64) -> R<usize> {
    let start = out.len();
    let vertices = gielinor_client::model_draw::vertices(&model.vertices, angle, position)?;
    let bounds = Bounds::new(&model.vertices, position).with_selection(&model.vertices, angle, false);
    for (i, face) in model.faces.iter().enumerate() {
        if colors[i][2] == -2 { continue; }
        let color = if colors[i][2] == -1 { [colors[i][0]; 3] } else { colors[i] };
        let positions = face.map(|v| vertices[v as usize]);
        let texture = model.face_textures.as_ref().and_then(|t| (t[i] != -1).then_some(t[i] as u16));
        let map = model.texture_coords.as_ref()
            .and_then(|t| if t[i] == -1 { None } else { model.texture_triangles.get(t[i] as u8 as usize) })
            .unwrap_or(face);
        out.push(Triangle {
            #[cfg(feature = "enhanced-renderer")]
            material_hsl: Some(model.colors[i]),
            texture_mapping: Some(TextureMapping { vertices: map.map(|v| vertices[v as usize]), local: Some(map.map(|v| model.vertices[v as usize])) }),
            local_vertices: Some(face.map(|v| model.vertices[v as usize])),
            model_bounds: Some(bounds),
            model_order: Some(Submission::new(model, i, start)),
            depth_bias: model.face_depth_bias(i),
            view: 0,
            vertices: positions,
            colors: color,
            flat: colors[i][2] == -1,
            uv: texture_uv(map.map(|v| vertices[v as usize]), positions),
            texture,
            alpha: model.face_alpha(i),
            near_alpha: Some(model.alpha.as_ref().map_or(0, |a| a[i])),
            tag,
        });
    }
    Ok(start)
}

#[derive(Clone)]
enum End { Tile([i32; 2]), Npc(u16), Player(u16) }
fn parse_end(v: &str) -> R<End> {
    if let Some(n) = v.strip_prefix('n') { return Ok(End::Npc(n.parse()?)); }
    if let Some(n) = v.strip_prefix('p') { return Ok(End::Player(n.parse()?)); }
    let (x, y) = v.split_once(',').ok_or("endpoint: X,Y | n1 | p1")?;
    Ok(End::Tile([x.parse()?, y.parse()?]))
}
struct FWorld { def: usize, tile: [i32; 2], effect: Effect, elevation: Option<i32>, dead: bool }
struct FActor { def: usize, actor: End, effect: Effect, slot: u8, dead: bool }
struct FProj { def: usize, src: End, dst: End, heights: [i32; 2], start: i32, end: i32, slope: i32, offset: i32, motion: Motion, clock: Clock, dead: bool }

fn main() -> R<()> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() < 3 { return Err("CACHE SCRIPT.txt".into()); }
    let script = std::fs::read_to_string(&a[2])?;
    let lines: Vec<Vec<&str>> = script.lines().map(|l| l.trim()).filter(|l| !l.is_empty() && !l.starts_with('#')).map(|l| l.split_whitespace().collect()).collect();
    let cam = lines.iter().find(|l| l[0] == "camera").ok_or("script needs a camera line")?;
    let p: Vec<f32> = cam[1..7].iter().map(|v| v.parse()).collect::<Result<_, _>>()?;
    let (mut w, mut h) = (1280usize, 900usize);
    let mut focal: Option<f32> = None;
    let mut client = ClientState::load(DiskCache::open(&a[1], 64 << 20)?)?;
    let rebuild = Normal { world_area: -1, zone_x: (p[0] as u16) / 8, zone_y: (p[1] as u16) / 8, players: None };
    let mut scene = Scene::load_normal(&mut client.scripts.source.cache, &rebuild)?;
    let (bx, by) = (scene.base_x, scene.base_y);
    client.world_base = [bx, by];
    let kits = Kit::load_all(&mut client.scripts.source.cache)?;
    let sequences = Sequence::load_all(&mut client.scripts.source.cache)?;
    let (mut players, _) = State::initialize(&[0; 4608], 1)?;
    let mut high = Vec::new();
    let mut npc_index = Vec::<u16>::new();
    let mut schedule = Vec::<(i32, End, i32)>::new(); // (cycle, actor, sequence)
    let mut defs = Vec::<FileSpot>::new();
    let mut def_ix = HashMap::<String, usize>::new();
    let (mut fworld, mut factor, mut fproj) = (Vec::<FWorld>::new(), Vec::<FActor>::new(), Vec::<FProj>::new());
    let mut renders = Vec::<(Vec<i32>, String)>::new();
    let mut pending_aspots = Vec::<(End, u16, u16, u16, u8)>::new();
    let mut loc_events = Vec::<(i32, LocEvent)>::new();
    let mut flocs = Vec::<FLoc>::new();
    let mut pin = false;
    let mut shakes = Vec::<(usize, f32, i32, i32)>::new();
    for l in &lines {
        let mut def_of = |path: &str, client: &mut ClientState| -> R<usize> {
            if let Some(&i) = def_ix.get(path) { return Ok(i); }
            defs.push(load_def(path, &mut client.scripts.source.cache)?);
            def_ix.insert(path.to_string(), defs.len() - 1);
            Ok(defs.len() - 1)
        };
        match l[0] {
            "camera" => {}
            "size" => { w = l[1].parse()?; h = l[2].parse()?; }
            "focal" => focal = Some(l[1].parse()?),
            "npc" => {
                let id: u32 = l[1].parse()?;
                let def = client.npcs.get(&id).ok_or(format!("NPC {id} is not in the cache"))?.clone();
                let (x, y): (i32, i32) = (l[2].parse()?, l[3].parse()?);
                let mut path = gielinor_client::actor_path::Path::default();
                path.teleport([x, y], def.size as i32);
                let mut npc = Npc { visual: Default::default(), path, sequence: Default::default(), pose: Default::default(),
                    idle_fallback: Default::default(), spots: Default::default(), overlays: Default::default(), turn: Default::default(),
                    facing: None, animations: def.animations, freezes: Default::default(), exact: None, id: id as u16,
                    position: [x, y], angle: l[4].parse()?, spawn_cycle: None, steps: vec![], jump: false,
                    blocks: vec![], appearance: NpcAppearance::default() };
                npc.turn.angle = l[4].parse()?; npc.turn.target = l[4].parse()?;
                let index = 1 + client.npc_state.npcs.len() as u16;
                client.npc_state.npcs.insert(index, npc);
                client.npc_state.indices.push(index);
                npc_index.push(index);
                for kv in &l[5..] {
                    if let Some(v) = kv.strip_prefix("seq=") { let (s, c) = v.split_once('@').ok_or("seq=SEQ@CYCLE")?; schedule.push((c.parse()?, End::Npc(npc_index.len() as u16), s.parse()?)); }
                }
                println!("n{}: npc {id} {:?} size {} at ({x}, {y})", npc_index.len(), def.name, def.size);
            }
            "player" => {
                let (x, y, gender): (i32, i32, u8) = (l[1].parse()?, l[2].parse()?, l[3].parse()?);
                let ids = l[5].split(',').filter(|s| !s.is_empty() && *s != "-").map(|s| s.parse::<u32>()).collect::<Result<Vec<_>, _>>()?;
                let (mut stand, mut walk) = (808u16, 819u16);
                let index = 1 + high.len();
                for kv in &l[6..] {
                    if let Some(v) = kv.strip_prefix("stand=") { stand = v.parse()?; }
                    else if let Some(v) = kv.strip_prefix("walk=") { walk = v.parse()?; }
                    else if let Some(v) = kv.strip_prefix("seq=") { let (s, c) = v.split_once('@').ok_or("seq=SEQ@CYCLE")?; schedule.push((c.parse()?, End::Player(index as u16), s.parse()?)); }
                }
                let bytes = appearance_block(&Appearance::standard(&kits, gender), &client.items, &ids, &format!("Player{index}"), stand, walk)?;
                let update = player_appearance::decode(&bytes, &client.items)?;
                let mut pl = Player::at([0, x, y], Some(update), 1, None);
                pl.turn.angle = l[4].parse()?; pl.turn.target = l[4].parse()?;
                if index == 1 { client.data.position = [0, x, y]; }
                players.players[index] = Some(pl);
                high.push(index as u16);
                println!("p{index}: player at ({x}, {y}) items {ids:?}");
            }
            "spot" => {
                let q = gielinor_client::world_spot::Request { id: l[1].parse()?, height: l[4].parse()?, delay: 0, position: [0, l[2].parse()?, l[3].parse()?] };
                let mut e = q.start(0);
                e.animation.start = l[5].parse()?;
                client.world_spots.push(e);
            }
            "aspot" => pending_aspots.push((parse_end(l[1])?, l[2].parse()?, l[3].parse::<i32>()? as u16, l[4].parse()?, l.get(5).map_or(Ok(0), |v| v.parse())?)),
            "proj" => {
                let (src, dst) = (parse_end(l[2])?, parse_end(l[3])?);
                let ent = |e: &End| match e { End::Tile(_) => 0, End::Npc(n) => npc_index[*n as usize - 1] as i32 + 1, End::Player(n) => -(*n as i32 + 1) };
                let tile = |e: &End, client: &ClientState, players: &State| -> [i32; 3] { match e {
                    End::Tile(t) => [0, t[0], t[1]],
                    End::Npc(n) => { let q = client.npc_state.npcs[&npc_index[*n as usize - 1]].position; [0, q[0], q[1]] }
                    End::Player(n) => { let q = players.players[*n as usize].as_ref().unwrap().position; [0, q[1], q[2]] }
                } };
                client.projectiles.push(Request { id: l[1].parse()?, source: tile(&src, &client, &players), target: tile(&dst, &client, &players),
                    heights: [l[4].parse()?, l[5].parse()?], entities: [ent(&src), ent(&dst)], start: l[6].parse()?, end: l[7].parse()?,
                    slope: l[8].parse()?, offset: l[9].parse()? }.start(0));
            }
            "fspot" => {
                let d = def_of(l[1], &mut client)?;
                fworld.push(FWorld { def: d, tile: [l[2].parse()?, l[3].parse()?], elevation: None, dead: false,
                    effect: Effect { id: 65535, height: l[4].parse::<u8>()? as i32, start: l[5].parse()?, clock: Clock::default() } });
            }
            "faspot" => {
                let d = def_of(l[2], &mut client)?;
                factor.push(FActor { def: d, actor: parse_end(l[1])?, slot: l.get(5).map_or(Ok(0), |v| v.parse())?, dead: false,
                    effect: Effect { id: 65535, height: l[3].parse::<i32>()? as u16 as i16 as i32, start: l[4].parse()?, clock: Clock::default() } });
            }
            "fproj" => {
                let d = def_of(l[1], &mut client)?;
                fproj.push(FProj { def: d, src: parse_end(l[2])?, dst: parse_end(l[3])?, heights: [l[4].parse()?, l[5].parse()?], start: l[6].parse()?,
                    end: l[7].parse()?, slope: l[8].parse()?, offset: l[9].parse()?, motion: Motion::default(), clock: Clock::default(), dead: false });
            }
            "pin" => pin = true,
            "shake" => shakes.push((l[1].parse()?, l[2].parse()?, l[3].parse()?, l[4].parse()?)),
            "loc" => {
                let (id, tile, shape, rotation, start): (u32, [i32; 2], u8, u8, i32) = (l[1].parse()?, [l[2].parse()?, l[3].parse()?], l[4].parse()?, l[5].parse()?, l[6].parse()?);
                let sync = l[7..].contains(&"sync");
                loc_events.push((start, LocEvent::Add { id, tile, shape, rotation, sync }));
                if let Some(e) = l.get(7).filter(|v| **v != "-" && **v != "sync") { loc_events.push((e.parse()?, LocEvent::Remove { tile, shape })); }
            }
            "locanim" => loc_events.push((l[6].parse()?, LocEvent::Anim { tile: [l[1].parse()?, l[2].parse()?], shape: l[3].parse()?, rotation: l[4].parse()?, sequence: l[5].parse()? })),
            "floc" => {
                let rotation: u8 = l[5].parse()?;
                let def = load_loc(l[1], rotation, &mut client.scripts.source.cache)?;
                let mut f = FLoc { def, tile: [l[2].parse()?, l[3].parse()?], shape: l[4].parse()?, start: l[6].parse()?, end: None, clock: Clock::default(), started: false, sync: false, draws: [0.0, 0.0], swaps: vec![] };
                for (i, v) in l[7..].iter().enumerate() {
                    if *v == "sync" { f.sync = true; }
                    else if let Some(d) = v.strip_prefix("draws=") { let (a, b) = d.split_once(',').ok_or("draws=A,B")?; f.draws = [a.parse()?, b.parse()?]; }
                    else if i == 0 && *v != "-" { f.end = Some(v.parse()?); }
                }
                flocs.push(f);
            }
            "flocanim" => {
                let tile: [i32; 2] = [l[1].parse()?, l[2].parse()?];
                let at: i32 = l[4].parse()?;
                let f = flocs.iter_mut().filter(|f| f.tile == tile).last().ok_or("flocanim: no file object on that tile")?;
                if l[3] == "none" { f.swaps.push((at, None, None, true)); }
                else if let Some(id) = l[3].strip_prefix("seq:") { f.swaps.push((at, None, Some(id.parse()?), false)); }
                else { f.swaps.push((at, Some(PathBuf::from(l[3])), None, false)); }
            }
            "render" => {
                let cycles = if l[1].contains(':') {
                    let v: Vec<i32> = l[1].split(':').map(|x| x.parse()).collect::<Result<_, _>>()?;
                    (v[0]..=v[1]).step_by(*v.get(2).unwrap_or(&1) as usize).collect()
                } else { l[1].split(',').map(|x| x.parse()).collect::<Result<_, _>>()? };
                renders.push((cycles, l[2].to_string()));
            }
            other => return Err(format!("script: unknown line `{other}`").into()),
        }
    }
    players.high = high;
    client.player_state = Some(players);
    client.preferences.render_self = true;
    // actor graphics are applied at cycle 0 with their start as the delay, exactly as the update block does
    for (actor, id, height, start, slot) in pending_aspots {
        match actor {
            End::Npc(n) => client.npc_state.npcs.get_mut(&npc_index[n as usize - 1]).ok_or("aspot: no such npc")?.spots.apply(slot, id, height, start, 0),
            End::Player(n) => client.player_state.as_mut().unwrap().players[n as usize].as_mut().ok_or("aspot: no such player")?.spots.apply(slot, id, height, start, 0),
            End::Tile(_) => return Err("aspot needs n1 / p1".into()),
        }
    }
    let obj_defs = gielinor_client::object::Definition::load_all(&mut client.scripts.source.cache)?;
    let mut animations = Animations::load(&mut client.scripts.source.cache)?;
    if pin {
        let placed: Vec<([i32; 4], u8, u8, i32)> = scene.objects.iter().filter_map(|o| {
            let d = obj_defs.get(&o.id)?; let a = d.animation?;
            Some(([o.plane as i32, bx + o.x as i32, by + o.y as i32, gielinor_client::location_update::category(o.shape).ok()? as i32], o.shape, o.rotation, a as i32))
        }).collect();
        let mut n = 0;
        for (key, shape, rotation, sequence) in placed {
            if scene.animate_location(key, gielinor_client::location_animation::Request { at: 0, shape, rotation, sequence }, &obj_defs)? { n += 1; }
        }
        println!("pin: {n} animated map objects started at frame 0");
    }
    let mut mesh = WorldMesh::build(&scene, &mut client.scripts.source.cache, 0)?;
    if mesh.pending_assets { return Err("assets pending".into()); }
    let mut actors = Actors::load(&mut client.scripts.source.cache)?;
    let local = [((p[0] - bx as f32) * 128.) as i32, ((p[1] - by as f32) * 128.) as i32];
    let floor_height = scene.height_fine(0, local[0], local[1]).ok_or("target height missing")? as f32;
    let yaw = p[2] * std::f32::consts::TAU / 16384.0;
    let (x, z) = (local[0] as f32, local[1] as f32);
    let target = [x, floor_height - p[5], z];
    let mut camera = gielinor_client::render3d::Camera::look_at([x + yaw.sin() * p[3], target[1] - p[4], z - yaw.cos() * p[3]], target, focal.unwrap_or(1000. * w as f32 / 1280.))?;
    camera.far = 30000;
    let mut textures = HashMap::new();
    for (id, d) in texture::Definition::load_all(&mut client.scripts.source.cache)? {
        textures.insert(id, texture::Texture::load(&mut client.scripts.source.cache, d, 128, 0.8)?);
    }
    let palette = gielinor_client::floor::palette(0.8)?;
    let last = renders.iter().flat_map(|(c, _)| c.iter().copied()).max().unwrap_or(0);
    // an actor's fine position (world fine units) and angle
    let actor_at = |client: &ClientState, e: &End| -> Option<([i32; 2], u16, u64, u8, i32)> { match e {
        End::Npc(n) => client.npc_state.npcs.get(&npc_index[*n as usize - 1]).map(|q| (q.path.position, q.angle, 0x4000000000000000 | npc_index[*n as usize - 1] as u64, q.visual.alpha(client.actor_cycle), gielinor_client::actor_height::npc(&sequences, q))),
        End::Player(n) => client.player_state.as_ref().and_then(|s| s.root_actor(*n)).map(|q| (q.path.position, q.turn.angle, 0x2000000000000000 | *n as u64, q.visual.alpha(client.actor_cycle), gielinor_client::actor_height::player(&sequences, q))),
        End::Tile(_) => None,
    } };
    for c in 0..=last {
        client.actor_cycle = c;
        for (at, who, seq) in &schedule {
            if *at != c { continue; }
            match who {
                End::Npc(n) => client.npc_state.npcs.get_mut(&npc_index[*n as usize - 1]).unwrap().sequence.set(*seq, 0, 0, &sequences)?,
                End::Player(n) => client.player_state.as_mut().unwrap().players[*n as usize].as_mut().unwrap().sequence.set(*seq, 0, 0, &sequences)?,
                End::Tile(_) => {}
            }
        }
        // ---- objects: LOC_ADD_CHANGE / LOC_DEL / LOC_ANIM as live_runtime.rs applies them, then the mesh's clocks
        let mut changed = false;
        for (at, ev) in &loc_events {
            if *at != c { continue; }
            changed = true;
            match ev {
                LocEvent::Add { id, tile, shape, rotation, sync } => {
                    let key = [0, tile[0], tile[1], gielinor_client::location_update::category(*shape)? as i32];
                    scene.replace_location(key, &gielinor_client::location_update::Object { id: *id as i32, shape: *shape, rotation: *rotation }, &obj_defs)?;
                    let d = obj_defs.get(id).ok_or(format!("object {id} is not in the cache"))?;
                    if *sync { if let Some(a) = d.animation {
                        scene.animate_location(key, gielinor_client::location_animation::Request { at: 0, shape: *shape, rotation: *rotation, sequence: a as i32 }, &obj_defs)?; } }
                    println!("cycle {c}: loc add {id} {:?} at {tile:?} shape {shape} rotation {rotation}; sequence {:?} random start {} sync {sync}; models {:?} size {:?} ambient {} contrast {}", d.name, d.animation, d.randomize_animation, d.models, d.size, d.ambient, d.contrast);
                }
                LocEvent::Remove { tile, shape } => {
                    let key = [0, tile[0], tile[1], gielinor_client::location_update::category(*shape)? as i32];
                    scene.replace_location(key, &gielinor_client::location_update::Object { id: -1, shape: 0, rotation: 0 }, &obj_defs)?;
                    println!("cycle {c}: loc del at {tile:?}");
                }
                LocEvent::Anim { tile, shape, rotation, sequence } => {
                    let key = [0, tile[0], tile[1], gielinor_client::location_update::category(*shape)? as i32];
                    let ok = scene.animate_location(key, gielinor_client::location_animation::Request { at: 0, shape: *shape, rotation: *rotation, sequence: *sequence }, &obj_defs)?;
                    println!("cycle {c}: loc anim {sequence} at {tile:?}: applied {ok}");
                }
            }
        }
        if changed {
            let mut next = WorldMesh::build(&scene, &mut client.scripts.source.cache, 0)?;
            next.retain_clocks(&mesh, &animations);
            mesh = next;
        }
        mesh.advance(&scene, &mut client.scripts.source.cache, &mut animations, 1)?;
        // ---- the same for file objects
        for f in &mut flocs {
            if c < f.start || f.end.is_some_and(|e| c >= e) { continue; }
            let swaps: Vec<_> = f.swaps.iter().filter(|s| s.0 == c).cloned().collect();
            for (_, dir, id, none) in swaps {
                if none { f.def.sequence = None; } else {
                    let (s, fr, ba) = load_anim(&dir, id, &mut client.scripts.source.cache)?;
                    f.def.sequence = s; f.def.frames = fr; f.def.bases = ba;
                }
                f.clock = Clock::default(); f.started = true;   // a LOC_ANIM start is never random
            }
            let Some(s) = &f.def.sequence else { continue };
            if !f.started {
                f.started = true;
                if f.def.random && !f.sync && s.loop_frames != -1 { f.clock.randomize(s, f.draws)?; }
            }
            if !f.clock.stopped {
                let (flags, _) = f.clock.advance(s, 1)?;
                if flags & 9 == 9 { f.clock.stopped = true; }
            }
        }
        // ---- the client's clocks: actors (sequences, poses, their spots), then projectiles, then tile graphics
        client.tick_actor_sequences()?;            // leaves actor_cycle = c + 1
        client.draw_projectiles_in_views(&scene, &HashMap::new(), 1)?;
        client.draw_root_graphics(1)?;
        // ---- the same clocks for the file entries
        for f in &mut factor {
            if f.dead || c < f.effect.start { continue; }
            match &defs[f.def].sequence {
                None => f.dead = true,
                Some(s) => { let (flags, _) = f.effect.clock.advance(s, 1)?; if flags & 1 != 0 { f.dead = true; } }
            }
        }
        for f in &mut fproj {
            if f.dead { continue; }
            if c > f.end { f.dead = true; continue; }
            if c < f.start { continue; }
            let point = |e: &End, height: i32, client: &ClientState| -> [i32; 3] {
                let fine: [i32; 2] = match (actor_at(client, e), e) {
                    (Some((pos, ..)), _) => [pos[0] - bx * 128, pos[1] - by * 128],
                    (None, End::Tile(t)) => [(t[0] - bx) * 128 + 64, (t[1] - by) * 128 + 64],
                    _ => [0, 0],
                };
                let elevation = scene.elevation(0, fine[0], fine[1]).unwrap_or(0);
                [fine[0] + bx * 128, fine[1] + by * 128, elevation - height]
            };
            let (s, t) = (point(&f.src, f.heights[0], &client), point(&f.dst, f.heights[1], &client));
            f.motion.advance(s, t, f.start, f.end, c, 1, f.slope, f.offset);
            if let Some(seq) = &defs[f.def].sequence { f.clock.advance(seq, 1)?; }
        }
        for f in &mut fworld {
            if f.dead { continue; }
            match &defs[f.def].sequence {
                None => f.dead = true,
                Some(s) => if c >= f.effect.start { let (flags, _) = f.effect.clock.advance(s, 1)?; if flags & 1 != 0 { f.dead = true; } },
            }
        }
        let wanted: Vec<&String> = renders.iter().filter(|(cs, _)| cs.contains(&c)).map(|(_, o)| o).collect();
        if wanted.is_empty() { continue; }
        let (mut tris, pending) = actors.triangles(&mut client, &scene, &mesh.item_support)?;
        for wmsg in &pending { println!("pending: {wmsg}"); }
        // file objects: scenery, submitted after the map's own and NOT registered with the actors' order (as mesh objects are not)
        let mut ltris = Vec::<Triangle>::new();
        for f in &flocs {
            if c < f.start || f.end.is_some_and(|e| c >= e) { continue; }
            let d = &f.def;
            let mut model = d.model.clone();
            let mut lift = 0;
            if let Some(s) = &d.sequence {
                lift = gielinor_client::location_animation::height_offset(Some((if f.clock.stopped { -1 } else { s.id as i32 }, s.extra)), None);
                if !f.clock.stopped {
                    if let Some(id) = s.frames.get(f.clock.frame) {
                        model.rotate_quarter(4 - (d.rotation & 3));
                        let fr = &d.frames[id];
                        model.animate_classic(&d.bases[&fr.skeleton], fr, false)?;
                        model.rotate_quarter(d.rotation);
                    }
                }
            }
            let (x, y) = ((f.tile[0] - bx) as usize, (f.tile[1] - by) as usize);
            let [xs, ys] = if d.rotation & 1 != 0 { [d.size[1], d.size[0]] } else { d.size };
            let (x1, x2, y1, y2) = (x + xs as usize / 2, x + (xs as usize + 1) / 2, y + ys as usize / 2, y + (ys as usize + 1) / 2);
            let hgt = |a: usize, b: usize| scene.height(0, a, b).unwrap_or(0);
            let base = (hgt(x1, y1) + hgt(x2, y1) + hgt(x1, y2) + hgt(x2, y2)) >> 2;
            let centre = gielinor_client::location_draw_position::center(f.shape, [x as i32, y as i32], [xs, ys]);
            if d.contour >= 0 {
                let origin = gielinor_client::location_animation::contour_origin(d.size, d.rotation, [x, y], |a, b| scene.height(0, a, b))?;
                model.vertices = gielinor_client::model_contour::rendered(&model, origin, d.contour, |a, b| scene.height(0, a, b))?;
            }
            let tag = ((70000u64 + 1) << 32) | ((x as u64) << 14) | y as u64;
            let start = place(&mut ltris, &model, &d.colors, [centre[0], base.wrapping_sub(lift), centre[1]], if f.shape == 11 { 256 } else { 0 }, tag)?;
            for t in &mut ltris[start..] { if let Some(o) = &mut t.model_order { o.scene_height_offset = lift; } }
            println!("cycle {c}: floc {:?} frame {} elapsed {} stopped {}", d.name, f.clock.frame, f.clock.elapsed, f.clock.stopped);
        }
        let mut temporary = 100000usize;
        // file projectiles, then file actor spots, then file tile spots (the order of world_actors::triangles)
        for f in &fproj {
            if f.dead || !f.motion.moving { continue; }
            let d = &defs[f.def];
            let mut model = d.posed(&f.clock)?;
            let radians = f.motion.pitch as f64 * 0.0030679615;
            let sin = (radians.sin() * 65536.0) as i32;
            let cos = (radians.cos() * 65536.0) as i32;
            for v in &mut model.vertices {
                let y = v[1] as i32;
                let z = v[2] as i32;
                v[1] = (y.wrapping_mul(cos).wrapping_sub(z.wrapping_mul(sin)) >> 16) as f32;
                v[2] = (y.wrapping_mul(sin).wrapping_add(z.wrapping_mul(cos)) >> 16) as f32;
            }
            let q = [f.motion.position[0], f.motion.position[2], f.motion.position[1]];
            let start = place(&mut tris, &model, &d.colors, [q[0] as i32 - bx * 128, q[1] as i32, q[2] as i32 - by * 128], f.motion.yaw, 0)?;
            for t in &mut tris[start..] { if let Some(o) = &mut t.model_order { o.temporary_id = Some(temporary); } }
            temporary += 1;
            println!("cycle {c}: fproj {:?} at ({:.1}, {:.1}) height {:.1} yaw {} pitch {} frame {}", d.name, f.motion.position[0] - bx as f64 * 128., f.motion.position[1] - by as f64 * 128., -f.motion.position[2], f.motion.yaw, f.motion.pitch, f.clock.frame);
        }
        for f in &factor {
            if f.dead || client.actor_cycle < f.effect.start { continue; }
            let Some((pos, angle, tag, visual_alpha, lift)) = actor_at(&client, &f.actor) else { continue };
            let d = &defs[f.def];
            let (x, z) = (pos[0] - bx * 128, pos[1] - by * 128);
            let Some(hh) = scene.elevation(0, x, z) else { continue };
            let mut model = d.posed(&f.effect.clock)?;
            model.translate([0, -f.effect.height, 0]);
            let start = place(&mut tris, &model, &d.colors, [x, hh, z], angle, if d.flag { tag } else { 0 })?;
            for t in &mut tris[start..] {
                if visual_alpha != 0 && t.local_vertices.is_some() { t.alpha = gielinor_client::actor_visual::face_alpha(t.alpha, visual_alpha); }
                if let Some(o) = &mut t.model_order { o.actor_spot = Some(tag); o.source_ordinal = Some(f.slot as usize); o.scene_height_offset = lift; }
                // shift_actor_heights: the sequence's height byte (opcode 16) of what the actor is doing lifts the body AND its graphics
                if lift != 0 {
                    for v in &mut t.vertices { v[1] -= lift as f32; }
                    if let Some(m) = &mut t.texture_mapping { for v in &mut m.vertices { v[1] -= lift as f32; } }
                    if let Some(b) = &mut t.model_bounds { b.position[1] = b.position[1].wrapping_sub(lift); }
                }
            }
            println!("cycle {c}: faspot {:?} frame {} elapsed {}", d.name, f.effect.clock.frame, f.effect.clock.elapsed);
        }
        for f in &mut fworld {
            if f.dead || c < f.effect.start { continue; }
            let d = &defs[f.def];
            let (x, z) = ((f.tile[0] - bx) * 128 + 64, (f.tile[1] - by) * 128 + 64);
            if f.elevation.is_none() { f.elevation = scene.elevation(0, x, z); }
            let Some(hh) = f.elevation else { continue };
            let mut model = d.posed(&f.effect.clock)?;
            model.translate([0, -f.effect.height, 0]);
            let start = place(&mut tris, &model, &d.colors, [x, hh, z], 0, 0)?;
            for t in &mut tris[start..] { if let Some(o) = &mut t.model_order { o.temporary_id = Some(temporary); } }
            temporary += 1;
            println!("cycle {c}: fspot {:?} frame {} elapsed {}", d.name, f.effect.clock.frame, f.effect.clock.elapsed);
        }
        for q in &client.projectiles {
            if q.motion.moving { println!("cycle {c}: proj {} at ({:.1}, {:.1}) height {:.1} yaw {} pitch {} frame {}", q.request.id, q.motion.position[0] - bx as f64 * 128., q.motion.position[1] - by as f64 * 128., -q.motion.position[2], q.motion.yaw, q.motion.pitch, q.clock.frame); }
        }
        for e in &client.world_spots { println!("cycle {c}: spot {} frame {} elapsed {}", e.animation.id, e.animation.clock.frame, e.animation.clock.elapsed); }
        if let Some(st) = &client.player_state { for i in 1..=8u16 { if let Some(q) = st.root_actor(i) { for e in q.spots.ordered() { println!("cycle {c}: aspot {} on player {i} frame {} elapsed {} lift {}", e.id, e.clock.frame, e.clock.elapsed, gielinor_client::actor_height::player(&sequences, q)); } } } }
        for (i, n) in &client.npc_state.npcs { for e in n.spots.ordered() { println!("cycle {c}: aspot {} on npc index {i} frame {} elapsed {}", e.id, e.clock.frame, e.clock.elapsed); } }
        gielinor_client::model_order::rebase(&mut ltris, mesh.triangles.len());
        gielinor_client::model_order::rebase(&mut tris, mesh.triangles.len() + ltris.len());
        let mut t = target;
        for &(axis, amp, s0, s1) in &shakes {
            if c >= s0 && c < s1 { let k = ((c as u32).wrapping_mul(2654435761) >> 16) & 1023; t[axis.min(2)] += amp * (k as f32 / 511.5 - 1.0); }
        }
        let mut camera = gielinor_client::render3d::Camera::look_at([x + yaw.sin() * p[3], target[1] - p[4], z - yaw.cos() * p[3]], t, focal.unwrap_or(1000. * w as f32 / 1280.))?;
        camera.far = 30000;
        let mut frame = Frame::new(w, h)?;
        gielinor_client::world_actors::register_root_order(&mut frame, &camera, &client, &scene, &tris);
        frame.draw_iter(&camera, mesh.visible_triangles(0).chain(ltris.iter()).chain(tris.iter()), &palette, &textures);
        let bmp = frame.raster()?.bmp()?;
        for o in wanted { std::fs::write(format!("{o}-{c:04}.bmp"), &bmp)?; }
        println!("cycle {c}: {} actor and effect faces", tris.len());
    }
    println!("asset_effect: base ({bx}, {by}); {} scene faces", mesh.triangles.len());
    Ok(())
}
