//! Static reference-cache inspection with the maintained Classic camera and textures.
use gielinor_client::{
    disk_cache::DiskCache,
    rebuild::Normal,
    render3d::{Camera, Frame},
    scene::Scene,
    texture,
    world_mesh::WorldMesh,
};
use std::{
    collections::{BTreeSet, HashMap},
    path::Path,
};
#[derive(Debug, PartialEq)]
struct Options {
    remote: Option<std::net::SocketAddr>,
    mode: String,
    adapter: String,
}
fn options(args: &[String]) -> Result<Options, Box<dyn std::error::Error>> {
    let mut remote = None;
    let mut mode = None;
    let mut adapter = None;
    for arg in args {
        if let Some(value) = arg.strip_prefix("--js5=") {
            if remote.is_some() {
                return Err("duplicate JS5 option".into());
            }
            remote = Some(value.parse()?);
        } else if let Some(value) = arg.strip_prefix("--renderer=") {
            if mode.replace(value.to_owned()).is_some() {
                return Err("duplicate renderer option".into());
            }
        } else if let Some(value) = arg.strip_prefix("--adapter=") {
            if adapter.replace(value.to_owned()).is_some() {
                return Err("duplicate adapter option".into());
            }
        } else {
            return Err("unknown snapshot option".into());
        }
    }
    let mode = mode.unwrap_or_else(|| "classic".into());
    let adapter = adapter.unwrap_or_else(|| "auto".into());
    if !matches!(mode.as_str(), "classic" | "lite" | "gpu") {
        return Err("invalid renderer mode".into());
    }
    if adapter != "auto" {
        adapter.parse::<u32>()?;
    }
    #[cfg(not(feature = "enhanced-renderer"))]
    if mode != "classic" {
        return Err("Enhanced snapshot requires enhanced-renderer feature".into());
    }
    Ok(Options {
        remote,
        mode,
        adapter,
    })
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 7 {
        return Err("REFERENCE_CACHE WORLD_X WORLD_Z PLANE PITCH YAW NEW_OUTPUT_DIRECTORY [--js5=ADDRESS] [--renderer=classic|lite|gpu] [--adapter=auto|INDEX]".into());
    }
    let options = options(&args[7..])?;
    let remote = options.remote;
    let world = [args[1].parse::<u16>()?, args[2].parse::<u16>()?];
    let plane = args[3].parse::<usize>()?;
    let pitch = args[4].parse::<i32>()?;
    let yaw = args[5].parse::<i32>()?;
    if world.iter().any(|v| !(48..=16335).contains(v))
        || plane > 3
        || !(1024..=3064).contains(&pitch)
        || !(0..16384).contains(&yaw)
    {
        return Err("world/plane/camera out of bounds".into());
    }
    let output = Path::new(&args[6]);
    if output.exists() {
        return Err("output directory must be new".into());
    }
    let mut cache = if let Some(address) = remote {
        let mut c = DiskCache::open_remote(&args[0], 64 << 20, address)?;
        c.wait_for_recovery(true);
        c
    } else {
        DiskCache::open(&args[0], 64 << 20)?
    };
    let crcs = cache.login_crcs()?;
    let rebuild = Normal {
        world_area: -1,
        zone_x: world[0] / 8,
        zone_y: world[1] / 8,
        players: None,
    };
    let scene = Scene::load_normal(&mut cache, &rebuild)?;
    if !scene.missing_regions.is_empty() {
        return Err("snapshot requires all referenced regions".into());
    }
    let mesh = WorldMesh::build(&scene, &mut cache, plane)?;
    if mesh.pending_assets {
        return Err("snapshot assets pending".into());
    }
    let ids = mesh
        .visible_triangles(plane as u8)
        .filter_map(|t| t.texture)
        .collect::<BTreeSet<_>>();
    let definitions = texture::Definition::load_all(&mut cache)?;
    let mut textures = HashMap::new();
    for &id in &ids {
        let definition = definitions
            .get(&id)
            .ok_or("texture definition missing")?
            .clone();
        textures.insert(
            id,
            texture::Texture::load(&mut cache, definition, 128, 0.8)?,
        );
    }
    let local = [
        (i32::from(world[0]) - scene.base_x) * 128 + 64,
        (i32::from(world[1]) - scene.base_y) * 128 + 64,
    ];
    let focus = [
        local[0],
        scene
            .height_fine(plane, local[0], local[1])
            .ok_or("focus height missing")?
            - 50,
        local[1],
    ];
    let settings = gielinor_client::viewport_shape::Settings::default();
    let shape = settings.shape([765, 503]);
    let orbit = 600 + 3 * (pitch / 8);
    let eye = gielinor_client::camera::position(
        focus,
        pitch,
        yaw,
        orbit,
        shape.size[1],
        settings.zoom.map(i32::from),
    );
    let mut camera = Camera::root_scene(
        eye,
        pitch,
        yaw,
        shape.focal as f32,
        [scene.width, scene.height],
    )?;
    camera.far = gielinor_client::device_options::DeviceOptions::default().far_clip();
    let palette = gielinor_client::floor::palette(0.8)?;
    let (pixels, raster_triangles, backend) = if options.mode == "classic" {
        let mut frame = Frame::new(shape.size[0] as usize, shape.size[1] as usize)?;
        frame.draw_iter(
            &camera,
            mesh.visible_triangles(plane as u8),
            &palette,
            &textures,
        );
        (frame.pixels, frame.triangles, String::new())
    } else {
        #[cfg(feature = "enhanced-renderer")]
        {
            let settings = gielinor_client::enhanced::Settings::from_launch_args(
                &options.mode,
                &options.adapter,
                &[],
            )?;
            let mut renderer = gielinor_client::enhanced::Renderer::new(settings.clone())?;
            let mut observed = Frame::new(shape.size[0] as usize, shape.size[1] as usize)?;
            observed.begin_presentation_capture();
            observed.draw_iter(
                &camera,
                mesh.visible_triangles(plane as u8),
                &palette,
                &textures,
            );
            let geometry = observed
                .take_presentation_geometry()?
                .ok_or("presentation capture absent")?;
            let triangles = geometry.iter().collect::<Vec<_>>();
            let mut pixels = Vec::new();
            for _ in 0..3 {
                pixels = renderer.render(gielinor_client::enhanced::SceneFrame {
                    camera: &camera,
                    triangles: &triangles,
                    stable_count: 0,
                    stable_revision: None,
                    albedo: None,
                    palette: &palette,
                    textures: &textures,
                    size: shape.size.map(|n| n as u32),
                    light_focus: focus.map(|n| n as f32),
                    ambient: 64,
                    contrast: 768,
                })?;
            }
            (
                pixels,
                observed.triangles,
                format!(
                    "mode={}\nsettings={settings:?}\nframes=3\nsubmission=maintained Classic accepted color geometry\nbackend={}\n",
                    options.mode,
                    renderer.stats.json()
                ),
            )
        }
        #[cfg(not(feature = "enhanced-renderer"))]
        {
            return Err("Enhanced snapshot unavailable".into());
        }
    };
    if !pixels.windows(2).any(|p| p[0] != p[1]) {
        return Err("snapshot frame empty".into());
    }
    if crcs != cache.login_crcs()? {
        return Err("reference CRCs changed".into());
    }
    let report=format!("STATIC REFERENCE-CACHE CLASSIC SNAPSHOT; NOT LIVE GAMEPLAY\nworld={world:?}\nbase={},{}\nplane={plane}\npitch={pitch}\nyaw={yaw}\nfocus={focus:?}\norbit={orbit}\nunclamped_eye={eye:?}\nprojection_eye={:?}\nfocal={}\ncanvas={:?}\nviewport_offset={:?}\nfar={}\nmesh_triangles={}\nraster_triangles={}\ntextures={ids:?}\nreference_crcs={crcs:?}\nexplicit_js5_recovery={}\nactors=false\nui=false\nanimation_advance=false\nvarps=default_zero\nvisibility=static_plane_filter; not live scene traversal or automatic roofs\n",scene.base_x,scene.base_y,camera.position,shape.focal,shape.size,shape.offset,camera.far,mesh.triangles.len(),raster_triangles,remote.is_some());
    let bmp = gielinor_client::raster::Raster::from_pixels(
        shape.size[0] as usize,
        shape.size[1] as usize,
        pixels,
    )?
    .bmp()?;
    std::fs::create_dir(output)?;
    std::fs::write(output.join(format!("{}.bmp", options.mode)), bmp)?;
    let report = if options.mode == "classic" {
        report
    } else {
        report.replace("CLASSIC SNAPSHOT", "ENHANCED SNAPSHOT") + &backend
    };
    std::fs::write(output.join("scope.txt"), report)?;
    println!(
        "Static snapshot: {} mesh triangles, {} textures, {} raster triangles; no window or login",
        mesh.triangles.len(),
        textures.len(),
        raster_triangles
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(values: &[&str]) -> Result<Options, Box<dyn std::error::Error>> {
        options(&values.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }
    #[test]
    fn explicit_options_reject_duplicates_and_invalid_modes() {
        assert_eq!(
            parse(&[]).unwrap(),
            Options {
                remote: None,
                mode: "classic".into(),
                adapter: "auto".into()
            }
        );
        for args in [
            vec!["--renderer=unknown"],
            vec!["--adapter=-1"],
            vec!["--js5=bad"],
            vec!["--unknown=1"],
            vec!["--renderer=classic", "--renderer=classic"],
            vec!["--adapter=1", "--adapter=1"],
            vec!["--js5=127.0.0.1:1", "--js5=127.0.0.1:2"],
        ] {
            assert!(parse(&args).is_err(), "{args:?}");
        }
        let p = parse(&["--adapter=1", "--js5=127.0.0.1:1", "--renderer=classic"]).unwrap();
        assert_eq!(p.adapter, "1");
        assert!(p.remote.is_some());
    }
    #[test]
    fn optional_renderer_gate_is_explicit() {
        for mode in ["lite", "gpu"] {
            let arg = format!("--renderer={mode}");
            #[cfg(feature = "enhanced-renderer")]
            assert_eq!(parse(&[&arg]).unwrap().mode, mode);
            #[cfg(not(feature = "enhanced-renderer"))]
            assert!(parse(&[&arg]).is_err());
        }
    }
}
