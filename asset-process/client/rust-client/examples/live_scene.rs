//! research-live: the scene tool's picture with objects placed BY ID from the cache.
//! usage: live_scene CONTENT CACHE PLACEMENTS.txt X Z YAW DISTANCE ELEVATION TARGET_UP OUT.bmp
//! The same steps as examples/authored_scene_snapshot.rs in classic mode (same floor mount, same WorldMesh::build,
//! same camera, same Frame), with one difference: after the authored floor's own placements, every row of
//! PLACEMENTS.txt (`OBJECT_ID X Z SHAPE ROTATION`, tiles of the authored floor) is pushed as a PlacedObject whose id
//! is looked up by the client's ordinary loaders: object::Definition::load_all (archive 2 group 6) and the model
//! loader (archive 7). Nothing of those objects is in the mounted package.
#[cfg(not(feature = "custom-assets"))]
fn main() { eprintln!("requires custom-assets"); std::process::exit(1); }
#[cfg(feature = "custom-assets")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use gielinor_client::{authored_content::Content, authored_floor::ORIGIN, disk_cache::DiskCache, render3d::Frame,
        scene::{PlacedObject, Scene}, texture, world_mesh::WorldMesh};
    use std::collections::{BTreeSet, HashMap};
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    if a.len() != 10 && a.len() != 11 { return Err("CONTENT CACHE PLACEMENTS.txt X Z YAW DISTANCE ELEVATION TARGET_UP OUT.bmp".into()); }
    let content = Content::read(std::path::Path::new(&a[0]))?;
    let cache = DiskCache::open(&a[1], 64 << 20)?;
    let mut cache = content.mount(cache)?;
    // `runtime` as an 11th argument adds the rows AFTER the scene's initial objects are frozen, as a server's object
    // spawn does (scene.rs add_object_collision records the initial list; world_mesh.rs stamps the baked ground
    // shadows from that list only). Without it the rows are part of the map as loaded.
    let rows = std::fs::read_to_string(&a[2])?;
    let runtime = a.get(10).is_some_and(|v| v == "runtime");
    let mut scene = if rows.trim().is_empty() || runtime { Scene::authored_placement_study_at(&mut cache, ORIGIN)? }
        else { Scene::authored_floor_study_at(&mut cache, ORIGIN)? };
    let definitions = gielinor_client::object::Definition::load_all(&mut cache)?;
    let mut by_id = 0;
    for line in rows.lines() {
        let v = line.split_whitespace().map(str::parse::<u32>).collect::<Result<Vec<_>, _>>()?;
        if v.is_empty() { continue; }
        if v.len() != 5 { return Err("placement row".into()); }
        if !definitions.contains_key(&v[0]) { return Err(format!("object {} is not in the cache", v[0]).into()); }
        scene.objects.push(PlacedObject { initial_index: Some(scene.objects.len()), rejected: false, runtime: false,
            id: v[0], plane: 0, x: v[1] as u8, y: v[2] as u8, shape: v[3] as u8, rotation: v[4] as u8 });
        by_id += 1;
    }
    scene.add_object_collision(&definitions)?;
    let mut mesh = WorldMesh::build(&scene, &mut cache, 0)?;
    if mesh.pending_assets { return Err("assets pending".into()); }
    let mut animations = gielinor_client::animation_runtime::Animations::load(&mut cache)?;
    mesh.environment_motion_active(false);
    mesh.advance(&scene, &mut cache, &mut animations, 1)?;
    mesh.sample_environment_motion(0.02)?;
    let p: Vec<f32> = a[3..9].iter().map(|v| v.parse()).collect::<Result<_, _>>()?;
    let local = [(p[0] * 128.) as i32, (p[1] * 128.) as i32];
    let floor_height = scene.height_fine(0, local[0], local[1]).ok_or("target height missing")? as f32;
    let yaw = p[2] * std::f32::consts::TAU / 16384.0;
    let (x, z) = (p[0] * 128., p[1] * 128.);
    let target = [x, floor_height - p[5], z];
    let mut camera = gielinor_client::render3d::Camera::look_at([x + yaw.sin() * p[3], target[1] - p[4], z - yaw.cos() * p[3]], target, 1000.)?;
    camera.far = 30000;
    mesh.update_environment_cutaway(&camera, target);
    let ids = mesh.visible_triangles(0).filter_map(|t| t.texture).collect::<BTreeSet<_>>();
    let defs = texture::Definition::load_all(&mut cache)?;
    let mut textures = HashMap::new();
    for id in ids { textures.insert(id, texture::Texture::load(&mut cache, defs.get(&id).ok_or("texture")?.clone(), 128, 0.8)?); }
    let palette = gielinor_client::floor::palette(0.8)?;
    let mut frame = Frame::new(1280, 900)?;
    mesh.register_authored_art(&mut frame);
    frame.draw_iter(&camera, mesh.visible_triangles(0), &palette, &textures);
    std::fs::write(&a[9], frame.raster()?.bmp()?)?;
    println!("live_scene: {} placements ({by_id} by cache id), {} mesh faces, {} raster triangles", scene.objects.len(), mesh.triangles.len(), frame.triangles);
    Ok(())
}
