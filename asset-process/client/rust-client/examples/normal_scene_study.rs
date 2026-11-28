//! Explicit offline native scene navigation; no game login.
#[cfg(not(windows))]
fn main() {
    eprintln!("native study requires Windows");
    std::process::exit(1);
}
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use gielinor_client::{
        disk_cache::DiskCache,
        rebuild::Normal,
        scene::Scene,
        viewport::{study_view, StudyOptions},
    };
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    if a.len() < 6 {
        return Err(
            "REFERENCE_CACHE ZONE_X ZONE_Z MAX_PLANE MODE ADAPTER [--js5=ADDRESS] [--smoke | --benchmark=NEW_DIRECTORY (native-profiling)]".into(),
        );
    }
    let zone = [a[1].parse::<u16>()?, a[2].parse::<u16>()?];
    let plane = a[3].parse::<usize>()?;
    if zone.iter().any(|v| !(6..=2041).contains(v)) || plane > 3 {
        return Err("study scene bounds".into());
    }
    let mut options = StudyOptions::default();
    let mut remote = None;
    for arg in &a[6..] {
        #[cfg(feature="native-profiling")]
        if let Some(value)=arg.strip_prefix("--benchmark="){if value.is_empty()||options.benchmark.is_some(){return Err("empty or duplicate benchmark".into());}options.benchmark=Some(value.into());continue;}

        if arg == "--smoke" && !options.smoke {
            options.smoke = true;
        } else if let Some(value) = arg.strip_prefix("--js5=") {
            if remote.is_some() {
                return Err("duplicate JS5 option".into());
            }
            remote = Some(value.parse::<std::net::SocketAddr>()?);
        } else {
            return Err("unknown or duplicate study option".into());
        }
    }
    #[cfg(feature="native-profiling")]
    if options.benchmark.as_ref().is_some_and(|p|options.smoke||p.exists()){return Err("benchmark requires a new output directory and excludes smoke".into());}
    #[cfg(feature = "enhanced-renderer")]
    {
        options.renderer = Some(gielinor_client::enhanced::Settings::from_launch_args(
            &a[4],
            &a[5],
            &[],
        )?);
    }
    #[cfg(not(feature = "enhanced-renderer"))]
    {
        if a[4] != "classic" || a[5] != "auto" {
            return Err(
                "default study requires classic auto; Enhanced needs enhanced-renderer feature"
                    .into(),
            );
        }
    }
    let mut cache = if let Some(address) = remote {
        let mut c = DiskCache::open_remote(&a[0], 64 << 20, address)?;
        c.wait_for_recovery(true);
        c
    } else {
        DiskCache::open(&a[0], 64 << 20)?
    };
    let scene = Scene::load_normal(
        &mut cache,
        &Normal {
            world_area: -1,
            zone_x: zone[0],
            zone_y: zone[1],
            players: None,
        },
    )?;
    if !scene.missing_regions.is_empty() {
        return Err("study requires complete referenced regions".into());
    }
    study_view(cache, scene, plane, options)?;
    Ok(())
}
