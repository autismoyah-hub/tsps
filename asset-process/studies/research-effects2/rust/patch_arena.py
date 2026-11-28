"""patch_arena.py: writes client/rust-client/examples/asset_arena.rs = examples/asset_effect.rs + animated objects.
Run once (it starts from asset_effect.rs every time). Kept so the example can be regenerated and read as a diff."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import pathlib
ROOT = pathlib.Path(_AP.fix('<CLIENT_SRC>/examples'))
s = (ROOT / 'asset_effect.rs').read_text(encoding='utf-8')


def rep(a, b):
    global s
    assert s.count(a) == 1, (s.count(a), a[:70])
    s = s.replace(a, b)


rep("//! research-effects: spot animations", """//! research-effects2: asset_effect.rs + ANIMATED OBJECTS (locations with a sequence), by id through the client's own path
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
//! research-effects: spot animations""")
rep("actor_spot::Effect, animation::{Frame as AFrame, Sequence, Skeleton}, animation_runtime::Clock,",
    "actor_spot::Effect, animation::{Frame as AFrame, Sequence, Skeleton}, animation_runtime::{Animations, Clock},")
rep("impl FileSpot {", r'''/// One of our OBJECTS from files: what an object definition names, loaded loose.
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

impl FileSpot {''')
rep("let scene = Scene::load_normal(", "let mut scene = Scene::load_normal(")
rep("    let mut pending_aspots = Vec::<(End, u16, u16, u16, u8)>::new();",
    "    let mut pending_aspots = Vec::<(End, u16, u16, u16, u8)>::new();\n    let mut loc_events = Vec::<(i32, LocEvent)>::new();\n    let mut flocs = Vec::<FLoc>::new();\n    let mut pin = false;\n    let mut shakes = Vec::<(usize, f32, i32, i32)>::new();")
rep('''            "render" => {''', r'''            "pin" => pin = true,
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
            "render" => {''')
rep("    let mesh = WorldMesh::build(&scene, &mut client.scripts.source.cache, 0)?;", r'''    let obj_defs = gielinor_client::object::Definition::load_all(&mut client.scripts.source.cache)?;
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
    let mut mesh = WorldMesh::build(&scene, &mut client.scripts.source.cache, 0)?;''')
rep("        // ---- the client's clocks: actors (sequences, poses, their spots), then projectiles, then tile graphics", r'''        // ---- objects: LOC_ADD_CHANGE / LOC_DEL / LOC_ANIM as live_runtime.rs applies them, then the mesh's clocks
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
        // ---- the client's clocks: actors (sequences, poses, their spots), then projectiles, then tile graphics''')
rep("        let mut temporary = 100000usize;", r'''        // file objects: scenery, submitted after the map's own and NOT registered with the actors' order (as mesh objects are not)
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
        let mut temporary = 100000usize;''')
# camera shake for films: recompute the camera per rendered frame
rep("        let mut frame = Frame::new(w, h)?;", r'''        let mut t = target;
        for &(axis, amp, s0, s1) in &shakes {
            if c >= s0 && c < s1 { let k = ((c as u32).wrapping_mul(2654435761) >> 16) & 1023; t[axis.min(2)] += amp * (k as f32 / 511.5 - 1.0); }
        }
        let mut camera = gielinor_client::render3d::Camera::look_at([x + yaw.sin() * p[3], target[1] - p[4], z - yaw.cos() * p[3]], t, focal.unwrap_or(1000. * w as f32 / 1280.))?;
        camera.far = 30000;
        let mut frame = Frame::new(w, h)?;''')
rep("        gielinor_client::model_order::rebase(&mut tris, mesh.triangles.len());", "        gielinor_client::model_order::rebase(&mut ltris, mesh.triangles.len());\n        gielinor_client::model_order::rebase(&mut tris, mesh.triangles.len() + ltris.len());")
rep("frame.draw_iter(&camera, mesh.visible_triangles(0).chain(tris.iter()), &palette, &textures);", "frame.draw_iter(&camera, mesh.visible_triangles(0).chain(ltris.iter()).chain(tris.iter()), &palette, &textures);")
(ROOT / 'asset_arena.rs').write_text(s, encoding='utf-8')
