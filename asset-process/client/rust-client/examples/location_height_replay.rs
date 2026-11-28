//! Cache-backed bridge placement regression; no networking or cache mutation.
use gielinor_client::{
    disk_cache::DiskCache,
    location_update::Object,
    object::Definition,
    object_model::Models,
    rebuild::Chunk,
    scene::{PlacedObject, Scene},
    terrain::{Region, Tile},
    world_mesh::WorldMesh,
};

fn scene(bridge: bool) -> gielinor_client::Result<Scene> {
    terrain_scene([0, if bridge { 2 } else { 0 }, 0, 0], false)
}

fn terrain_scene(flags: [u8; 4], floors: bool) -> gielinor_client::Result<Scene> {
    let mut region = Region {
        tiles: vec![Tile::default(); 4 * 4096],
        extension_flags: 0,
        height_rules: vec![Some(30); 4 * 4096],
    };
    for (plane, &flags) in flags.iter().enumerate() {
        for tile in &mut region.tiles[plane * 4096..(plane + 1) * 4096] {
            tile.flags = flags;
            tile.underlay = u16::from(floors);
        }
    }
    let mut scene = Scene::empty_sized(0, 0, 16, 16)?;
    for plane in 0..4 {
        for x in [0, 8] {
            for y in [0, 8] {
                scene.add_chunk(
                    &region,
                    Chunk {
                        plane: plane as u8,
                        rotation: 0,
                        zone_x: 0,
                        zone_y: 0,
                    },
                    plane,
                    x,
                    y,
                )?;
            }
        }
    }
    Ok(scene)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let defs = Definition::load_all(&mut cache)?;
    let mut candidates = defs
        .values()
        .filter(|d| {
            d.support_items == 1
                && d.animation.is_none()
                && d.transforms.is_none()
                && d.size.iter().all(|&size| size <= 2)
                && !d.model_ids(10).is_empty()
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|d| d.id);
    let mut models = Models::default();
    let mut id = None;
    for def in candidates {
        if models
            .object(&mut cache, def, 10, 0)?
            .is_some_and(|m| !m.faces.is_empty())
        {
            id = Some(def.id);
            break;
        }
    }
    let id = id.ok_or("no static cache model")?;
    let object = Object {
        id: id as i32,
        shape: 10,
        rotation: 0,
    };
    for bridge_flag in [false, true] {
        for occupied in [false, true] {
            let physical = if bridge_flag { 3 } else { 2 };
            let mut flags = [0; 4];
            flags[physical] = 8;
            if bridge_flag {
                flags[1] = 2;
            }
            let mut slots = terrain_scene(flags, false)?;
            if occupied {
                slots.objects.push(PlacedObject { initial_index: None, rejected: false,
                    runtime: false,
                    id,
                    plane: physical as u8,
                    x: 4,
                    y: 4,
                    shape: 10,
                    rotation: 0,
                });
            }
            slots.add_object_collision(&defs)?;
            let key = [2, 4, 4, 2];
            slots.replace_location(key, &Object { id: -1, ..object }, &defs)?;
            slots.replace_location(key, &object, &defs)?;
            let visible = WorldMesh::build(&slots, &mut cache, 0)?;
            assert_eq!(
                visible.object_instances > 0,
                occupied,
                "runtime force inheritance bridge={bridge_flag}"
            );
            let all = WorldMesh::build(&slots, &mut cache, 3)?;
            assert!(all.object_instances > 0);
            assert_eq!(all.visible_triangles(0).count(), visible.triangles.len());
            slots.replace_location(key, &Object { id: -1, ..object }, &defs)?;
            slots.replace_location(key, &object, &defs)?;
            assert_eq!(
                WorldMesh::build(&slots, &mut cache, 0)?.object_instances,
                visible.object_instances
            );
        }
    }
    println!("Runtime force-ground slot inheritance and replacement preserve expected roof visibility with and without bridges");
    let mut plain = scene(false)?;
    let mut bridge = scene(true)?;
    let key = [0, 4, 4, 2];
    plain.replace_location(key, &object, &defs)?;
    bridge.replace_location(key, &object, &defs)?;
    let a = WorldMesh::build(&plain, &mut cache, 0)?;
    let b = WorldMesh::build(&bridge, &mut cache, 0)?;
    assert!(!a.triangles.is_empty());
    assert_eq!(a.triangles.len(), b.triangles.len());
    for (a, b) in a.triangles.iter().zip(&b.triangles) {
        assert_eq!(a.tag, b.tag, "height plane leaked into scene pick tag");
        for (va, vb) in a.vertices.iter().zip(b.vertices) {
            assert_eq!(vb, [va[0], va[1] - 240.0, va[2]]);
        }
    }
    let mut map = scene(true)?;
    map.objects.push(PlacedObject { initial_index: None, rejected: false,
        runtime: false,
        id,
        plane: 0,
        x: 4,
        y: 4,
        shape: 10,
        rotation: 0,
    });
    let c = WorldMesh::build(&map, &mut cache, 0)?;
    assert_eq!(a.triangles.len(), c.triangles.len());
    for (a, c) in a.triangles.iter().zip(&c.triangles) {
        assert_eq!(a.vertices, c.vertices);
    }
    assert!(
        c.item_support.is_empty(),
        "below-bridge geometry supported a logical ground pile"
    );
    assert!(!a.item_support.is_empty());
    map.objects.push(PlacedObject { initial_index: None, rejected: false,
        runtime: false,
        id,
        plane: 1,
        x: 4,
        y: 4,
        shape: 10,
        rotation: 0,
    });
    assert_eq!(
        map.location(key).id,
        id as i32,
        "lowered static slot missing"
    );
    let top = WorldMesh::build(&map, &mut cache, 0)?;
    assert_eq!(
        top.triangles.len(),
        a.triangles.len() + b.triangles.len(),
        "upper bridge object omitted at scene level zero"
    );
    assert_eq!(
        top.item_support, a.item_support,
        "item support used the physical plane"
    );
    map.replace_location(key, &Object { id: -1, ..object }, &defs)?;
    assert_eq!(
        map.objects.len(),
        1,
        "removal touched the below-bridge object"
    );
    assert_eq!(map.objects[0].plane, 0);
    map.replace_location(key, &object, &defs)?;
    let replaced = WorldMesh::build(&map, &mut cache, 1)?;
    assert_eq!(
        replaced.triangles.len(),
        a.triangles.len() + b.triangles.len()
    );
    for (actual, expected) in replaced
        .triangles
        .iter()
        .zip(a.triangles.iter().chain(&b.triangles))
    {
        assert_eq!(
            actual.vertices, expected.vertices,
            "static and runtime objects sharing plane zero lost independent heights"
        );
    }
    assert_eq!(
        plain.collision[0].flag_at(4, 4),
        bridge.collision[0].flag_at(4, 4)
    );
    bridge.replace_location(key, &Object { id: -1, ..object }, &defs)?;
    assert!(WorldMesh::build(&bridge, &mut cache, 0)?
        .triangles
        .is_empty());
    let floor_count = |mesh: &WorldMesh, plane: u64| {
        mesh.triangles
            .iter()
            .filter(|t| t.tag >> 63 != 0 && (t.tag >> 28) & 3 == plane)
            .count()
    };
    let plain_floor = WorldMesh::build(&terrain_scene([0; 4], true)?, &mut cache, 0)?;
    let bridge_floor = WorldMesh::build(&terrain_scene([0, 2, 0, 0], true)?, &mut cache, 0)?;
    let forced_floor = WorldMesh::build(&terrain_scene([0, 0, 0, 8], true)?, &mut cache, 0)?;
    let count = floor_count(&plain_floor, 0);
    assert!(count > 0);
    assert_eq!(plain_floor.triangles.len(), count);
    assert_eq!(floor_count(&bridge_floor, 0), count);
    assert_eq!(floor_count(&bridge_floor, 1), count);
    assert_eq!(bridge_floor.triangles.len(), count * 2);
    assert_eq!(floor_count(&forced_floor, 0), count);
    assert_eq!(floor_count(&forced_floor, 3), count);
    assert_eq!(forced_floor.triangles.len(), count * 2);
    for flags in [[0; 4], [0, 2, 0, 0], [0, 0, 0, 8]] {
        let scene = terrain_scene(flags, true)?;
        let all = WorldMesh::build(&scene, &mut cache, 3)?;
        for level in 0..4 {
            let limited = WorldMesh::build(&scene, &mut cache, level)?;
            let visible: Vec<_> = all.visible_triangles(level as u8).collect();
            assert_eq!(visible.len(), limited.triangles.len());
            for (a, b) in visible.iter().zip(&limited.triangles) {
                assert_eq!(a.tag, b.tag);
                assert_eq!(a.vertices, b.vertices);
            }
        }
    }
    println!("All-plane terrain submission matches rebuilt geometry at every roof level, including bridge/force-ground floors");
    println!("Terrain inclusion: {count} triangles per physical floor, upper bridge and force-ground floor included at scene level zero; support uses logical slots");
    println!("Cache model {id}: {} triangles, bridge height -240, stable pick tags/collision plane, static placement unchanged, removal cleared geometry", a.triangles.len());
    for bridge_flag in [false, true] {
        let physical = u8::from(bridge_flag);
        let mut crowded = scene(bridge_flag)?;
        for _ in 0..6 { crowded.objects.push(PlacedObject { initial_index: None, rejected:false,runtime:false,id,plane:physical,x:4,y:4,shape:10,rotation:0 }); }
        crowded.add_object_collision(&defs)?;
        assert_eq!(crowded.objects.iter().filter(|o| o.rejected).count(),1);
        let full = WorldMesh::build(&crowded,&mut cache,0)?;
        assert_eq!(full.triangles.len(),a.triangles.len()*5,"only five cache meshes submitted");
        for _ in 0..5 { crowded.replace_location(key,&Object{id:-1,..object},&defs)?; }
        assert_eq!(crowded.location(key).id,-1,"rejected cache object has no scene lookup entry");
        assert!(WorldMesh::build(&crowded,&mut cache,0)?.triangles.is_empty());
        crowded.replace_location(key,&object,&defs)?;
        let restored = WorldMesh::build(&crowded,&mut cache,0)?;
        assert_eq!(restored.triangles.len(),a.triangles.len(),"capacity freed through physical/bridge removal");
    }
    println!("Five-object capacity: cache meshes, rejected lookup, removal and reinsertion pass on ordinary and bridge-lowered slots");
    let mut missing_shapes = 0;
    for shape in [0,4,22] {
        if !defs[&id].model_ids(shape).is_empty() { continue; }
        let mut missing = scene(false)?;
        let category = gielinor_client::location_update::category(shape)? as i32;
        missing.replace_location([0,4,4,category],&Object{id:id as i32,shape,rotation:0},&defs)?;
        assert_eq!(missing.location([0,4,4,category]).id,-1);
        assert!(missing.objects.is_empty());
        assert!(WorldMesh::build(&missing,&mut cache,3)?.triangles.is_empty());
        let mut raster = gielinor_client::raster::Raster::new(512,512,512*512)?;
        raster.clear(0xff302b25);
        gielinor_client::minimap::objects(&missing,&defs,&[],0,&mut raster)?;
        assert!(raster.pixels().iter().all(|&v|v==0xff302b25));
        missing_shapes += 1;
    }
    assert!(missing_shapes > 0);
    println!("Null cache model shapes: {missing_shapes} categories omit lookup, world geometry and minimap pixels");
    Ok(())
}
