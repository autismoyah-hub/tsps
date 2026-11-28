//! Offline declared-placement collision audit. Never connects to a server.
#[cfg(not(feature="custom-assets"))]
fn main() {eprintln!("build with --features custom-assets");std::process::exit(1);}

#[cfg(feature="custom-assets")]
mod audit {
    use gielinor_client::{authored_floor::{Floor,Placement,ORIGIN},disk_cache::DiskCache,scene::{Scene,PlacedObject},object::Definition};
    use std::{fmt::Write,path::Path};
    fn pin(value:&str)->Result<[u8;32],Box<dyn std::error::Error>> {
        if value.len()!=64 || !value.bytes().all(|b|b.is_ascii_hexdigit()) {return Err("expected SHA256 identity".into());}
        let mut bytes=[0;32];
        for (i,byte) in bytes.iter_mut().enumerate() {*byte=u8::from_str_radix(&value[i*2..i*2+2],16)?;}
        Ok(bytes)
    }
    fn install(scene:&mut Scene,placements:&[Placement])->usize {
        for p in placements {
            let x=ORIGIN[0]+i32::from(p.tile[0])-scene.base_x;
            let z=ORIGIN[1]+i32::from(p.tile[1])-scene.base_y;
            if x<0 || z<0 || x as usize>=scene.width || z as usize>=scene.height {continue;}
            scene.objects.push(PlacedObject{initial_index:Some(scene.objects.len()),rejected:false,runtime:false,
                id:p.id,plane:0,x:x as u8,y:z as u8,shape:p.shape,rotation:p.rotation});
        }
        scene.objects.len()
    }
    fn grid(scene:&Scene)->(String,String) {
        let mut flags=String::from("local_x\tlocal_z\tworld_x\tworld_z\tflags\n");
        let mut steps=String::from("local_x\tlocal_z\tdx\tdz\tcan_step\n");
        for x in 0..scene.width as i32 {for z in 0..scene.height as i32 {
            writeln!(flags,"{x}\t{z}\t{}\t{}\t{}",scene.base_x+x,scene.base_y+z,scene.collision[0].flag_at(x,z).unwrap()).unwrap();
            for dx in -1..=1 {for dz in -1..=1 {if dx!=0 || dz!=0 {
                writeln!(steps,"{x}\t{z}\t{dx}\t{dz}\t{}",gielinor_client::route::can_step(&scene.collision[0],[x,z],1,[dx,dz])).unwrap();
            }}}
        }}
        (flags,steps)
    }
    fn heights(scene:&Scene)->(String,String) {
        let mut vertices=String::from("local_x\tlocal_z\tworld_x\tworld_z\theight\n");
        let mut centers=String::from("local_x\tlocal_z\tworld_fine_x\tworld_fine_z\tloaded\theight\n");
        for x in 0..=scene.width {for z in 0..=scene.height {
            writeln!(vertices,"{x}\t{z}\t{}\t{}\t{}",scene.base_x+x as i32,scene.base_y+z as i32,scene.height(0,x,z).unwrap()).unwrap();
        }}
        for x in 0..scene.width {for z in 0..scene.height {
            let fine=[x as i32*128+64,z as i32*128+64];
            writeln!(centers,"{x}\t{z}\t{}\t{}\t{}\t{}",scene.base_x*128+fine[0],scene.base_y*128+fine[1],scene.loaded(0,x,z),scene.height_fine(0,fine[0],fine[1]).unwrap()).unwrap();
        }}
        (vertices,centers)
    }
    pub fn run()->Result<(),Box<dyn std::error::Error>> {
        let args=std::env::args().skip(1).collect::<Vec<_>>();
        if args.len()!=5 && args.len()!=7 {return Err("PACKAGE PACKAGE_SHA256 FLOOR FLOOR_SHA256 NEW_OUTPUT_DIRECTORY [ORIGIN_DX ORIGIN_DZ]".into());}
        let package_pin=pin(&args[1])?;let floor_pin=pin(&args[3])?;
        let offset=if args.len()==7 {[args[5].parse::<i32>()?,args[6].parse::<i32>()?]}else{[0,0]};
        if offset.iter().any(|v|!(0..=24).contains(v)) {return Err("origin offset must stay in 0..24".into());}
        let output=Path::new(&args[4]);if output.exists() {return Err("output directory must be new".into());}
        let floor=Floor::read(Path::new(&args[2]),floor_pin)?;
        let placements=floor.placements().to_vec();
        let mut cache=DiskCache::open_package_only(&args[0],64<<20,package_pin)?;
        cache.mount_floor(floor,floor_pin)?;
        let mut scene=Scene::authored_floor_study_at(&mut cache,[ORIGIN[0]+offset[0],ORIGIN[1]+offset[1]])?;
        let count=install(&mut scene,&placements);
        let definitions=Definition::load_all(&mut cache)?;
        scene.add_object_collision(&definitions)?;
        let (flags,steps)=grid(&scene);
        let (vertices,centers)=heights(&scene);
        let mut metadata=String::from("id\tsize_x\tsize_z\tinteract_type\tblocks_projectiles\tobstructs_ground\tsolid\tmodel_count\n");
        for id in scene.objects.iter().map(|o|o.id).collect::<std::collections::BTreeSet<_>>() {
            let d=definitions.get(&id).ok_or("declared placement definition missing")?;
            writeln!(metadata,"{id}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",d.size[0],d.size[1],d.interact_type,d.blocks_projectiles,d.obstructs_ground,d.solid,d.models.len())?;
        }
        let summary=format!("OFFLINE DECLARED PLACEMENTS; NOT LIVE GAMEPLAY\npackage_sha256={}\nfloor_sha256={}\nscene_base={},{}\nsize={},{}\nplacements={}\nflag_rows={}\nstep_rows={}\n",args[1].to_ascii_lowercase(),args[3].to_ascii_lowercase(),scene.base_x,scene.base_y,scene.width,scene.height,count,scene.width*scene.height,scene.width*scene.height*8);
        std::fs::create_dir(output)?;
        for (name,text) in [("collision-flags.tsv",flags),("eight-direction-steps.tsv",steps),("object-definitions.tsv",metadata),("vertex-heights.tsv",vertices),("tile-center-heights.tsv",centers),("scope.txt",summary)] {std::fs::write(output.join(name),text)?;}
        println!("Offline declared-placement audit: {count} placements, {} flags and {} directional steps; no server connection",scene.width*scene.height,scene.width*scene.height*8);
        Ok(())
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn height_exports_include_border_and_native_interpolation_without_mutation() {
            use gielinor_client::{terrain::{Region,Tile},rebuild::Chunk};
            let mut scene=Scene::empty_sized(1000,2000,12,12).unwrap();
            let mut region=Region {tiles:vec![Tile::default();4*4096],extension_flags:0,height_rules:vec![Some(0);4*4096]};
            for x in 0..64 {for z in 0..64 {region.height_rules[x*64+z]=Some((2+x+z)as u8);}}
            scene.add_chunk(&region,Chunk{plane:0,zone_x:0,zone_y:0,rotation:0},0,0,0).unwrap();
            let before=(0..=12).flat_map(|x|(0..=12).map(move|z|(x,z))).map(|(x,z)|scene.height(0,x,z)).collect::<Vec<_>>();
            let first=heights(&scene);assert_eq!(first,heights(&scene));
            assert_eq!(first.0.lines().count(),170);assert_eq!(first.1.lines().count(),145);
            assert!(first.0.lines().any(|l|l=="2\t3\t1002\t2003\t-56"));
            assert!(first.0.lines().any(|l|l.starts_with("12\t12\t1012\t2012\t")));
            assert!(first.1.lines().any(|l|l=="2\t3\t128320\t256448\ttrue\t-64"));
            assert!(first.1.lines().any(|l|l.contains("\tfalse\t")));
            assert_eq!(before,(0..=12).flat_map(|x|(0..=12).map(move|z|(x,z))).map(|(x,z)|scene.height(0,x,z)).collect::<Vec<_>>());
        }
        #[test]
        fn placement_crop_preserves_declared_identity() {
            let mut scene=Scene::empty(ORIGIN[0]+24,ORIGIN[1]+24).unwrap();
            let placements=[Placement{id:1,tile:[0,0],shape:10,rotation:0},Placement{id:2,tile:[24,24],shape:2,rotation:3},Placement{id:3,tile:[127,127],shape:22,rotation:1}];
            assert_eq!(install(&mut scene,&placements),2);
            assert_eq!((scene.objects[0].id,scene.objects[0].x,scene.objects[0].y,scene.objects[0].shape,scene.objects[0].rotation),(2,0,0,2,3));
            assert_eq!((scene.objects[1].id,scene.objects[1].x,scene.objects[1].y),(3,103,103));
        }
        #[test]
        fn exported_coordinates_and_steps_are_deterministic_and_read_only() {
            let mut scene=Scene::empty_sized(1000,2000,12,12).unwrap();
            for x in 0..12 {for z in 0..12 {scene.collision[0].change(x,z,u32::MAX,false).unwrap();}}
            scene.collision[0].rectangle(5,5,1,1,true,true);
            let before=scene.collision[0].flags().to_vec();
            let first=grid(&scene);assert_eq!(first,grid(&scene));assert_eq!(scene.collision[0].flags(),before);
            assert_eq!(first.0.lines().count(),145);assert_eq!(first.1.lines().count(),1153);
            assert!(first.0.lines().any(|line|line=="5\t5\t1005\t2005\t131328"));
            assert!(first.1.lines().any(|line|line=="4\t5\t1\t0\tfalse"));
            assert!(first.1.lines().any(|line|line=="4\t5\t-1\t0\ttrue"));
        }
    }
}
#[cfg(feature="custom-assets")]
fn main()->Result<(),Box<dyn std::error::Error>> {audit::run()}
