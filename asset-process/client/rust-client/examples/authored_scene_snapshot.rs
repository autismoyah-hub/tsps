//! Pinned authored inspection at an explicit pose; optional local alignment CSV.
#[cfg(feature="custom-assets")]
#[path="support/local_triangle_dump.rs"]
mod local_triangle_dump;
#[derive(Default)]
struct Options {remote:Option<std::net::SocketAddr>,triangles:bool}
impl Options {
    fn parse(args:&[String])->Result<Self,Box<dyn std::error::Error>> {
        let mut o=Self::default();
        for arg in args {
            if arg=="--triangles"&&!o.triangles{o.triangles=true;}
            else if let Some(value)=arg.strip_prefix("--js5="){if o.remote.is_some(){return Err("duplicate JS5".into());}o.remote=Some(value.parse()?);}
            else{return Err("unknown or duplicate snapshot option".into());}
        }Ok(o)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pose { tile: [f32; 2], yaw: i32, distance: f32, elevation: f32, target_up: f32 }
impl Pose {
    fn parse(args: &[String]) -> Result<Self, Box<dyn std::error::Error>> {
        if args.len()!=6 {return Err("expected X Z YAW DISTANCE ELEVATION TARGET_UP".into());}
        let p=Self{tile:[args[0].parse()?,args[1].parse()?],yaw:args[2].parse()?,distance:args[3].parse()?,elevation:args[4].parse()?,target_up:args[5].parse()?};
        if p.tile.iter().any(|v|!v.is_finite()||!(1.0..=102.0).contains(v)) || !(0..16384).contains(&p.yaw)
            || !p.distance.is_finite() || !(250.0..=12000.0).contains(&p.distance)
            || !p.elevation.is_finite() || !(1.0..=12000.0).contains(&p.elevation)
            || !p.target_up.is_finite() || !(0.0..=650.0).contains(&p.target_up) {return Err("snapshot pose out of bounds".into());}
        Ok(p)
    }
    fn camera(self, floor_height: f32) -> gielinor_client::Result<(gielinor_client::render3d::Camera,[f32;3])> {
        let [x,z]=self.tile.map(|v|v*128.0);let yaw=self.yaw as f32*std::f32::consts::TAU/16384.0;
        let target=[x,floor_height-self.target_up,z];
        let mut camera=gielinor_client::render3d::Camera::look_at([x+yaw.sin()*self.distance,target[1]-self.elevation,z-yaw.cos()*self.distance],target,1000.)?;
        camera.far=30000;Ok((camera,target))
    }
}
#[cfg(not(feature="custom-assets"))]
fn main(){eprintln!("requires custom-assets");std::process::exit(1);}
#[cfg(feature="custom-assets")]
fn main()->Result<(),Box<dyn std::error::Error>> {
    use gielinor_client::{authored_content::Content,authored_floor::ORIGIN,disk_cache::DiskCache,scene::Scene,world_mesh::WorldMesh,render3d::Frame,texture};
    use std::{collections::{BTreeSet,HashMap},path::Path};
    use sha2::{Digest,Sha256};
    let a=std::env::args().skip(1).collect::<Vec<_>>();
    if a.len()<11 || a.len()>13{return Err("CONTENT REFERENCE_CACHE X Z YAW DISTANCE ELEVATION TARGET_UP MODE ADAPTER NEW_OUTPUT_DIRECTORY [--js5=ADDRESS] [--triangles]".into());}
    let pose=Pose::parse(&a[2..8])?;
    let options=Options::parse(&a[11..])?;let remote=options.remote;
    #[cfg(feature="enhanced-renderer")]
    let settings=gielinor_client::enhanced::Settings::from_launch_args(&a[8],&a[9],&[])?;
    #[cfg(not(feature="enhanced-renderer"))]
    if a[8]!="classic" || a[9]!="auto" {return Err("default snapshot requires classic auto".into());}
    let output=Path::new(&a[10]);if output.exists(){return Err("output directory must be new".into());}
    let content_path=Path::new(&a[0]);
    let info=std::fs::symlink_metadata(content_path)?;
    if !info.is_file() || info.file_type().is_symlink() || info.len()>65536 {return Err("content declaration bounds/type".into());}
    let mut bytes=Vec::new();
    use std::io::Read;
    std::fs::File::open(content_path)?.take(65537).read_to_end(&mut bytes)?;
    let content=Content::decode(&bytes,content_path.parent().unwrap_or(Path::new(".")))?;
    let declaration_hash=format!("{:x}",Sha256::digest(&bytes));
    let mut cache=if let Some(address)=remote {let mut c=DiskCache::open_remote(&a[1],64<<20,address)?;c.wait_for_recovery(true);c}else{DiskCache::open(&a[1],64<<20)?};
    let original_crcs=cache.login_crcs()?;let mut cache=content.mount(cache)?;
    let scene=Scene::authored_placement_study_at(&mut cache,ORIGIN)?;
    let mut mesh=WorldMesh::build(&scene,&mut cache,0)?;if mesh.pending_assets{return Err("snapshot assets pending".into());}
    let mut animations=gielinor_client::animation_runtime::Animations::load(&mut cache)?;
    // Offline snapshots must not sample elapsed wall time from native scheduling.
    mesh.environment_motion_active(false);
    mesh.advance(&scene,&mut cache,&mut animations,1)?;
    mesh.sample_environment_motion(0.02)?;
    let local=pose.tile.map(|v|(v*128.) as i32);
    let floor_height=scene.height_fine(0,local[0],local[1]).ok_or("target height missing")? as f32;
    let inspection_center=[pose.tile[0]*128.,floor_height,pose.tile[1]*128.];
    let inspection=options.triangles.then(||local_triangle_dump::export(&mesh.triangles,inspection_center)).transpose()?;
    let (camera,target)=pose.camera(floor_height)?;
    mesh.update_environment_cutaway(&camera,target);
    let ids=mesh.visible_triangles(0).filter_map(|t|t.texture).collect::<BTreeSet<_>>();
    let definitions=texture::Definition::load_all(&mut cache)?;let mut textures=HashMap::new();
    for id in ids {textures.insert(id,texture::Texture::load(&mut cache,definitions.get(&id).ok_or("texture definition missing")?.clone(),128,0.8)?);}
    let palette=gielinor_client::floor::palette(0.8)?;let mut frame=Frame::new(1280,900)?;
    #[cfg(feature="enhanced-renderer")]
    if settings.mode!=gielinor_client::enhanced::Mode::Classic {frame.begin_presentation_capture();}
    mesh.register_authored_art(&mut frame);
    frame.draw_iter(&camera,mesh.visible_triangles(0),&palette,&textures);
    #[cfg(feature="enhanced-renderer")]
    let mut backend=String::new();
    #[cfg(not(feature="enhanced-renderer"))]
    let backend=String::new();
    #[cfg(feature="enhanced-renderer")]
    if settings.mode!=gielinor_client::enhanced::Mode::Classic {
        let geometry=frame.take_presentation_geometry()?.ok_or("capture absent")?;let triangles=geometry.iter().collect::<Vec<_>>();
        let mut renderer=gielinor_client::enhanced::Renderer::new(settings)?;
        let casters=if renderer.settings().shadows>0 {gielinor_client::enhanced::root_shadow_casters(mesh.visible_triangles(0))} else {Vec::new()};
        for _ in 0..3 {frame.pixels=renderer.render_with_shadow_casters(gielinor_client::enhanced::SceneFrame{camera:&camera,triangles:&triangles,stable_count:0,stable_revision:None,albedo:None,palette:&palette,textures:&textures,size:[1280,900],light_focus:target,ambient:64,contrast:768},None,&casters)?;}
        backend=renderer.stats.json();
    }
    if frame.triangles==0 || !frame.pixels.windows(2).any(|p|p[0]!=p[1]) {return Err("empty snapshot".into());}
    if cache.login_crcs()?!=original_crcs{return Err("original reference CRCs changed".into());}
    let image=frame.raster()?.bmp()?;
    let mut scope=format!("OFFLINE PINNED AUTHORED SNAPSHOT; NOT LIVE GAMEPLAY\ncontent_sha256={declaration_hash}\npose={pose:?}\nmode={}\nplacements={}\nmesh_faces={}\nraster_triangles={}\nadvance_ticks=1\nenvironment_motion_seconds=0.02\noriginal_reference_crcs={original_crcs:?}\n",a[8],scene.objects.len(),mesh.triangles.len(),frame.triangles);
    if let Some((csv,rows))=&inspection {scope+=&format!("local_triangles_rows={rows}\nlocal_triangles_sha256={:x}\nlocal_triangles_center={inspection_center:?}\nlocal_triangles_scope=prepared mesh before cutaway and visibility; all vertices within strict X/Z550 and height12\n",Sha256::digest(csv.as_bytes()));}
    std::fs::create_dir(output)?;std::fs::write(output.join("view.bmp"),image)?;std::fs::write(output.join("scope.txt"),scope)?;
    if !backend.is_empty(){std::fs::write(output.join("renderer.json"),backend)?;}
    if let Some((csv,_))=inspection {std::fs::write(output.join("local-triangles.csv"),csv)?;}
    println!("Pinned authored snapshot: {} placements, {} raster triangles; no game login",scene.objects.len(),frame.triangles);Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn values()->Vec<String>{["53","46","0","2200","1800","0"].map(String::from).to_vec()}
    #[test]
    fn source_pose_keeps_target_raise_and_camera_offset() {
        let p=Pose::parse(&values()).unwrap();let (c,t)=p.camera(-240.).unwrap();assert_eq!(t,[6784.,-240.,5888.]);assert_eq!(c.position,[6784.,-2040.,3688.]);assert_eq!(c.far,30000);
        let mut raised=p;raised.target_up=650.;let (c2,t2)=raised.camera(-240.).unwrap();assert_eq!(t2[1],t[1]-650.);assert_eq!(c2.position[1],c.position[1]-650.);
    }
    #[test]
    fn invalid_camera_inputs_are_rejected() {
        for (i,value) in [(0,"NaN"),(1,"103"),(2,"16384"),(3,"inf"),(3,"249"),(4,"0"),(5,"651"),(5,"-1")] {let mut a=values();a[i]=value.into();assert!(Pose::parse(&a).is_err());}
        assert!(Pose::parse(&values()[..5]).is_err());
    }
    #[test]
    fn inspection_is_explicit_and_options_reject_duplicates() {
        assert!(!Options::parse(&[]).unwrap().triangles);
        let o=Options::parse(&["--triangles".into(),"--js5=127.0.0.1:1234".into()]).unwrap();assert!(o.triangles);assert!(o.remote.is_some());
        for args in [vec!["--triangles","--triangles"],vec!["--js5=127.0.0.1:1","--js5=127.0.0.1:2"],vec!["--triangles=path"],vec!["--js5=invalid"]] {assert!(Options::parse(&args.into_iter().map(String::from).collect::<Vec<_>>()).is_err());}
    }
}
