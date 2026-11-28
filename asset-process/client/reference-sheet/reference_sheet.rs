//! Local-only rev239 reference archive. No network, native window, or cache writes.
use gielinor_client::{
    disk_cache::DiskCache,
    font::{Font, Style},
    item_model::Models,
    model::Model,
    npc, npc_model,
    raster::Raster,
    render3d::{texture_uv, Camera, Frame, TextureMapping, Triangle},
    texture,
};
use std::{
    collections::{BTreeSet, HashMap},
    io::Write,
    path::Path,
};
type E = Box<dyn std::error::Error>;
fn q(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if c < ' ' => o.push_str(&format!("\\u{:04x}", c as u32)),
            _ => o.push(c),
        }
    }
    o.push('"');
    o
}
fn pairs(v: &[(u16, u16)]) -> String {
    format!(
        "[{}]",
        v.iter()
            .map(|(a, b)| format!("[{a},{b}]"))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn text(r: &mut Raster, f: &Font, s: &str, x: i32, y: i32) -> Result<(), E> {
    f.draw(
        r,
        s,
        x,
        y,
        Style {
            color: 0xe5e9ed,
            shadow: None,
            alpha: 256,
        },
        &[],
    )?;
    Ok(())
}
fn triangles(m: &Model, colors: &[[i32; 3]]) -> Vec<Triangle> {
    m.faces
        .iter()
        .enumerate()
        .filter_map(|(i, face)| {
            if colors[i][2] == -2 {
                return None;
            }
            let flat = colors[i][2] == -1;
            let vertices = face.map(|v| m.vertices[v as usize]);
            let map = m
                .texture_coords
                .as_ref()
                .and_then(|v| {
                    if v[i] == -1 {
                        None
                    } else {
                        m.texture_triangles.get(v[i] as u8 as usize)
                    }
                })
                .unwrap_or(face)
                .map(|v| m.vertices[v as usize]);
            Some(Triangle {
                texture_mapping: Some(TextureMapping {
                    vertices: map,
                    local: Some(map),
                }),
                near_alpha: Some(m.alpha.as_ref().map_or(0, |a| a[i])),
                local_vertices: None,
                model_bounds: None,
                model_order: Some(gielinor_client::model_order::Submission::new(m, i, 0)),
                depth_bias: m.face_depth_bias(i),
                view: 0,
                vertices,
                colors: if flat { [colors[i][0]; 3] } else { colors[i] },
                flat,
                uv: texture_uv(map, vertices),
                texture: m.face_textures.as_ref().and_then(|v| {
                    if v[i] == -1 {
                        None
                    } else {
                        Some(v[i] as u16)
                    }
                }),
                alpha: m.face_alpha(i),
                tag: 1,
            })
        })
        .collect()
}
fn sheet(
    cache: &mut DiskCache,
    font: &Font,
    m: &Model,
    colors: &[[i32; 3]],
    header: &[String],
) -> Result<(Raster, usize, [f32; 3], [f32; 3]), E> {
    if m.vertices.is_empty() || m.faces.is_empty() {
        return Err("empty geometry".into());
    }
    let mut lo = [f32::INFINITY; 3];
    let mut hi = [f32::NEG_INFINITY; 3];
    for v in &m.vertices {
        for i in 0..3 {
            lo[i] = lo[i].min(v[i]);
            hi[i] = hi[i].max(v[i]);
        }
    }
    let center = std::array::from_fn(|i| (lo[i] + hi[i]) * 0.5);
    let radius = m
        .vertices
        .iter()
        .map(|v| {
            (0..3)
                .map(|i| (v[i] - center[i]).powi(2))
                .sum::<f32>()
                .sqrt()
        })
        .fold(1f32, f32::max);
    let divisor = radius * 2.5;
    let ts = triangles(m, colors);
    let defs = texture::Definition::load_all(cache)?;
    let mut textures = HashMap::new();
    for id in ts.iter().filter_map(|t| t.texture).collect::<BTreeSet<_>>() {
        textures.insert(
            id,
            texture::Texture::load(
                cache,
                defs.get(&id).ok_or("missing texture definition")?.clone(),
                128,
                0.8,
            )?,
        );
    }
    let palette = gielinor_client::floor::palette(0.8)?;
    let w = 1056;
    let mut lines = Vec::new();
    for line in header {
        let mut current = String::new();
        for word in line.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_string()
            } else {
                format!("{current} {word}")
            };
            if font.metrics.width(&candidate, &[]) > 1028 && !current.is_empty() {
                lines.push(current);
                current = word.to_string();
            } else {
                current = candidate;
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines.push(format!(
        "Bounds min {lo:?}, max {hi:?}; extents {:?} cache units",
        std::array::from_fn::<_, 3, _>(|i| hi[i] - lo[i])
    ));
    let header_h = 40 + lines.len() * 18;
    let h = header_h + 740;
    let mut pixels = vec![0xff171e27; w * h];
    let mut submitted = 0;
    let views = [
        ("FRONT (-Z)", [0., 0., -1.]),
        ("BACK (+Z)", [0., 0., 1.]),
        ("LEFT (-X)", [-1., 0., 0.]),
        ("RIGHT (+X)", [1., 0., 0.]),
        ("THREE QUARTER", [-0.65, -0.45, -0.65]),
        ("TOP", [0., -1., -0.001]),
    ];
    for (n, (_, direction)) in views.iter().enumerate() {
        let norm = direction.iter().map(|v| v * v).sum::<f32>().sqrt();
        let eye = std::array::from_fn(|i| center[i] + direction[i] / norm * (radius * 5. + 100.));
        let mut cam = Camera::look_at(eye, center, 300.)?;
        cam.far = 100000;
        cam.orthographic = Some(divisor);
        let mut frame = Frame::new(352, 352)?;
        frame.clear(0xff202a35);
        let mut grid_units = 32.;
        while grid_units * 300. / divisor < 16. {
            grid_units *= 2.;
        }
        let grid_step = grid_units * 300. / divisor;
        for y in 0..352 {
            for x in 0..352 {
                let dx = ((x as f32 - 176.) / grid_step).round() * grid_step + 176.;
                let dy = ((y as f32 - 176.) / grid_step).round() * grid_step + 176.;
                if (x as f32 - dx).abs() < 0.5 || (y as f32 - dy).abs() < 0.5 {
                    frame.pixels[y * 352 + x] = 0xff2a3643;
                }
            }
        }
        frame.draw(&cam, &ts, &palette, &textures);
        submitted += frame.triangles;
        let x = (n % 3) * 352;
        let y = header_h + (n / 3) * 370;
        for yy in 0..352 {
            pixels[(y + yy) * w + x..(y + yy) * w + x + 352]
                .copy_from_slice(&frame.pixels[yy * 352..(yy + 1) * 352]);
        }
    }
    if submitted == 0 {
        return Err("no rasterized triangles".into());
    }
    let mut r = Raster::from_pixels(w, h, pixels)?;
    for (i, line) in lines.iter().enumerate() {
        text(&mut r, font, line, 14, 22 + i as i32 * 18)?;
    }
    for (n, (name, _)) in views.iter().enumerate() {
        let x = (n % 3) * 352;
        let y = header_h + (n / 3) * 370;
        text(&mut r, font, name, x as i32 + 12, y as i32 + 20)?;
        // Screen-space measurement grid: exactly 128 cache units per major step.
        let step = 128. * 300. / divisor;
        if step >= 8. {
            let mut xx = 176.;
            while xx < 340. {
                r.fill_rect(
                    (x as f32 + xx) as i32,
                    (y + 330) as i32,
                    1,
                    7,
                    0x8191a0,
                    256,
                );
                xx += step;
            }
        }
        r.fill_rect((x + 12) as i32, (y + 337) as i32, 328, 1, 0x8191a0, 256);
        text(
            &mut r,
            font,
            &format!(
                "Grid {:.0}u; {:.2} units/px",
                {
                    let mut u = 32.;
                    while u * 300. / divisor < 16. {
                        u *= 2.;
                    }
                    u
                },
                divisor / 300.
            ),
            x as i32 + 12,
            y as i32 + 349,
        )?;
    }
    Ok((r, submitted, lo, hi))
}
fn main() -> Result<(), E> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() != 6 {
        return Err("CACHE OUTPUT TYPE START_ID END_ID (exclusive)".into());
    }
    let root = Path::new(&a[2]);
    std::fs::create_dir_all(root)?;
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    let crcs = cache.login_crcs()?;
    let font = Font::load(&mut cache, 494)?;
    let mut models = Models::default();
    let start: u32 = a[4].parse()?;
    let end: u32 = a[5].parse()?;
    let kind = a[3].as_str();
    let group = match kind {
        "npcs" => 9,
        "items" => 10,
        "objects" => 6,
        "kits" => 3,
        _ => return Err("unknown type".into()),
    };
    let item_defs = if kind == "items" {
        Some(gielinor_client::item::Definitions::load(&mut cache)?)
    } else {
        None
    };
    let records = cache.group(2, group, [0; 4])?;
    let mut records = records.into_iter().collect::<Vec<_>>();
    records.sort_by_key(|v| v.0);
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("events.jsonl"))?;
    let mut count = 0;
    let mut skipped = 0;
    for (id, data) in records {
        if id < start || id >= end {
            continue;
        }
        if root.join("PAUSE").exists() {
            println!("PAUSED before {kind} {id}");
            break;
        }
        let asset_started = std::time::Instant::now();
        let outcome = (|| -> Result<String, E> {
            let (
                name,
                category,
                variants,
                model_ids,
                recolor,
                retexture,
                scale,
                animations,
                details,
            ) = match kind {
                "npcs" => {
                    let d = npc::Definition::decode_local(id, &data)?;
                    if d.name.trim().is_empty() || d.name == "null" {
                        return Ok("unnamed".into());
                    }
                    let cat = if d.combat < 0 {
                        "noncombat".into()
                    } else {
                        format!(
                            "combat-{:03}-{:03}",
                            d.combat / 50 * 50,
                            d.combat / 50 * 50 + 49
                        )
                    };
                    let mut vs = vec![];
                    if let Some(mut m) = npc_model::mesh(&mut cache, &mut models, &d, false, None)?
                    {
                        let colors = m.light(64 + d.ambient, 850 + d.contrast, [-30, -50, -30])?;
                        npc_model::resize(&mut m, &d);
                        vs.push(("body".to_string(), m, colors));
                    }
                    (
                        d.name,
                        cat,
                        vs,
                        d.models,
                        d.recolor,
                        d.retexture,
                        vec![d.scale[0] as i32, d.scale[1] as i32, d.scale[0] as i32],
                        d.animations.to_vec(),
                        format!(
                            "\"combat_level\":{},\"size_tiles\":{},\"transforms\":{}",
                            d.combat,
                            d.size,
                            format!("{:?}", d.transforms)
                                .replace("Some(", "")
                                .replace("None", "null")
                                .replace(')', "")
                        ),
                    )
                }
                "items" => {
                    let defs = item_defs.as_ref().unwrap();
                    let d = defs.items.get(&id).ok_or("item absent")?;
                    if d.name.trim().is_empty() || d.name == "null" {
                        return Ok("unnamed".into());
                    }
                    let mut vs = vec![];
                    let m = models.inventory(&mut cache, defs, id, 1, None)?;
                    let c = m.light(64 + d.ambient, 768 + d.contrast, [-30, -50, -30])?;
                    vs.push(("inventory / ground (quantity 1)".into(), m, c));
                    for gender in 0..2 {
                        if let Some(m) = models.wearable(&mut cache, d, gender, false, None)? {
                            let c = m.light(64, 850, [-30, -50, -30])?;
                            vs.push((format!("worn body type {gender}"), m, c));
                        }
                    }
                    let mut all_ids = vec![d.model as u32];
                    all_ids.extend(
                        d.wear_models
                            .iter()
                            .flatten()
                            .chain(d.head_models.iter().flatten())
                            .filter_map(|&v| (v >= 0).then_some(v as u32)),
                    );
                    all_ids.sort_unstable();
                    all_ids.dedup();
                    let counts = format!(
                        "[{}]",
                        d.counts
                            .iter()
                            .map(|(a, b)| format!("[{a},{b}]"))
                            .collect::<Vec<_>>()
                            .join(",")
                    );
                    (d.name.clone(),format!("slot-{}",d.wear_positions[0]),vs,all_ids,d.recolor.clone(),d.retexture.clone(),d.scale.to_vec(),vec![],format!("\"inventory_quantity\":1,\"inventory_model\":{},\"inventory_count_variants\":{},\"wear_models\":{:?},\"head_models\":{:?},\"wear_offsets\":{:?},\"wear_positions\":{:?},\"icon_zoom\":{},\"icon_angles\":{:?},\"icon_offsets\":{:?},\"note\":{:?},\"bought\":{:?},\"placeholder\":{:?}",d.model,counts,d.wear_models,d.head_models,d.wear_offset,d.wear_positions,d.zoom,d.angles,d.offset,d.note,d.bought,d.placeholder))
                }
                "objects" => {
                    let d = gielinor_client::object::Definition::decode_local(id, &data)?;
                    if d.name.trim().is_empty() || d.name == "null" {
                        return Ok("unnamed".into());
                    }
                    let shapes = d.model_shapes.clone().unwrap_or(vec![10]);
                    let shapes = shapes.into_iter().collect::<BTreeSet<_>>();
                    let mut om = gielinor_client::object_model::Models::default();
                    let mut vs = vec![];
                    for shape in shapes {
                        if let Some(m) = om.object(&mut cache, &d, shape, 0)? {
                            let c =
                                m.light(64 + d.ambient as i32, 768 + d.contrast, [-30, -50, -30])?;
                            vs.push((format!("shape {shape}, rotation 0"), (*m).clone(), c));
                        }
                    }
                    let anim = d.animation.map(|v| vec![v as i32]).unwrap_or_default();
                    let shape_map = d
                        .model_shapes
                        .as_ref()
                        .map(|v| format!("{v:?}"))
                        .unwrap_or("null".into());
                    (d.name,format!("shape-{}",d.model_shapes.as_ref().and_then(|v|v.first()).copied().unwrap_or(10)),vs,d.models,d.recolor,d.retexture,d.scale.map(i32::from).to_vec(),anim,format!("\"model_shapes\":{},\"size_tiles\":{:?},\"offset\":{:?},\"terrain_contour\":{},\"terrain_applied\":false",shape_map,d.size,d.offset,d.contour))
                }
                "kits" => {
                    let d = gielinor_client::identity_kit::Kit::decode(id, &data, true)?;
                    let mut vs = vec![];
                    if let Some(m) = d.mesh(&mut cache, &mut models, false)? {
                        let c = m.light(64, 850, [-30, -50, -30])?;
                        vs.push(("body kit".into(), m, c));
                    }
                    (
                        format!("Player kit {id} body part {}", d.body_part),
                        format!("body-part-{}", d.body_part),
                        vs,
                        d.models,
                        d.recolor,
                        d.retexture,
                        vec![128; 3],
                        vec![],
                        format!(
                            "\"body_part\":{},\"disabled\":{},\"head_models\":{:?}",
                            d.body_part, d.disabled, d.heads
                        ),
                    )
                }
                _ => unreachable!(),
            };
            if variants.is_empty() {
                return Ok("no-model (transform-only or unavailable)".into());
            }
            let dir = root.join(kind).join(&category);
            std::fs::create_dir_all(&dir)?;
            let png = dir.join(format!("{id}.png"));
            let json = dir.join(format!("{id}.json"));
            if png.exists() && json.exists() {
                return Ok("existing".into());
            }
            let (_, m, _colors) = &variants[0];
            let headers = vec![
                format!("JAGEX REFERENCE | {kind} {id} | {name} | CACHE REV 239"),
                format!(
                    "Models: {model_ids:?} | Vertices: {} | Faces: {}",
                    m.vertices.len(),
                    m.faces.len()
                ),
                format!("Recolors: {recolor:?}"),
                format!("Retextures: {retexture:?} | Scale /128: {scale:?}"),
                format!("Animations [idle,turns,walk,run,crawl]: {animations:?}"),
                "BIND POSE | orthographic | common scale across views | fixed light [-30,-50,-30]"
                    .into(),
                "Internal reference only. Camera axes named explicitly; no animation sampled."
                    .into(),
            ];
            let mut panels = vec![];
            let mut variant_json = vec![];
            let mut raster = 0;
            let mut lo = [0.; 3];
            let mut hi = [0.; 3];
            for (variant, model, lighting) in &variants {
                let mut h = headers.clone();
                h[6]=format!("Variant: {variant} | internal reference | camera axes explicit | no animation sampled");
                h[1] = format!(
                    "Models: {model_ids:?} | Vertices: {} | Faces: {}",
                    model.vertices.len(),
                    model.faces.len()
                );
                h.push(format!("Definition: {details}"));
                let (panel, n, l, u) = sheet(&mut cache, &font, model, lighting, &h)?;
                if panels.is_empty() {
                    lo = l;
                    hi = u;
                }
                raster += n;
                variant_json.push(format!("{{\"label\":{},\"vertices\":{},\"faces\":{},\"bounds_min\":{l:?},\"bounds_max\":{u:?}}}",q(variant),model.vertices.len(),model.faces.len()));
                panels.push(panel);
            }
            let total_h: usize = panels.iter().map(|p| p.height()).sum();
            let mut all = Vec::with_capacity(1056 * total_h);
            for panel in panels {
                all.extend_from_slice(panel.pixels());
            }
            std::fs::write(&png, Raster::from_pixels(1056, total_h, all)?.png()?)?;
            let details = format!("{details},\"variants\":[{}]", variant_json.join(","));
            let metadata=format!("{{\"cache_revision\":239,\"type\":{},\"id\":{id},\"name\":{},\"category\":{},\"model_ids\":{model_ids:?},\"recolors\":{},\"retextures\":{},\"scale\":{scale:?},\"animations\":{animations:?},\"vertices\":{},\"faces\":{},\"bounds_min\":{lo:?},\"bounds_max\":{hi:?},\"raster_triangles\":{raster},\"pose\":\"bind\",\"source_commit\":\"fb37b55363800205e85a75a09fd2dae949700a1f\",{details}}}",q(kind),q(&name),q(&category),pairs(&recolor),pairs(&retexture),m.vertices.len(),m.faces.len());
            std::fs::write(json, metadata)?;
            Ok("rendered".into())
        })();
        let status = match outcome {
            Ok(v) => v,
            Err(e) => format!("error: {e}"),
        };
        if status == "rendered" {
            count += 1
        } else {
            skipped += 1
        }
        writeln!(
            log,
            "{{\"type\":{},\"id\":{id},\"status\":{},\"elapsed_ms\":{}}}",
            q(kind),
            q(&status),
            asset_started.elapsed().as_millis()
        )?;
        log.flush()?;
        std::fs::write(root.join("progress.json"),format!("{{\"type\":{},\"last_id\":{id},\"range\":[{start},{end}],\"rendered_this_chunk\":{count},\"other_this_chunk\":{skipped}}}",q(kind)))?;
    }
    if cache.login_crcs()? != crcs {
        return Err("cache CRCs changed".into());
    }
    println!("DONE {kind} {start}..{end} rendered={count} other={skipped}");
    Ok(())
}
