//! Native decoded terrain, assembled from ordinary104-tile views for offline review.
//! No game login. The player scene size and renderer are unchanged.
#[cfg(not(all(feature="custom-assets",feature="enhanced-renderer")))]
fn main(){panic!("requires custom-assets,enhanced-renderer");}
#[cfg(all(feature="custom-assets",feature="enhanced-renderer"))]
fn main()->Result<(),Box<dyn std::error::Error>>{
 use gielinor_client::{asset_package::Package,disk_cache::DiskCache,rebuild::Normal,scene::Scene,world_mesh::WorldMesh,terrain::Region,render3d::{Camera,Frame},enhanced::{Settings,Renderer,Mode,SceneFrame}};
 use std::{path::Path,collections::{HashMap,BTreeMap},fmt::Write};
 let a=std::env::args().skip(1).collect::<Vec<_>>();if a.len()<6{return Err("PACKAGE EXPECTED_SHA256 CACHE NEW_OUTPUT MODE ADAPTER [renderer options]".into());}
 if a[1].len()!=64{return Err("SHA256 length".into());}let mut pin=[0;32];for(i,v)in pin.iter_mut().enumerate(){*v=u8::from_str_radix(&a[1][i*2..i*2+2],16)?;}
 let output=Path::new(&a[3]);if output.exists(){return Err("output must be new".into());}
 let package=Package::read(Path::new(&a[0]))?;if package.identity()!=pin{return Err("package pin mismatch".into());}
 let mut regions=BTreeMap::new();for id in package.terrain_ids(){regions.insert(id,Region::decode(package.terrain(id).unwrap(),((id>>8)*64)as i32,((id&255)*64)as i32)?);}
 let mut cache=DiskCache::open(&a[2],64<<20)?;let original=cache.login_crcs()?;cache.mount_package(package,pin)?;
 let mut decoded=String::from("local_x,local_z,world_x,world_z,ground_height,upper_height,upper_flags\n");
 for x in 0..=192{for z in 0..=192{let wx=11008+x;let wz=512+z;let id=((wx/64)<<8)|(wz/64);let r=&regions[&(id as u32)];let i=(wx as usize&63)*64+(wz as usize&63);writeln!(decoded,"{x},{z},{wx},{wz},{},{},{}",-r.tiles[i].height,-r.tiles[4096+i].height,r.tiles[4096+i].flags)?;}}
 let mut triangles=Vec::new();let mut views=Vec::new();let mut samples=String::from("x,z,fx,fz,elevation,collision\n");
 for mx in 0..3{for mz in 0..3{
  let wx=11008+mx*64;let wz=512+mz*64;
  let rebuild=Normal{world_area:0,zone_x:((wx+32)>>3)as u16,zone_y:((wz+32)>>3)as u16,players:None};
  let scene=Scene::load_normal(&mut cache,&rebuild)?;if !scene.missing_regions.is_empty(){return Err(format!("missing atlas neighbors {:?}",scene.missing_regions).into());}
  for x in wx..wx+64{for z in wz..wz+64{let lx=x-scene.base_x;let lz=z-scene.base_y;for(fx,fz)in[(32,32),(32,96),(64,64),(96,32),(96,96)]{writeln!(samples,"{x},{z},{fx},{fz},{},{}",scene.elevation(0,lx*128+fx,lz*128+fz).ok_or("native elevation absent")?,scene.collision[0].flag_at(lx,lz).ok_or("native collision absent")?)?;}}}
  let mesh=WorldMesh::build(&scene,&mut cache,0)?;if mesh.pending_assets{return Err("pending atlas assets".into());}
  let dx=((scene.base_x-11008)*128)as f32;let dz=((scene.base_y-512)*128)as f32;
  for source in mesh.visible_triangles(0){
   let tx=scene.base_x+((source.tag>>14)&16383)as i32;let tz=scene.base_y+(source.tag&16383)as i32;
   if !(wx..wx+64).contains(&tx)||!(wz..wz+64).contains(&tz){continue;}
   let mut t=*source;for v in &mut t.vertices{v[0]+=dx;v[2]+=dz;}t.local_vertices=None;t.model_bounds=None;t.model_order=None;triangles.push(t);
  }
  views.push(format!("view={mx},{mz} base={},{} dimensions={}x{}",scene.base_x,scene.base_y,scene.width,scene.height));
 }}
 if cache.login_crcs()?!=original{return Err("original references changed".into());}
 let settings=Settings::from_launch_args(&a[4],&a[5],&a[6..])?;let mut renderer=Renderer::new(settings.clone())?;let palette=gielinor_client::floor::palette(0.8)?;let textures=HashMap::new();
 std::fs::create_dir(output)?;std::fs::write(output.join("decoded-corners.csv"),decoded)?;
 std::fs::write(output.join("native-samples.csv"),samples)?;
 let mut scope=format!("OFFLINE NATIVE TERRAIN REVIEW; NOT LIVE GAMEPLAY\npackage={}\nsource_views=9\ntriangles={}\nmode={}\noriginal_login_crcs={original:?}\n{}\n",a[1],triangles.len(),a[4],views.join("\n"));
 let shots=[("overview",[12288.,-35000.,11800.],[12288.,-550.,12288.],1350.,80000),("oblique",[30000.,-18000.,-13000.],[12288.,-600.,12288.],1400.,80000),("harbor-to-headland",[18000.,-800.,8000.],[12000.,-1000.,16000.],900.,20000)];
 for(name,eye,focus,focal,far)in shots{
  let mut camera=Camera::look_at(eye,focus,focal)?;camera.far=far;
  let mut frame=Frame::new(1440,1040)?;frame.clear(0xff254c67);
  if settings.mode==Mode::Classic{frame.draw(&camera,&triangles,&palette,&textures);}else{let refs=triangles.iter().collect::<Vec<_>>();frame.pixels=renderer.render(SceneFrame{camera:&camera,triangles:&refs,stable_count:refs.len(),stable_revision:Some(1),albedo:None,palette:&palette,textures:&textures,size:[1440,1040],light_focus:focus,ambient:64,contrast:768})?;}
  std::fs::write(output.join(format!("{name}.bmp")),frame.raster()?.bmp()?)?;writeln!(scope,"{name}: eye={eye:?} focus={focus:?} focal={focal} far={far}")?;
 }
 std::fs::write(output.join("scope.txt"),scope)?;println!("Native terrain review: {} triangles;9 unchanged104-tile views; no game login",triangles.len());Ok(())
}
