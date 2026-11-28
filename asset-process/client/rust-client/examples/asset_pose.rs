//! Asset system tool: the standard figure posed at chosen frames of a sequence, exactly as the client poses a player.
//! usage: asset_pose CACHE OUT.bmp [seq=ID] [frames=F1,F2,..] [zoom=N] [yaw=N] [pitch=N] [cell=WxH]
//!                   [under=SEQ:FRAME] [center=body|model] [aim=head|body|legs|feet|hands|HEIGHT]
//!                   [draw=actor|depth|auto|widget] ENTRY [ENTRY...]
//!        asset_pose CACHE info SEQ|item:ID|skeleton:ID|kits [...]   (print what each is; draws nothing)
//!        `info kits` lists the body kits of the standard figure this tool draws (Appearance::standard: for each
//!        body part the lowest-numbered enabled kit of the gender), slot by slot: kit id, model ids, the kit's own
//!        recolours, the size and label histogram of the kit mesh, and the player recolours the figure applies.
//! One column per entry, one row per frame. A frame of `bind` (or no seq=) leaves the rest pose untouched.
//!
//! ENTRY  = PIECE[+PIECE...]   every piece is worn by the SAME figure, applied left to right (an outfit)
//! PIECE  = none                        the bare figure
//!        | item:ID                     a real item in the slot its definition declares (wear position 1), hiding
//!                                      the slots it declares as wear positions 2 and 3; an item that declares
//!                                      no slot goes in the weapon slot, which is what item:ID always meant
//!        | item:ID@SLOT                the item in SLOT, still hiding its declared wear positions 2 and 3
//!        | item:ID@SLOT+HIDE[+HIDE]    the item in SLOT, hiding exactly the listed slots instead
//!        | item:ID@SLOT+nohide         the item in SLOT, hiding nothing else
//!        | model:PATH                  one of our models merged after the finished body (nothing hidden; the old form)
//!        | model:PATH@SLOT[+HIDE...]   one of our models standing in SLOT's place in the assembly; the body kit
//!                                      of SLOT and of every HIDE slot is removed, as for a real item there
//! SLOT / HIDE = index 0..11 or name: 0 head, 1 cape, 2 amulet, 3 weapon, 4 body (torso), 5 shield, 6 arms,
//!        7 legs, 8 hair, 9 hands, 10 feet, 11 jaw (beard). After an `@`, a `+word` that is a slot name or
//!        `nohide` belongs to that piece; a `+item:`, `+model:` or `+none` starts the next piece.
//!        e.g. item:1127@body+item:1079@legs+model:path/to/helm.bin@head+hair+jaw
//!
//! How the client hides the body: there is no geometric test. A player's appearance is twelve slot values
//! (player_model.rs `equipment: [u32; 12]`): 0 = nothing, 256.. = body kit + 256, 2048.. = item + 2048. A slot
//! holds ONE thing, so an item put in slot 4 simply takes the torso kit's place, and the item definition's
//! second and third wear positions name further slots to blank (widget_appearance.rs Composition::equip:
//! `equipment[slot] = item + 2048; equipment[hidden] = 0`). Appearance::mesh then combines whatever the twelve
//! slots hold, in slot order. For remote players the server sends the twelve values already blanked.
//!
//! The mesh is assembled first, lit once in the rest pose, and only then posed by Animations::apply_actor,
//! which is the order the client's world_actors uses, so a merged model is moved by its own vertex labels like
//! any worn item. With no model:PATH@SLOT piece the mesh is the client's own Appearance::mesh. With one, this
//! file repeats that function's loop with our model standing in its slot (same slot order, same Model::combine,
//! same player recolours applied to the whole mesh, ours included, as they are to a real item).
//! under=SEQ:FRAME supplies the locomotion sequence the client blends beneath an action (walk under attack).
//! draw= chooses the routine that paints the figure (examples/support/actor_draw.rs has the evidence):
//!   actor  (default) as the live client paints a player in the world: model_order::Mode::Priority. Faces are
//!          painted by priority group, far to near inside a group, with NO depth test between the faces of the
//!          figure, so a face of higher priority paints over a lower one wherever both are drawn, in front or
//!          behind, and a strip lying exactly in a surface shows if its priority is higher.
//!   depth  the old picture: every face depth-tested per pixel against the others, priorities ignored.
//!   auto   as the live client paints an ordinary NPC or a scenery object (Mode::Auto): per-pixel depth, faces
//!          in model order, priorities ignored, per-face depth offsets honoured.
//!   widget as the client's interface paints a player model (equipment screen): priority order, no depth at all.
//! center=body (default) frames every cell on the bare rest figure so columns and rows register pixel for
//! pixel; center=model frames each cell on its own rest mesh, which is what asset_figure does.
//! aim=HEIGHT (worn-space height in units, feet 0, crown 196) or aim=head (180) | body (140) | hands (100) |
//! legs (65) | feet (10) points the camera at that height of the figure instead of the middle of the bare figure
//! (98). Every cell still shares the one camera, so columns and rows register pixel for pixel; with a small
//! zoom= this gives a close view of a helm or of boots. aim= overrides center=.
use gielinor_client::{
    actor_sequence::State, animation_runtime::{Animations, Applied, Clock}, disk_cache::DiskCache,
    identity_kit::Kit, interface_model, item::Definitions, item_icon::Icons, item_model::Models,
    model::Model, player_colors, player_model::{Appearance, KIT_SLOTS}, raster::Raster,
};
use std::collections::{BTreeMap, HashMap, HashSet};
#[path = "support/actor_draw.rs"]
pub mod actor_draw;
pub type R<T> = Result<T, Box<dyn std::error::Error>>;
/// Equipment slot names in the client's order. Evidence in the source: player_model.rs KIT_SLOTS
/// `[8, 11, 4, 6, 9, 7, 10]` maps the body-kit parts hair, jaw, torso, arms, hands, legs, feet to slots;
/// Appearance::sequence_equipment writes a sequence's shield to slot 5 and its weapon to slot 3;
/// widget_appearance.rs reads the head item from slot 0 and treats 11 as the beard. Capes (1) and amulets (2)
/// are confirmed by the wear position 1 that real capes and amulets declare.
pub const SLOT_NAMES: [&str; 12] = ["head", "cape", "amulet", "weapon", "body", "shield", "arms", "legs", "hair", "hands", "feet", "jaw"];
pub fn slot_index(s: &str) -> Option<usize> {
    if let Ok(n) = s.parse::<usize>() {
        return (n < 12).then_some(n);
    }
    match s {
        "torso" | "chest" => Some(4),
        "beard" => Some(11),
        "gloves" => Some(9),
        "boots" => Some(10),
        "helm" | "helmet" => Some(0),
        _ => SLOT_NAMES.iter().position(|n| *n == s),
    }
}
fn names(slots: &[usize]) -> String {
    slots.iter().map(|&s| SLOT_NAMES[s]).collect::<Vec<_>>().join("+")
}
/// One figure's worth of an entry: the twelve slot values, our models standing in slots, our models merged
/// after the body, and the worn meshes (by slot) used to report labels and bounding boxes.
pub struct Outfit {
    pub equipment: [u32; 12],
    pub slot_models: [Option<Model>; 12],
    pub loose: Vec<Model>,
    pub worn_items: Vec<(usize, u32, Model)>,
    pub notes: Vec<String>,
}
/// Splits `SPEC@SLOT+HIDE..` into the spec, the slot and the explicit hide list (None when none was written).
fn placement(piece: &str, rest: &[&str]) -> R<(String, Option<usize>, Option<Vec<usize>>)> {
    let (spec, slot) = match piece.rsplit_once('@') {
        Some((spec, slot)) => match slot_index(slot) {
            Some(s) => (spec, Some(s)),
            None => return Err(format!("unknown slot `{slot}` in `{piece}` (use 0..11 or {})", SLOT_NAMES.join(", ")).into()),
        },
        None => (piece, None),
    };
    if rest.is_empty() {
        return Ok((spec.to_string(), slot, None));
    }
    if slot.is_none() {
        return Err(format!("`+{}` after `{piece}`: hidden slots need an @SLOT first", rest[0]).into());
    }
    let mut hidden = Vec::new();
    for h in rest {
        if *h != "nohide" {
            hidden.push(slot_index(h).ok_or(format!("unknown slot `{h}` after `{piece}`"))?);
        }
    }
    Ok((spec.to_string(), slot, Some(hidden)))
}
pub fn outfit(entry: &str, standard: &Appearance, items: &Definitions, models: &mut Models, cache: &mut DiskCache) -> R<Outfit> {
    let mut o = Outfit { equipment: standard.equipment, slot_models: Default::default(), loose: Vec::new(), worn_items: Vec::new(), notes: Vec::new() };
    // group the `+` separated tokens: a token that is not item:/model:/none continues the piece before it
    let mut pieces: Vec<(&str, Vec<&str>)> = Vec::new();
    for token in entry.split('+') {
        if token.starts_with("item:") || token.starts_with("model:") || token == "none" {
            pieces.push((token, Vec::new()));
        } else {
            match pieces.last_mut() {
                Some((_, rest)) => rest.push(token),
                None => return Err(format!("bad entry {entry}").into()),
            }
        }
    }
    for (piece, rest) in pieces {
        if piece == "none" {
            if !rest.is_empty() { return Err(format!("bad entry {entry}").into()); }
            continue;
        }
        let (slot, hidden);
        if let Some(body) = piece.strip_prefix("item:") {
            let (spec, at, explicit) = placement(body, &rest)?;
            let id = spec.parse::<u32>().map_err(|_| format!("bad item id in `{piece}`"))?;
            let d = items.items.get(&id).ok_or(format!("no item {id}"))?;
            let declared: Vec<usize> = d.wear_positions[1..].iter().filter(|&&p| (0..12).contains(&p)).map(|&p| p as usize).collect();
            slot = match at {
                Some(s) => s,
                None if (0..12).contains(&d.wear_positions[0]) => d.wear_positions[0] as usize,
                None => 3,
            };
            hidden = explicit.clone().unwrap_or(declared);
            let why = match (at, explicit.is_some()) {
                (None, _) if (0..12).contains(&d.wear_positions[0]) => "slot and hidden slots from the item definition",
                (None, _) => "definition declares no wear position: weapon slot",
                (Some(_), false) => "slot as given, hidden slots from the item definition",
                (Some(_), true) => "slot and hidden slots as given",
            };
            o.notes.push(format!("item {id} {:?} in {} ({slot}), hides [{}] ({why}; definition wear_positions {:?})", d.name, SLOT_NAMES[slot], names(&hidden), d.wear_positions));
            o.equipment[slot] = id + 2048;
            o.slot_models[slot] = None;
            o.worn_items.retain(|(s, _, _)| *s != slot);
            if let Some(m) = models.wearable(cache, d, 0, false, None)? {
                o.worn_items.push((slot, id, m));
            }
        } else if let Some(body) = piece.strip_prefix("model:") {
            // a Windows path may itself hold no `@`; if the text after the last `@` is not a slot, placement reports it
            let (path, at, explicit) = if body.contains('@') { placement(body, &rest)? } else {
                if !rest.is_empty() { return Err(format!("`+{}` after `{piece}`: hidden slots need an @SLOT first", rest[0]).into()); }
                (body.to_string(), None, None)
            };
            let m = Model::decode(&std::fs::read(&path).map_err(|e| format!("{path}: {e}"))?)?;
            match at {
                None => {
                    o.notes.push(format!("model {path} merged after the body, nothing hidden"));
                    o.loose.push(m);
                    continue;
                }
                Some(s) => {
                    slot = s;
                    hidden = explicit.unwrap_or_default();
                    o.notes.push(format!("model {path} in {} ({slot}), hides [{}]", SLOT_NAMES[slot], names(&hidden)));
                    o.equipment[slot] = 0;
                    o.worn_items.retain(|(s, _, _)| *s != slot);
                    o.slot_models[slot] = Some(m);
                }
            }
        } else {
            return Err(format!("bad entry {entry}").into());
        }
        // Composition::equip: the second and third wear positions blank those slots
        for h in hidden {
            if h != slot {
                o.equipment[h] = 0;
                o.slot_models[h] = None;
                o.worn_items.retain(|(s, _, _)| *s != h);
            }
        }
    }
    Ok(o)
}
/// The player mesh for `player` (whose equipment already holds the outfit's slot values).
pub fn assemble(o: &Outfit, player: &Appearance, cache: &mut DiskCache, models: &mut Models, kits: &HashMap<u32, Kit>, items: &Definitions) -> R<Model> {
    let body = if o.slot_models.iter().all(|m| m.is_none()) {
        player.mesh(cache, models, kits, items, false)?
    } else {
        // Appearance::mesh_with_colors, with our models standing in their slots
        let mut parts = vec![];
        for (slot, &id) in player.equipment.iter().enumerate() {
            if let Some(m) = &o.slot_models[slot] {
                parts.push(m.clone());
            } else if (256..2048).contains(&id) {
                if let Some(kit) = kits.get(&(id - 256)) {
                    if let Some(m) = kit.mesh(cache, models, false)? {
                        parts.push(m);
                    }
                }
            } else if id >= 2048 {
                let d = items.items.get(&(id - 2048)).ok_or("appearance item")?;
                if let Some(m) = models.wearable(cache, d, player.gender as usize, false, None)? {
                    parts.push(m);
                }
            }
        }
        let mut model = Model::combine(&parts)?;
        for i in 0..5 {
            let index = player.colors[i] as usize;
            if let Some(&to) = player_colors::COLORS[i].get(index) {
                model.recolor(player_colors::BASE[i], to);
            }
            if let Some(&to) = player_colors::SECONDARY_COLORS[i].get(index) {
                model.recolor(player_colors::SECONDARY_BASE[i], to);
            }
        }
        model
    };
    if o.loose.is_empty() {
        return Ok(body);
    }
    let mut parts = vec![body];
    parts.extend(o.loose.iter().cloned());
    Ok(Model::combine(&parts)?)
}
pub fn standard_figure(kits: &HashMap<u32, Kit>) -> Appearance {
    let mut standard = Appearance::standard(kits, 0);
    standard.colors = [3, 2, 3, 1, 0];
    standard
}
/// aim=: a named region of the standard figure or a height in worn space (units above the feet).
fn aim_height(v: &str) -> R<i32> {
    Ok(match v {
        "head" => 180,
        "body" | "torso" | "chest" => 140,
        "hands" => 100,
        "legs" => 65,
        "feet" => 10,
        _ => v.parse::<i32>().map_err(|_| format!("aim={v}: use head, body, hands, legs, feet or a height in units"))?,
    })
}
fn half_height(m: &Model) -> i32 {
    m.vertices.iter().map(|v| -(v[1] as i32)).max().unwrap_or(0).max(0) / 2
}
fn histogram(values: Option<&Vec<u8>>, count: usize) -> String {
    match values {
        None => format!("none stored (all {count} read as 0)"),
        Some(v) => {
            let mut h = BTreeMap::<u8, usize>::new();
            for &x in v { *h.entry(x).or_default() += 1; }
            format!("{h:?}")
        }
    }
}
fn info(cache: &mut DiskCache, ids: &[String]) -> R<()> {
    let mut animations = Animations::load(cache)?;
    let items = Definitions::load(cache)?;
    let mut models = Models::default();
    for id in ids {
        if id == "kits" {
            let kits = Kit::load_all(cache)?;
            let standard = standard_figure(&kits);
            println!("standard figure: gender {} colors {:?} equipment {:?}", standard.gender, standard.colors, standard.equipment);
            for i in 0..5 {
                let index = standard.colors[i] as usize;
                if let Some(&to) = player_colors::COLORS[i].get(index) {
                    println!("player_recolor part={i} from={} to={to}", player_colors::BASE[i]);
                }
                if let Some(&to) = player_colors::SECONDARY_COLORS[i].get(index) {
                    println!("player_recolor part={i} from={} to={to}", player_colors::SECONDARY_BASE[i]);
                }
            }
            for (part, &slot) in KIT_SLOTS.iter().enumerate() {
                let value = standard.equipment[slot];
                if !(256..2048).contains(&value) {
                    println!("kit slot={slot} name={} part={part} id=none", SLOT_NAMES[slot]);
                    continue;
                }
                let kit = kits.get(&(value - 256)).ok_or("standard kit")?;
                let m = kit.mesh(cache, &mut models, false)?.ok_or("standard kit has no mesh")?;
                println!("kit slot={slot} name={} part={part} id={} models={:?} recolor={:?} retexture={:?} vertices={} faces={}",
                    SLOT_NAMES[slot], kit.id, kit.models, kit.recolor, kit.retexture, m.vertices.len(), m.faces.len());
                println!("    vertex labels {}", histogram(m.vertex_skins.as_ref(), m.vertices.len()));
                println!("    face labels   {}", histogram(m.face_skins.as_ref(), m.faces.len()));
                match &m.priorities {
                    Some(p) => println!("    face priorities {}", histogram(Some(p), m.faces.len())),
                    None => println!("    face priorities uniform {}", m.priority),
                }
            }
            continue;
        }
        if let Some(item) = id.strip_prefix("item:") {
            let Some(d) = items.items.get(&item.parse::<u32>()?) else {
                println!("item {item}: missing");
                continue;
            };
            println!("item {item}: {:?} wear_models={:?} params={:?}", d.name, d.wear_models[0], d.params);
            let slots = d.wear_positions.iter().map(|&p| if (0..12).contains(&p) { format!("{p} {}", SLOT_NAMES[p as usize]) } else { p.to_string() }).collect::<Vec<_>>();
            println!("  wear_positions [{}]  (first = slot worn in, second and third = slots it blanks)", slots.join(", "));
            println!("  wear_models male {:?} female {:?}  wear_offset(y) {:?}  head_models male {:?} female {:?}", d.wear_models[0], d.wear_models[1], d.wear_offset, d.head_models[0], d.head_models[1]);
            println!("  recolor (from, to) {:?}  retexture {:?}  inventory model {} scale {:?}", d.recolor, d.retexture, d.model, d.scale);
            for (g, name) in ["male", "female"].iter().enumerate() {
                let ids = d.wear_models[g];
                for (n, &mid) in ids.iter().enumerate() {
                    if mid == -1 { continue; }
                    // item_model.rs wearable: the third model is loaded only when a second exists
                    let used = ids[0] != -1 && (n < 1 || ids[1] != -1);
                    let m = models.raw(cache, mid as u32)?;
                    let (mut lo, mut hi) = ([i32::MAX; 3], [i32::MIN; 3]);
                    for v in &m.vertices {
                        for x in 0..3 { lo[x] = lo[x].min(v[x] as i32); hi[x] = hi[x].max(v[x] as i32); }
                    }
                    println!("  {name} worn model {mid}{}: {} vertices, {} faces, format {:?}, box min {lo:?} max {hi:?}", if used { "" } else { " (NOT loaded by the client)" }, m.vertices.len(), m.faces.len(), m.format);
                    println!("    vertex labels {}", histogram(m.vertex_skins.as_ref(), m.vertices.len()));
                    println!("    face labels   {}", histogram(m.face_skins.as_ref(), m.faces.len()));
                    match &m.priorities {
                        Some(p) => println!("    face priorities {}", histogram(Some(p), m.faces.len())),
                        None => println!("    face priorities uniform {}", m.priority),
                    }
                    let mut colours = BTreeMap::<u16, usize>::new();
                    for &c in &m.colors { *colours.entry(c).or_default() += 1; }
                    println!("    face colours (raw, before the definition's recolours) {colours:?}; alpha {} textures {}", m.alpha.is_some(), m.face_textures.as_ref().is_some_and(|t| t.iter().any(|&x| x != -1)));
                }
            }
            continue;
        }
        if let Some(sk) = id.strip_prefix("skeleton:") {
            // skeleton:ID lists the transform groups (type 0 pivot, 1 move, 2 turn, 3 scale, 5 alpha) and their labels
            let sk = sk.parse::<u16>()?;
            let s = gielinor_client::animation::Skeleton::decode(sk, &cache.file(1, sk as u32, 0, [0; 4])?)?;
            for (i, (kind, labels)) in s.transform_types.iter().zip(&s.labels).enumerate() {
                println!("skeleton {sk} group {i}: type {kind} labels {labels:?}");
            }
            continue;
        }
        let id = id.parse::<u32>()?;
        let Some(s) = animations.sequences.get(&id).cloned() else {
            println!("seq {id}: missing");
            continue;
        };
        let mut probe = Model::combine(&[])?;
        let applied = animations.apply_observed(cache, &mut probe, id, &Clock::default())?;
        println!(
            "seq {id}: frames={} lengths={:?} total={} loop={} priority={} weapon={} shield={} interleave={} bone_mask={} cached={:?} range={:?} first={:?}",
            s.frames.len(), s.lengths, s.total_length, s.loop_frames, s.priority, s.weapon, s.shield,
            s.interleave.as_ref().map_or("no".to_string(), |v| format!("{:?}", &v[..v.len() - 1])),
            s.bone_mask.is_some(), s.cached_animation, s.cached_range, applied
        );
    }
    Ok(())
}
#[allow(dead_code)]
fn main() -> R<()> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() < 4 {
        return Err("CACHE OUT.bmp [seq=ID] [frames=F1,F2,..] [zoom=N] [yaw=N] [pitch=N] [cell=WxH] [under=SEQ:FRAME] [center=body|model] [aim=head|body|legs|feet|hands|HEIGHT] [draw=actor|depth|auto|widget] ENTRY [ENTRY...]".into());
    }
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    if a[2] == "info" {
        return info(&mut cache, &a[3..]);
    }
    let kits = Kit::load_all(&mut cache)?;
    let items = Definitions::load(&mut cache)?;
    let mut models = Models::default();
    let icons = Icons::load(&mut cache, 0.8)?;
    let mut animations = Animations::load(&mut cache)?;
    let (mut zoom, mut yaw, mut pitch) = (520i32, 192i32, 0i32);
    let mut seq: Option<u32> = None;
    let mut frames: Vec<Option<usize>> = Vec::new();
    let mut under: Option<(u32, usize)> = None;
    let mut cell: Option<(usize, usize)> = None;
    let mut center_model = false;
    let mut aim: Option<i32> = None;
    let mut draw = actor_draw::Draw::Actor;
    let mut entries = Vec::new();
    for e in &a[3..] {
        if let Some(v) = e.strip_prefix("zoom=") { zoom = v.parse()?; }
        else if let Some(v) = e.strip_prefix("yaw=") { yaw = v.parse()?; }
        else if let Some(v) = e.strip_prefix("pitch=") { pitch = v.parse()?; }
        else if let Some(v) = e.strip_prefix("seq=") { seq = Some(v.parse()?); }
        else if let Some(v) = e.strip_prefix("frames=") {
            for f in v.split(',') {
                frames.push(if f == "bind" { None } else { Some(f.parse()?) });
            }
        }
        else if let Some(v) = e.strip_prefix("under=") {
            let (s, f) = v.split_once(':').ok_or("under=SEQ:FRAME")?;
            under = Some((s.parse()?, f.parse()?));
        }
        else if let Some(v) = e.strip_prefix("cell=") {
            let (w, h) = v.split_once('x').ok_or("cell=WxH")?;
            cell = Some((w.parse()?, h.parse()?));
        }
        else if let Some(v) = e.strip_prefix("center=") {
            center_model = match v { "model" => true, "body" => false, _ => return Err("center=body|model".into()) };
        }
        else if let Some(v) = e.strip_prefix("aim=") { aim = Some(aim_height(v)?); }
        else if let Some(v) = e.strip_prefix("draw=") { draw = actor_draw::Draw::parse(v)?; }
        else { entries.push(e.clone()); }
    }
    println!("draw={draw:?}");
    if let Some(h) = aim { println!("aim: camera pointed at height {h} (every cell shares it)"); }
    let mut painter = actor_draw::Painter::load(&mut cache)?;
    if entries.is_empty() { return Err("no entries".into()); }
    if frames.is_empty() { frames.push(if seq.is_some() { Some(0) } else { None }); }
    if seq.is_none() && frames.iter().any(|f| f.is_some()) { return Err("frames need seq=".into()); }
    let (cw, ch) = cell.unwrap_or(if zoom < 400 { (600, 800) } else { (300, 400) });
    let sequence = match seq {
        Some(id) => Some(animations.sequences.get(&id).ok_or(format!("no sequence {id}"))?.clone()),
        None => None,
    };
    if let Some(s) = &sequence {
        println!(
            "seq {}: frames={} lengths={:?} weapon={} shield={} interleave={} cached={:?}",
            s.id, s.frames.len(), s.lengths, s.weapon, s.shield, s.interleave.is_some(), s.cached_animation
        );
    }
    let standard = standard_figure(&kits);
    let bare = standard.mesh(&mut cache, &mut models, &kits, &items, false)?;
    let mut raster = Raster::new(cw * entries.len(), ch * frames.len(), 1 << 26)?;
    raster.clear(0xff302b25);
    for (k, e) in entries.iter().enumerate() {
        for (r, frame) in frames.iter().enumerate() {
            let mut o = outfit(e, &standard, &items, &mut models, &mut cache)?;
            if r == 0 {
                for n in &o.notes { println!("{e}: {n}"); }
            }
            let mut player = standard.clone();
            player.equipment = o.equipment;
            // GamePlayer: an action sequence may replace the weapon and shield slots while it plays.
            let mut replaced = false;
            if let (Some(s), Some(_)) = (&sequence, frame) {
                let swapped = player.sequence_equipment(s);
                if swapped.equipment != player.equipment {
                    replaced = true;
                    for slot in [3, 5] {
                        if swapped.equipment[slot] != player.equipment[slot] {
                            o.slot_models[slot] = None;
                            o.worn_items.retain(|(s, _, _)| *s != slot);
                        }
                    }
                    player = swapped;
                }
            }
            let mut m = assemble(&o, &player, &mut cache, &mut models, &kits, &items)?;
            // which merged vertices came from the worn pieces (combine merges equal coordinates, so match by position)
            let mut set: HashSet<[i32; 3]> = HashSet::new();
            for w in o.worn_items.iter().map(|(_, _, w)| w).chain(o.slot_models.iter().flatten()).chain(o.loose.iter()) {
                set.extend(w.vertices.iter().map(|v| v.map(|c| c as i32)));
            }
            let worn_indices: Vec<usize> = m.vertices.iter().enumerate().filter(|(_, v)| set.contains(&v.map(|c| c as i32))).map(|(i, _)| i).collect();
            let rest_half = aim.unwrap_or_else(|| half_height(if center_model { &m } else { &bare }));
            // lit once in the rest pose, as the client lights the cached player mesh before animating a copy
            let colors = m.light(64, 850, [-30, -50, -30])?;
            let mut note = String::from("bind");
            if let (Some(s), Some(f)) = (&sequence, frame) {
                let count = if s.cached_animation.is_some() { (s.cached_range[1].saturating_sub(s.cached_range[0])) as usize } else { s.frames.len() };
                if *f >= count { return Err(format!("sequence {} has {count} frames, asked for {f}", s.id).into()); }
                let action = State { id: s.id as i32, frame: *f as i32, ..Default::default() };
                match under {
                    Some((us, uf)) => {
                        let pose = State { id: us as i32, frame: uf as i32, ..Default::default() };
                        animations.apply_actor(&mut cache, &mut m, Some(&action), Some(&pose), -1)?;
                        note = format!("blended over {us}:{uf}");
                    }
                    None => {
                        note = match animations.apply_observed(&mut cache, &mut m, s.id, &action.clock())? {
                            Applied::Classic { frame_id, skeleton_id, .. } => format!("classic frame {}:{} skeleton {skeleton_id}", frame_id >> 16, frame_id & 65535),
                            Applied::Cached { animation_id, skeleton_id, .. } => format!("cached animation {animation_id} skeleton {skeleton_id}"),
                            other => return Err(format!("frame not applied: {other:?}").into()),
                        };
                    }
                }
            }
            let label = frame.map_or("bind".to_string(), |f| f.to_string());
            let total = format!("{} vertices {} faces", m.vertices.len(), m.faces.len());
            let swap = if replaced { "; weapon replaced by the sequence" } else { "" };
            if worn_indices.is_empty() {
                println!("{e} frame {label}: {note}; {total}{swap}");
            } else {
                let (mut lo, mut hi) = ([i32::MAX; 3], [i32::MIN; 3]);
                let mut skins = BTreeMap::<u8, usize>::new();
                for &i in &worn_indices {
                    for x in 0..3 {
                        lo[x] = lo[x].min(m.vertices[i][x] as i32);
                        hi[x] = hi[x].max(m.vertices[i][x] as i32);
                    }
                    *skins.entry(m.vertex_skins.as_ref().map_or(0, |s| s[i])).or_default() += 1;
                }
                println!("{e} frame {label}: {note}; worn {} vertices, labels {skins:?}, box min {lo:?} max {hi:?}; figure {total}{swap}", worn_indices.len());
            }
            // interface_model centres on the mesh's own height; feed it the rest height instead so every cell
            // shares one camera (y is untouched by the yaw turn, so shifting it first is the same arithmetic)
            let mut placed = m.clone();
            for v in &mut placed.vertices { v[1] += rest_half as f32; }
            let vertices = interface_model::vertices(&placed, [pitch, yaw, 0], [0, 0], zoom, false)?;
            let origin = actor_draw::origin_depth(rest_half, pitch, zoom);
            let (sprite, warning) = painter.draw(&mut cache, &icons, &m, &colors, &vertices, origin, [cw, ch], draw)?;
            if let Some(w) = warning { println!("{e} frame {label}: WARNING {w}"); }
            sprite.draw(&mut raster, (k * cw) as i32, (r * ch) as i32, 256)?;
        }
    }
    std::fs::write(&a[2], raster.bmp()?)?;
    Ok(())
}
