//! Static decoder/backend measurements, never full-client frame-rate evidence.
#[cfg(not(all(feature = "custom-assets", feature = "enhanced-renderer")))]
fn main() {
    eprintln!("build with --features custom-assets,enhanced-renderer");
    std::process::exit(1);
}
#[cfg(all(feature = "custom-assets", feature = "enhanced-renderer"))]
#[path = "support/benchmark_scene.rs"]
mod benchmark_scene;
#[cfg(all(feature = "custom-assets", feature = "enhanced-renderer"))]
mod benchmark {
    use gielinor_client::{asset_package::Package, enhanced::{Mode, Renderer, SceneFrame, Settings},
        model::Model, render3d::{Camera, Frame, Triangle}, raster::Raster};
    use std::{collections::HashMap, fmt::Write, path::Path, time::Instant};
    const WARMUP: usize = 10;

    fn validate_frames(frames: usize) -> Result<(), &'static str> {
        if (1..=600).contains(&frames) { Ok(()) } else { Err("frames must be in 1..600") }
    }
    fn summarize(samples: &[f64]) -> Result<[f64; 4], &'static str> {
        if samples.is_empty() || samples.iter().any(|x| !x.is_finite() || *x < 0.) {
            return Err("invalid timing samples");
        }
        let mut sorted = samples.to_vec(); sorted.sort_by(f64::total_cmp);
        let percentile = |q: f64| sorted[((sorted.len()-1) as f64*q).round() as usize];
        Ok([samples.iter().sum::<f64>() / samples.len() as f64,
            percentile(0.5), percentile(0.95), percentile(0.99)])
    }
    fn benchmark_options(options:&[String])->Result<(usize,Option<String>,Option<String>,Vec<String>),String> {
        let mut copies=None;let mut camera=None;let mut scene=None;let mut renderer=Vec::new();
        for option in options {
            if let Some(value)=option.strip_prefix("--copies=") {
                if copies.is_some(){return Err("duplicate --copies".into());}
                let value=value.parse::<usize>().map_err(|_|"invalid --copies")?;
                if !(1..=25).contains(&value){return Err("copies must be in 1..25".into());}
                copies=Some(value);
            } else if let Some(value)=option.strip_prefix("--camera=") {
                if camera.is_some() || value.is_empty(){return Err("empty or duplicate --camera".into());}
                camera=Some(value.to_string());
            } else if let Some(value)=option.strip_prefix("--scene=") {
                if scene.is_some() || value.is_empty(){return Err("empty or duplicate --scene".into());}
                scene=Some(value.to_string());
            } else {renderer.push(option.clone());}
        }
        Ok((copies.unwrap_or(1),camera,scene,renderer))
    }
    struct CameraRecipe { camera:Camera, size:[u32;2], focus:[f32;3], scope:String }
    impl CameraRecipe {
        fn read(path:&Path)->Result<Self,Box<dyn std::error::Error>> {
            use std::io::Read;
            let file=std::fs::File::open(path)?;
            if !file.metadata()?.is_file(){return Err("camera must be a regular file".into());}
            let mut bytes=Vec::new();file.take(4097).read_to_end(&mut bytes)?;
            if bytes.len()>4096 {return Err("camera recipe exceeds 4096 bytes".into());}
            Self::parse(std::str::from_utf8(&bytes)?)
        }
        fn parse(text:&str)->Result<Self,Box<dyn std::error::Error>> {
            let c:Vec<_>=text.split_whitespace().collect();
            let (mut camera,size,focus)=match c.first().copied() {
                Some("look") if c.len()==10 => {
                    let f=c[1..8].iter().map(|s|s.parse::<f32>()).collect::<Result<Vec<_>,_>>()?;
                    if f[..6].iter().any(|v|!v.is_finite() || v.abs()>1_000_000.) || !f[6].is_finite() || !(1. ..=8192.).contains(&f[6]) {return Err("bounded look camera coordinates/focal".into());}
                    let size=[c[8].parse::<u32>()?,c[9].parse::<u32>()?];
                    if size.contains(&0) || size[0]>3840 || size[1]>2160 {return Err("camera dimensions must be within 3840x2160".into());}
                    let focus=[f[3],f[4],f[5]];
                    (Camera::look_at([f[0],f[1],f[2]],focus,f[6])?,size,focus)
                },
                Some("player") if c.len()==7 => {
                    let f=c[1..].iter().map(|s|s.parse::<i32>()).collect::<Result<Vec<_>,_>>()?;
                    if f[..3].iter().any(|v|!(-1_000_000..=1_000_000).contains(v)) || !(0..16384).contains(&f[3]) || !(0..16384).contains(&f[4]) || !(50..=12000).contains(&f[5]) {return Err("bounded player camera coordinates/angles/distance".into());}
                    let focus=[f[0],f[1],f[2]];
                    let eye=gielinor_client::camera::position(focus,f[3],f[4],f[5],503,[256,320]);
                    (Camera::angles(eye,f[3],f[4],308.)?,[765,503],focus.map(|v|v as f32))
                },
                _=>return Err("expected look(9 values) or player(6 values) camera".into())
            };
            camera.far=5250;
            Ok(Self{camera,size,focus,scope:format!("camera_recipe={}\nlight_focus={focus:?}\n",c.join(" "))})
        }
    }
    fn expanded_len(faces:usize,copies:usize)->Result<usize,&'static str> {
        if !(1..=25).contains(&copies){return Err("copies must be in 1..25");}
        faces.checked_mul(copies).filter(|&n|n<=1_000_000).ok_or("expanded triangle budget")
    }
    fn copy_offset(index:usize)->[f32;3] {[(index%5)as f32*850.,0.,(index/5)as f32*850.]}
    fn append_copies(triangles:&mut Vec<Triangle>,copies:usize)->Result<(),&'static str> {
        let base=triangles.len();let total=expanded_len(base,copies)?;
        triangles.try_reserve_exact(total-base).map_err(|_|"expanded triangle allocation")?;
        let owner_stride=triangles.iter().map(|t|t.tag).max().unwrap_or(0);
        for index in 1..copies {
            let offset=copy_offset(index);
            for face in 0..base {
                let mut t=triangles[face];
                t.vertices=t.vertices.map(|v|std::array::from_fn(|i|v[i]+offset[i]));
                t.local_vertices=Some(t.vertices);t.tag+=index as u64*owner_stride;
                if let Some(order)=&mut t.model_order {order.batch+=index*base;}
                triangles.push(t);
            }
        }
        Ok(())
    }
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let a: Vec<_> = std::env::args().skip(1).collect();
        if a.len() < 7 { return Err("PACKAGE PACKAGE_SHA256 MODEL_ID_OR_scene MODE ADAPTER FRAMES NEW_OUTPUT [--copies=1..25] [--camera=FILE] [--scene=FILE] [renderer options]".into()); }
        let (copies,camera_path,scene_path,renderer_options)=benchmark_options(&a[7..])?;
        if scene_path.is_some()!=(a[2]=="scene"){return Err("use positional scene together with --scene=FILE".into());}
        let settings = Settings::from_launch_args(&a[3], &a[4], &renderer_options)?;
        let frames: usize = a[5].parse()?; validate_frames(frames)?;
        let output = Path::new(&a[6]);
        if output.exists() { return Err("output directory must be new".into()); }
        let recipe=camera_path.as_deref().map(|p|CameraRecipe::read(Path::new(p))).transpose()?;
        let scene=scene_path.as_deref().map(|p|super::benchmark_scene::Recipe::read(Path::new(p))).transpose()?;
        if a[1].len()!=64 || !a[1].bytes().all(|b| b.is_ascii_hexdigit()) { return Err("expected package SHA256".into()); }
        let mut pin=[0;32];
        for (i,b) in pin.iter_mut().enumerate() { *b=u8::from_str_radix(&a[1][2*i..2*i+2],16)?; }
        let package=Package::read(Path::new(&a[0]))?;
        if package.identity()!=pin { return Err("package identity mismatch".into()); }
        use sha2::{Digest,Sha256};
        let (model,owners,ambient,contrast,input_scope)=if let Some(scene)=scene {
            let loaded=scene.load(&package)?;
            (loaded.model,loaded.owners,loaded.ambient,loaded.contrast,loaded.scope)
        } else {
            let id:u32=a[2].parse()?;
            let bytes=package.model(id).ok_or("model absent from pinned package")?;
            let model=Model::decode(bytes)?;let owners=vec![1;model.faces.len()];
            (model,owners,64,768,format!("model_id={id}\nmodel_sha256={:x}\n",Sha256::digest(bytes)))
        };
        if model.vertices.is_empty() || model.faces.is_empty() { return Err("empty model".into()); }
        if model.face_textures.as_ref().is_some_and(|t|t.iter().any(|&v|v>=0)) { return Err("static benchmark supports untextured models only".into()); }
        let light=model.light(ambient,contrast,[-30,-50,-30])?;
        let mut triangles=Vec::new();
        for (i,face) in model.faces.iter().enumerate() {
            if light[i][2]==-2 {continue;}
            let vertices=face.map(|v|model.vertices[v as usize]);
            let flat=light[i][2]==-1;
            triangles.push(Triangle {
                material_hsl:Some(model.colors[i]), texture_mapping:None,
                near_alpha:Some(model.alpha.as_ref().map_or(0,|a|a[i])),
                local_vertices:Some(vertices), model_bounds:None,
                depth_bias:model.face_depth_bias(i),
                model_order:Some(gielinor_client::model_order::Submission::new(&model,i,0)),
                vertices,colors:if flat {[light[i][0];3]}else{light[i]},flat,
                uv:[[0.;2];3],texture:None,alpha:model.face_alpha(i),tag:owners[i],view:0,
            });
        }
        if triangles.is_empty() {return Err("no visible model faces".into());}
        let mut low=[f32::INFINITY;3]; let mut high=[f32::NEG_INFINITY;3];
        for v in &model.vertices {for i in 0..3 {low[i]=low[i].min(v[i]);high[i]=high[i].max(v[i]);}}
        append_copies(&mut triangles,copies)?;
        let base_high=high;
        for index in 1..copies {let offset=copy_offset(index);for i in 0..3 {
            // All copies are translated from the same unchanged decoded model.
            high[i]=high[i].max(base_high[i]+offset[i]);
        }}
        let center=std::array::from_fn(|i|(low[i]+high[i])*0.5);
        let extent=(0..3).map(|i|high[i]-low[i]).fold(1.,f32::max);
        let mut camera=Camera::look_at([center[0]+extent,center[1]-extent,center[2]-extent*2.],center,400.)?;
        camera.far=(extent*8.).ceil().max(5250.) as i32;
        let (camera,size,focus,camera_scope)=match recipe {
            Some(r)=>(r.camera,r.size,r.focus,r.scope),
            None=>(camera,[640,480],center,String::new()),
        };
        let palette=gielinor_client::floor::palette(0.8)?; let textures=HashMap::new();
        let refs=triangles.iter().collect::<Vec<_>>();
        let mut classic=Frame::new(size[0] as usize,size[1] as usize)?;
        let mut renderer=Renderer::new(settings.clone())?;
        let mut rows=String::from("frame,total_ms,prepare_ms,render_ms,readback_ms,gpu_ms,uploads\n");
        let mut samples=Vec::new();let mut pixels=Vec::new();
        for index in 0..WARMUP+frames {
            let start=Instant::now();
            pixels=if settings.mode==Mode::Classic {
                classic.clear(0xff10151a);classic.draw(&camera,&triangles,&palette,&textures);classic.pixels.clone()
            }else{
                renderer.render(SceneFrame{camera:&camera,triangles:&refs,stable_count:refs.len(),
                    stable_revision:Some(1),albedo:None,palette:&palette,textures:&textures,
                    size,light_focus:focus,ambient,contrast})?
            };
            let ms=start.elapsed().as_secs_f64()*1000.;
            if index>=WARMUP {
                samples.push(ms);
                writeln!(rows,"{},{ms:.6},{:.6},{:.6},{:.6},{},{}",index-WARMUP,
                    renderer.stats.prepare_ms,renderer.stats.render_ms,renderer.stats.readback_ms,
                    renderer.stats.gpu_ms.map_or(String::new(),|v|v.to_string()),renderer.stats.uploads)?;
            }
        }
        let [mean,p50,p95,p99]=summarize(&samples)?;
        let copy_scope=if copies==1 {String::new()} else {format!("copies={copies}\ncopy_spacing=850\n")};
        let [width,height]=size;
        let report=format!("STATIC MODEL BENCHMARK; NOT FULL CLIENT OR LIVE GAMEPLAY\npackage_sha256={}\n{input_scope}mode={}\nsettings={settings:?}\nsize={width}x{height}\nwarmup={WARMUP}\nmeasured_frames={frames}\n{copy_scope}triangles={}\nmean_ms={mean:.6}\np50_ms={p50:.6}\np95_ms={p95:.6}\np99_ms={p99:.6}\ncamera={:?}\nfocal={}\nfar={}\nbackend={}\n",a[1].to_ascii_lowercase(),settings.mode.name(),triangles.len(),camera.position,camera.focal,camera.far,renderer.stats.json());
        let report=report+&camera_scope;
        let bitmap=Raster::from_pixels(size[0] as usize,size[1] as usize,pixels)?.bmp()?;
        std::fs::create_dir(output)?;
        std::fs::write(output.join("frames.csv"),rows)?;
        std::fs::write(output.join("scope.txt"),report)?;
        std::fs::write(output.join("render.bmp"),bitmap)?;
        println!("{} measured static frames after {WARMUP} warmups; no game login",frames);
        Ok(())
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn timings_reject_invalid_samples_and_use_nearest_rank_indices() {
            assert_eq!(summarize(&[4.,1.,3.,2.]).unwrap(),[2.5,3.,4.,4.]);
            assert_eq!(summarize(&[2.]).unwrap(),[2.;4]);
            for v in [vec![],vec![-1.],vec![f64::NAN],vec![f64::INFINITY]] {assert!(summarize(&v).is_err());}
        }
        #[test]
        fn stress_options_and_expansion_are_bounded() {
            assert_eq!(benchmark_options(&[]).unwrap(),(1,None,None,vec![]));
            assert_eq!(benchmark_options(&["--copies=25".into(),"--ao=4".into()]).unwrap(),(25,None,None,vec!["--ao=4".to_string()]));
            for value in ["0","26","-1","NaN"] {assert!(benchmark_options(&[format!("--copies={value}")]).is_err());}
            assert!(benchmark_options(&["--copies=1".into(),"--copies=2".into()]).is_err());
            assert!(expanded_len(usize::MAX,25).is_err());assert!(expanded_len(40001,25).is_err());assert_eq!(expanded_len(40000,25).unwrap(),1_000_000);
        }
        #[test]
        fn copied_faces_keep_independent_batches_and_do_not_change_the_source() {
            use gielinor_client::{model_bounds::Bounds,model_order::{Submission,Mode as OrderMode}};
            let vertices=[[0.,0.,10.],[0.,10.,10.],[10.,0.,10.]];
            let mut t=Triangle::selection_only(Bounds::new(&vertices,[0;3]),1);t.vertices=vertices;t.local_vertices=Some(vertices);t.model_bounds=None;
            t.model_order=Some(Submission{actor_spot:None,source_ordinal:None,temporary_id:None,scene_height_offset:0,mode:OrderMode::Auto,priority:0,has_priorities:false,has_alpha:false,batch:0});
            let mut faces=vec![t,t];append_copies(&mut faces,6).unwrap();assert_eq!(faces.len(),12);
            for (index,group) in faces.chunks_exact(2).enumerate() {for face in group {
                assert_eq!(face.tag,(index+1)as u64);assert_eq!(face.model_order.unwrap().batch,index*2);
                let expected=vertices.map(|v|std::array::from_fn(|axis|v[axis]+copy_offset(index)[axis]));
                assert_eq!(face.vertices,expected);assert_eq!(face.local_vertices,Some(expected));assert_eq!(face.colors,t.colors);
            }}
            assert_eq!(faces[0].vertices,vertices);assert_eq!(t.vertices,vertices);
        }
        #[test]
        fn frame_budget_is_bounded() {
            for n in [0,601,usize::MAX] {assert!(validate_frames(n).is_err());}
            for n in [1,60,600] {assert!(validate_frames(n).is_ok());}
        }
        #[test]
        fn stress_copies_preserve_distinct_part_owners_even_with_gaps() {
            use gielinor_client::model_bounds::Bounds;
            let vertices=[[0.,0.,10.],[0.,10.,10.],[10.,0.,10.]];
            let mut a=Triangle::selection_only(Bounds::new(&vertices,[0;3]),1);
            a.vertices=vertices;a.model_bounds=None;let mut b=a;b.tag=3;
            let mut faces=vec![a,b];append_copies(&mut faces,3).unwrap();
            assert_eq!(faces.iter().map(|t|t.tag).collect::<Vec<_>>(),vec![1,3,4,6,7,9]);
            assert_eq!(faces[0].vertices,vertices);
            assert_eq!(faces[4].vertices[0],[1700.,0.,10.]);
        }
        #[test]
        fn scene_option_requires_one_nonempty_path() {
            assert!(benchmark_options(&["--scene=".into()]).is_err());
            assert!(benchmark_options(&["--scene=a".into(),"--scene=b".into()]).is_err());
            assert_eq!(benchmark_options(&["--scene=a".into()]).unwrap(),(1,None,Some("a".into()),vec![]));
        }
        #[test]
        fn explicit_cameras_preserve_source_projection_recipes() {
            let look=CameraRecipe::parse("look 0 0 -500 0 0 0 400 640 480").unwrap();
            assert_eq!(look.camera.project([0.;3],[640,480]),Some([320,240]));
            assert_eq!(look.camera.project([100.,0.,0.],[640,480]),Some([400,240]));
            assert_eq!(look.size,[640,480]);assert_eq!(look.camera.far,5250);
            let player=CameraRecipe::parse("player 0 0 0 0 0 400").unwrap();
            assert_eq!(player.camera.position,[0.,0.,-500.]);assert_eq!(player.size,[765,503]);
            assert_eq!(player.camera.project([0.;3],[765,503]),Some([382,251]));
            assert_eq!(player.camera.project([100.,0.,0.],[765,503]),Some([443,251]));
            assert_eq!(player.camera.focal,308.);assert_eq!(player.camera.far,5250);
        }
        #[test]
        fn camera_recipes_reject_malformed_and_unbounded_input() {
            for s in ["","look 0 0 0","player 0 0 0 0 0 400 extra","look NaN 0 -500 0 0 0 400 640 480","look 0 0 -500 0 0 0 inf 640 480","look 0 0 -500 0 0 0 400 0 480","look 0 0 -500 0 0 0 400 640.5 480","look 0 0 -500 0 0 0 400 3841 480","look 0 0 0 0 0 0 400 640 480","player 0 0 0 -1 0 400","player 0 0 0 0 16384 400","player 0 0 0 0 0 12001","player -2147483648 0 0 0 0 400"] {assert!(CameraRecipe::parse(s).is_err(),"{s}");}
            assert!(benchmark_options(&["--camera=".into()]).is_err());
            assert!(benchmark_options(&["--camera=a".into(),"--camera=b".into()]).is_err());
            assert_eq!(benchmark_options(&["--camera=a".into()]).unwrap(),(1,Some("a".into()),None,vec![]));
        }
    }
}
#[cfg(all(feature = "custom-assets", feature = "enhanced-renderer"))]
fn main()->Result<(),Box<dyn std::error::Error>> {benchmark::run()}
