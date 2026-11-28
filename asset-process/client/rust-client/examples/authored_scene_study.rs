//! Pinned Classic inspection with validated cache references; never live gameplay evidence.
#[cfg(not(feature="custom-assets"))]
fn main() {eprintln!("build with --features custom-assets");std::process::exit(1);}

#[cfg(feature="custom-assets")]
fn main()->Result<(),Box<dyn std::error::Error>> {
    use gielinor_client::{asset_package::Package,authored_floor::{Floor,ORIGIN},disk_cache::DiskCache,
        environment_style::Bindings,environment_cutaway::Metadata,scene::Scene,world_mesh::WorldMesh,
        render3d::{Camera,Frame},raster::Raster};
    use std::{path::Path,collections::HashMap,fmt::Write};
    fn pin(text:&str)->Result<[u8;32],Box<dyn std::error::Error>> {
        if text.len()!=64 || !text.bytes().all(|b|b.is_ascii_hexdigit()) {return Err("expected SHA256 identity".into());}
        let mut bytes=[0;32];for (i,b) in bytes.iter_mut().enumerate() {*b=u8::from_str_radix(&text[i*2..i*2+2],16)?;}Ok(bytes)
    }
    let mut args=std::env::args().skip(1).collect::<Vec<_>>();
    let recovery=if args.last().is_some_and(|a|a.starts_with("--js5=")) {
        Some(args.pop().unwrap()[6..].parse::<std::net::SocketAddr>()?)
    }else{None};
    if args.len()!=10 {return Err("REFERENCE_CACHE PACKAGE PACKAGE_SHA256 FLOOR FLOOR_SHA256 BINDINGS BINDINGS_SHA256 CUTAWAY CUTAWAY_SHA256 NEW_OUTPUT_DIRECTORY [--js5=ADDRESS]".into());}
    let reference_cache=args.remove(0);
    let pins=[pin(&args[1])?,pin(&args[3])?,pin(&args[5])?,pin(&args[7])?];
    let output=Path::new(&args[8]);if output.exists() {return Err("output directory must be new".into());}
    let package=Package::read(Path::new(&args[0]))?;
    if package.identity()!=pins[0] {return Err("package identity mismatch".into());}
    let bindings=Bindings::read(Path::new(&args[4]),pins[2],&package)?;
    let cutaway=Metadata::read(Path::new(&args[6]),pins[3],&package)?;
    let floor=Floor::read(Path::new(&args[2]),pins[1])?.with_bindings(bindings)?.with_cutaway(cutaway)?;
    let mut cache=if let Some(address)=recovery {
        let mut cache=DiskCache::open_remote(reference_cache,64<<20,address)?;
        cache.wait_for_recovery(true);cache
    }else{DiskCache::open(reference_cache,64<<20)?};
    let original_crcs=cache.login_crcs()?;
    cache.mount_package(package,pins[0])?;
    cache.mount_floor(floor,pins[1])?;
    let scene=Scene::authored_placement_study_at(&mut cache,ORIGIN)?;
    let mesh=WorldMesh::build(&scene,&mut cache,0)?;
    if mesh.pending_assets || mesh.triangles.iter().any(|t|t.texture.is_some()) {
        return Err("offline study requires fully admitted untextured package geometry".into());
    }
    let palette=gielinor_client::floor::palette(0.8)?;
    let textures=HashMap::new();let target=[52.*128.,scene.elevation(0,52*128,52*128).unwrap_or(-240) as f32,52.*128.];
    let mut images=Vec::new();
    let mut report=format!("OFFLINE DECLARED PLACEMENTS; CLASSIC SOFTWARE STUDY; NOT LIVE GAMEPLAY\nplacements={}\nmesh_faces={}\n",scene.objects.len(),mesh.triangles.len());
    writeln!(report,"explicit_js5_recovery={}",recovery.is_some())?;
    for (name,pin) in ["package","floor","bindings","cutaway"].into_iter().zip([&args[1],&args[3],&args[5],&args[7]]) {
        writeln!(report,"{name}_sha256={}",pin.to_ascii_lowercase())?;
    }
    for (index,direction) in [[0.,-1.],[1.,0.],[0.,1.],[-1.,0.]].into_iter().enumerate() {
        let mut camera=Camera::look_at([target[0]+direction[0]*11000.,target[1]-10000.,target[2]+direction[1]*11000.],target,650.)?;
        camera.far=30000;mesh.update_environment_cutaway(&camera,target);
        let mut frame=Frame::new(800,600)?;
        mesh.register_authored_art(&mut frame);
        frame.draw_iter(&camera,mesh.visible_triangles(0),&palette,&textures);
        if !frame.pixels.windows(2).any(|p|p[0]!=p[1]) {return Err("empty offline study frame".into());}
        writeln!(report,"view_{index}_raster_triangles={}",frame.triangles)?;
        images.push((format!("classic-view-{index}.bmp"),frame.raster()?.bmp()?));
    }
    let geography=gielinor_client::environment_minimap::scene_geography(&scene,&mut cache)?.ok_or("authored geography absent")?;
    if !geography.iter().any(|&p|p!=0) {return Err("authored geography empty".into());}
    images.push(("minimap-geography.bmp".into(),Raster::from_pixels(scene.width*4,scene.height*4,geography)?.bmp()?));
    if cache.login_crcs()?!=original_crcs {return Err("original cache reference CRCs changed".into());}
    writeln!(report,"original_reference_crcs={original_crcs:?}")?;
    // Admission and rendering finish before any output directory is created.
    std::fs::create_dir(output)?;
    for (name,bytes) in images {std::fs::write(output.join(name),bytes)?;}
    std::fs::write(output.join("scope.txt"),report)?;
    println!("Pinned offline Classic study: four 800x600 views and 416x416 authored geography; no game login or window");
    Ok(())
}
