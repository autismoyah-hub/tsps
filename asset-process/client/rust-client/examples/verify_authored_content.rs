//! Offline verification of an explicit content declaration, not live gameplay.
#[cfg(not(feature = "custom-assets"))]
fn main() {
    eprintln!("build with --features custom-assets");
    std::process::exit(1);
}
#[cfg(feature = "custom-assets")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use gielinor_client::{
        authored_content::Content, authored_floor::ORIGIN, disk_cache::DiskCache,
        rebuild::Instance, scene::Scene, world_mesh::WorldMesh,
    };
    let a: Vec<_> = std::env::args().skip(1).collect();
    if a.len() != 3 {
        return Err("CONTENT CACHE JS5".into());
    }
    let mut cache = DiskCache::open_remote(&a[1], 64 << 20, a[2].parse()?)?;
    cache.wait_for_recovery(true);
    let before = cache.login_crcs()?;
    let mut cache = Content::read(std::path::Path::new(&a[0]))?.mount(cache)?;
    let scene = Scene::load_instance(
        &mut cache,
        &Instance {
            zone_x: (ORIGIN[0] / 8 + 6) as u16,
            zone_y: (ORIGIN[1] / 8 + 6) as u16,
            reload: false,
            chunks: vec![None; 676],
            regions: vec![],
        },
    )?;
    let mut mesh = WorldMesh::build(&scene, &mut cache, 3)?;
    assert!(!mesh.pending_assets);
    mesh.sample_environment_motion(1.)?;
    let geography = gielinor_client::environment_minimap::scene_geography(&scene, &mut cache)?;
    assert!(geography
        .as_ref()
        .is_some_and(|p| p.iter().any(|&p| p != 0)));
    assert_eq!(before, cache.login_crcs()?);
    println!("Explicit content declaration admitted; {} regional visual objects, {} triangles, original CRCs unchanged. Offline replay; functional scenery and machinery remain server-owned.",scene.objects.len(),mesh.triangles.len());
    Ok(())
}
