//! RESEARCH tool (builds/research-items): item definitions and inventory icons through the client's own code.
//!
//! usage: research_items CACHE defs OUT.json
//!            every item record of the cache decoded by item.rs Definition::decode_local (the function
//!            Definitions::load uses), one JSON object per line, with the raw record as hex.
//!        research_items CACHE decode IN.bin OUT.json [plain]
//!            IN.bin = u32 count, then (u32 id, u32 length, bytes) records (the item_samples.rs layout); each is
//!            decoded by the same function (or by Definition::decode with `plain`) and written in the same JSON.
//!        research_items CACHE verify [all | ID ...]
//!            for each plain item (not a note, bought or placeholder template) draws the icon twice: by the client's
//!            Icons::get and by this file's `replica` (the same public calls: Models::inventory -> Model::light ->
//!            icon_vertices -> icon_depth_bounds -> Frame::draw_icon -> Pixels::outline/shadow), and counts pixels
//!            that differ. The replica exists only because Icons::get reads its model from the cache by id and
//!            cannot be handed one of ours.
//!        research_items CACHE icons OUT.bmp [outline=N] [shadow=HEX] [bg=HEX] [size=WxH] SPEC ...
//!            SPEC = item:ID                      a real item, by the client's Icons::get
//!                 | replica:ID                   the same item by this file's replica (for looking at the two together)
//!                 | def:FILE.def+model:PATH      one of ours: FILE.def is an encoded item definition (decoded by
//!                                                Definition::decode_local), PATH the inventory model file it names
//!            Writes a strip of cells, one per SPEC, left to right, no labels. Transparent pixels take bg.
//!            Prints for each the count of drawn pixels and their bounding box inside the cell.
use gielinor_client::{
    disk_cache::DiskCache, floor, item::{Definition, Definitions}, item_icon::{Icons, Options},
    item_model::{icon_depth_bounds, icon_vertices}, model::Model, object::Param,
    render3d::{texture_uv, Camera, Frame, TextureMapping, Triangle}, sprite_pixels::Pixels,
    texture::{self, Texture},
};
use std::{collections::HashMap, fmt::Write as _, io::Write as _};
type R<T> = Result<T, Box<dyn std::error::Error>>;

fn js(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 32 || (c as u32) > 126 => { let _ = write!(o, "\\u{:04x}", c as u32); }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}
fn opt(s: &Option<String>) -> String { s.as_ref().map_or("null".into(), |s| js(s)) }
fn def_json(d: &Definition, raw: &[u8]) -> String {
    let mut o = String::new();
    let _ = write!(o, "{{\"id\":{},\"name\":{},\"examine\":{},\"model\":{},\"zoom\":{},\"angles\":{:?},\"offset\":{:?},\"stackable\":{},\"price\":{},\"wear_positions\":{:?},\"members\":{},",
        d.id, js(&d.name), js(&d.examine), d.model, d.zoom, d.angles, d.offset, d.stackable, d.price, d.wear_positions, d.members);
    let pairs = |v: &Vec<(u16, u16)>| format!("[{}]", v.iter().map(|(a, b)| format!("[{a},{b}]")).collect::<Vec<_>>().join(","));
    let _ = write!(o, "\"recolor\":{},\"retexture\":{},\"shift_click\":{},\"wear_models\":{:?},\"wear_offset\":{:?},\"head_models\":{:?},\"category\":{},\"weight\":{},\"note\":{:?},\"bought\":{:?},\"placeholder\":{:?},",
        pairs(&d.recolor), pairs(&d.retexture), d.shift_click, d.wear_models, d.wear_offset, d.head_models, d.category, d.weight, d.note, d.bought, d.placeholder);
    let _ = write!(o, "\"counts\":[{}],\"scale\":{:?},\"ambient\":{},\"contrast\":{},\"team\":{},\"tradeable\":{},\"stockmarket\":{},\"searchable\":{},",
        d.counts.iter().map(|(a, b)| format!("[{a},{b}]")).collect::<Vec<_>>().join(","), d.scale, d.ambient, d.contrast, d.team, d.tradeable, d.stockmarket, d.searchable);
    let _ = write!(o, "\"inventory_operations\":[{}],", d.inventory_operations.iter().map(opt).collect::<Vec<_>>().join(","));
    let _ = write!(o, "\"ground_operations\":[{}],", d.ground_operations.iter().map(|g| format!("{{\"index\":{},\"sub\":{},\"condition\":{},\"text\":{}}}",
        g.index, g.sub.map_or("null".into(), |s| s.to_string()),
        g.condition.map_or("null".into(), |(a, b, c, lo, hi)| format!("[{a},{b},{},{lo},{hi}]", c.map_or("null".into(), |c| c.to_string()))), js(&g.text))).collect::<Vec<_>>().join(","));
    let _ = write!(o, "\"sub_operations\":[{}],", d.sub_operations.iter().map(|s| s.as_ref().map_or("null".into(), |v| format!("[{}]", v.iter().map(opt).collect::<Vec<_>>().join(",")))).collect::<Vec<_>>().join(","));
    let mut params: Vec<_> = d.params.iter().collect();
    params.sort_by_key(|p| *p.0);
    let _ = write!(o, "\"params\":[{}],", params.iter().map(|(k, v)| match v {
        Param::Int(i) => format!("[{k},\"int\",{i}]"),
        Param::Long(i) => format!("[{k},\"long\",\"{i}\"]"),
        Param::String(s) => format!("[{k},\"string\",{}]", js(s)),
    }).collect::<Vec<_>>().join(","));
    let hex = |b: &[u8]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
    let _ = write!(o, "\"trailing\":\"{}\",\"raw\":\"{}\"}}", hex(&d.trailing), hex(raw));
    o
}

struct Replica { palette: Vec<u32>, textures: HashMap<u16, Texture> }
impl Replica {
    fn load(cache: &mut DiskCache) -> R<Self> {
        let mut textures = HashMap::new();
        for (id, d) in texture::Definition::load_all(cache)? { textures.insert(id, Texture::load(cache, d, 128, 0.8)?); }
        Ok(Self { palette: floor::palette(0.8)?, textures })
    }
    /// item_icon.rs Icons::render for a plain item, line for line, with the model handed in (already scaled and
    /// recoloured, as Models::inventory returns it).
    fn render(&self, model: &Model, d: &Definition, options: Options, tag: u64) -> R<Pixels> {
        let [w, h] = options.size;
        let mut frame = Frame::new(w, h)?;
        frame.coarse_shading = false;
        frame.clear(0);
        let colors = model.light(d.ambient + 64, d.contrast + 768, [-50, -10, -50])?;
        let mut zoom = d.zoom;
        if w != 36 { zoom = zoom.wrapping_mul(32) / (w as i32); }
        if options.enlarged { zoom = (zoom as f64 * 1.5) as i32; } else if options.outline == 2 { zoom = (zoom as f64 * 1.04) as i32; }
        let vertices = icon_vertices(model, d, zoom)?;
        let mut triangles = Vec::new();
        let mut priorities = Vec::new();
        for (i, face) in model.faces.iter().enumerate() {
            let mut color = colors[i];
            if color[2] == -2 { continue; }
            if color[2] == -1 { color = [color[0]; 3]; }
            let positions = face.map(|v| vertices[v as usize]);
            let mapping = model.texture_coords.as_ref()
                .and_then(|c| if c[i] == -1 { None } else { model.texture_triangles.get(c[i] as u8 as usize) })
                .unwrap_or(face);
            priorities.push(model.priorities.as_ref().map_or(model.priority, |p| p[i]));
            triangles.push(Triangle {
                #[cfg(feature = "enhanced-renderer")] material_hsl: None,
                texture_mapping: Some(TextureMapping { vertices: mapping.map(|v| vertices[v as usize]), local: None }),
                local_vertices: None, model_bounds: None, model_order: None, depth_bias: model.face_depth_bias(i),
                view: 0, vertices: positions, colors: color, flat: colors[i][2] == -1,
                uv: texture_uv(mapping.map(|v| vertices[v as usize]), positions),
                texture: model.face_textures.as_ref().and_then(|t| (t[i] != -1).then_some(t[i] as u16)),
                alpha: model.face_alpha(i), near_alpha: Some(model.alpha.as_ref().map_or(0, |a| a[i])), tag,
            });
        }
        let mut camera = Camera::look_at([0.0; 3], [0.0, 0.0, 1.0], 512.0)?;
        camera.screen_offset = [(h >> 1) as f32 - (w / 2) as f32, (h >> 1) as f32 - (h / 2) as f32];
        let [radius, origin] = icon_depth_bounds(model, d, zoom);
        frame.draw_icon(&camera, &triangles, model.priorities.as_ref().map(|_| priorities.as_slice()), radius, origin, &self.palette, &self.textures);
        let mut sprite = Pixels { canvas: options.size, offset: [0; 2], size: options.size, pixels: frame.pixels };
        if options.outline >= 1 { sprite.outline(1); }
        if options.outline >= 2 { sprite.outline(0xffffff); }
        if options.shadow != 0 { sprite.shadow(options.shadow); }
        Ok(sprite)
    }
}
fn prepared(model: &Model, d: &Definition) -> Model {
    // item_model.rs Models::inventory: scale, then recolours, then retextures
    let mut m = model.clone();
    m.scale(d.scale);
    for &(from, to) in &d.recolor { m.recolor(from, to); }
    for &(from, to) in &d.retexture { m.retexture(from, to); }
    m
}
fn bmp(w: usize, h: usize, px: &[u32]) -> Vec<u8> {
    let row = (w * 3 + 3) / 4 * 4;
    let mut o = Vec::with_capacity(54 + row * h);
    o.extend(b"BM"); o.extend(((54 + row * h) as u32).to_le_bytes()); o.extend([0u8; 4]); o.extend(54u32.to_le_bytes());
    o.extend(40u32.to_le_bytes()); o.extend((w as i32).to_le_bytes()); o.extend((h as i32).to_le_bytes());
    o.extend(1u16.to_le_bytes()); o.extend(24u16.to_le_bytes()); o.extend([0u8; 24]);
    for y in (0..h).rev() {
        for x in 0..w { let p = px[y * w + x]; o.extend([(p & 255) as u8, (p >> 8 & 255) as u8, (p >> 16 & 255) as u8]); }
        o.extend(std::iter::repeat(0u8).take(row - w * 3));
    }
    o
}
fn main() -> R<()> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() < 3 { return Err("CACHE defs|decode|verify|icons ...".into()); }
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    match a[2].as_str() {
        "defs" => {
            let mut records = cache.group(2, 10, [0; 4])?;
            records.sort_by_key(|r| r.0);
            let mut out = std::io::BufWriter::new(std::fs::File::create(&a[3])?);
            for (id, data) in &records {
                let d = Definition::decode_local(*id, data).map_err(|e| format!("item {id}: {e}"))?;
                writeln!(out, "{}", def_json(&d, data))?;
            }
            println!("{} item records decoded", records.len());
        }
        "decode" => {
            let b = std::fs::read(&a[3])?;
            let plain = a.get(5).is_some_and(|s| s == "plain");
            let u = |p: usize| u32::from_be_bytes(b[p..p + 4].try_into().unwrap());
            let n = u(0);
            let mut p = 4;
            let mut out = std::io::BufWriter::new(std::fs::File::create(&a[4])?);
            for _ in 0..n {
                let (id, len) = (u(p), u(p + 4) as usize);
                let data = &b[p + 8..p + 8 + len];
                p += 8 + len;
                let d = if plain { Definition::decode(id, data) } else { Definition::decode_local(id, data) }.map_err(|e| format!("item {id}: {e}"))?;
                writeln!(out, "{}", def_json(&d, data))?;
            }
            println!("{n} records decoded ({})", if plain { "Definition::decode" } else { "Definition::decode_local" });
        }
        "verify" => {
            let defs = Definitions::load(&mut cache)?;
            let mut icons = Icons::load(&mut cache, 0.8)?;
            let replica = Replica::load(&mut cache)?;
            let mut models = gielinor_client::item_model::Models::default();
            let mut ids: Vec<u32> = if a.get(3).is_some_and(|s| s == "all") { defs.items.keys().copied().collect() } else { a[3..].iter().map(|s| s.parse()).collect::<Result<_, _>>()? };
            ids.sort();
            let listed = ids.len() <= 40;
            let (mut done, mut skipped, mut failed, mut bad, mut total_diff, mut drawn) = (0, 0, 0, 0, 0usize, 0usize);
            for id in ids {
                let d = &defs.items[&id];
                if d.note[1] != -1 || d.bought[1] != -1 || d.placeholder[1] != -1 { skipped += 1; continue; }
                let options = Options { quantity: 1, outline: 1, shadow: 0x302020, quantity_mode: 0, enlarged: false, size: [36, 32] };
                let theirs = match icons.get(&mut cache, &defs, id, options) { Ok(p) => p, Err(e) => { failed += 1; if listed { println!("item {id}: client icon error {e}"); } continue; } };
                let raw = models.raw(&mut cache, d.model as u32)?;
                let ours = replica.render(&prepared(&raw, d), d, options, id as u64 + 1)?;
                let diff = theirs.pixels.iter().zip(&ours.pixels).filter(|(x, y)| x != y).count();
                let on = theirs.pixels.iter().filter(|&&p| p != 0).count();
                done += 1; total_diff += diff; drawn += on; bad += usize::from(diff != 0);
                if listed || diff != 0 { println!("item {id} {:?}: {} of 1152 pixels drawn, {} differ between Icons::get and the replica", d.name, on, diff); }
            }
            println!("compared {done} items ({skipped} note/bought/placeholder templates skipped, {failed} the client could not draw): {bad} items differ, {total_diff} differing pixels of {} ({drawn} drawn)", done * 1152);
        }
        "icons" => {
            let defs = Definitions::load(&mut cache)?;
            let mut icons = Icons::load(&mut cache, 0.8)?;
            let replica = Replica::load(&mut cache)?;
            let mut models = gielinor_client::item_model::Models::default();
            let (mut outline, mut shadow, mut bg, mut size) = (1u8, 0x302020u32, 0x3e3529u32, [36usize, 32usize]);
            let mut specs = Vec::new();
            for e in &a[4..] {
                if let Some(v) = e.strip_prefix("outline=") { outline = v.parse()?; }
                else if let Some(v) = e.strip_prefix("shadow=") { shadow = u32::from_str_radix(v, 16)?; }
                else if let Some(v) = e.strip_prefix("bg=") { bg = u32::from_str_radix(v, 16)?; }
                else if let Some(v) = e.strip_prefix("size=") { let (w, h) = v.split_once('x').ok_or("size=WxH")?; size = [w.parse()?, h.parse()?]; }
                else { specs.push(e.clone()); }
            }
            let [w, h] = size;
            let mut px = vec![bg; w * specs.len() * h];
            let options = Options { quantity: 1, outline, shadow, quantity_mode: 0, enlarged: false, size };
            for (k, s) in specs.iter().enumerate() {
                let sprite = if let Some(id) = s.strip_prefix("item:") {
                    icons.get(&mut cache, &defs, id.parse()?, options)?
                } else if let Some(id) = s.strip_prefix("replica:") {
                    let id: u32 = id.parse()?;
                    let d = &defs.items[&id];
                    let raw = models.raw(&mut cache, d.model as u32)?;
                    replica.render(&prepared(&raw, d), d, options, id as u64 + 1)?
                } else if let Some(rest) = s.strip_prefix("def:") {
                    let (def, model) = rest.split_once("+model:").ok_or("def:FILE+model:PATH")?;
                    let d = Definition::decode_local(60000, &std::fs::read(def)?)?;
                    let m = Model::decode(&std::fs::read(model)?)?;
                    replica.render(&prepared(&m, &d), &d, options, 1)?
                } else { return Err(format!("bad spec {s}").into()); };
                let (mut n, mut lo, mut hi) = (0, [usize::MAX; 2], [0usize; 2]);
                for y in 0..h { for x in 0..w {
                    let p = sprite.pixels[y * w + x];
                    if p != 0 { px[y * w * specs.len() + k * w + x] = p; n += 1; lo = [lo[0].min(x), lo[1].min(y)]; hi = [hi[0].max(x), hi[1].max(y)]; }
                }}
                println!("{s}: {n} pixels drawn, box x {}..{} y {}..{}", lo[0], hi[0], lo[1], hi[1]);
            }
            std::fs::write(&a[3], bmp(w * specs.len(), h, &px))?;
        }
        _ => return Err("defs|decode|verify|icons".into()),
    }
    Ok(())
}
