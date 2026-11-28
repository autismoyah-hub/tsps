use gielinor_client::{
    disk_cache::DiskCache,
    object::Definition,
    scene::{PlacedObject, Scene},
    script_runtime::Varbit,
    world_mesh::WorldMesh,
    Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open(std::env::args().nth(1).expect("cache path"), 64 << 20)?;
    let defs = Definition::load_all(&mut cache)?;
    let mut ids = defs.keys().copied().collect::<Vec<_>>();
    ids.sort_unstable();
    let mut chosen = None;
    for id in ids {
        let d = &defs[&id];
        let Some(targets) = &d.transforms else {
            continue;
        };
        let Some(first) = targets.first().copied().flatten() else {
            continue;
        };
        if defs[&(first as u32)].model_ids(10).is_empty() {
            continue;
        }
        let Some(second) = targets[..targets.len() - 1]
            .iter()
            .position(|v| v.is_none())
        else {
            continue;
        };
        let (base, low) = if let Some(bit) = d.transform_varbit {
            let b = Varbit::decode(&cache.file(2, 14, bit as u32, [0; 4])?)?;
            if second as u32 >= (1u32 << (b.high - b.low + 1)) {
                continue;
            }
            (b.base, b.low)
        } else if let Some(varp) = d.transform_varp {
            (varp, 0)
        } else {
            continue;
        };
        chosen = Some((id, base, low, second));
        break;
    }
    let (id, base, low, second) = chosen.expect("visible/hidden transform");
    let mut scene = Scene::empty(0, 0)?;
    scene.objects.push(PlacedObject { initial_index: None, rejected: false,
        runtime: false,
        id,
        plane: 0,
        x: 50,
        y: 50,
        shape: 10,
        rotation: 1,
    });
    let mut values = vec![0; 65536];
    let visible = WorldMesh::build_with_varps(&scene, &mut cache, 0, &values)?;
    assert!(!visible.triangles.is_empty());
    assert!(!visible.transforms_changed(&values));
    values[(base as usize + 1) % 65536] = 1;
    assert!(!visible.transforms_changed(&values));
    values[base as usize] = (second as i32) << low;
    assert!(visible.transforms_changed(&values));
    let hidden = WorldMesh::build_with_varps(&scene, &mut cache, 0, &values)?;
    assert!(hidden.triangles.is_empty());
    assert!(!hidden.transforms_changed(&values));
    values[base as usize] = 0;
    assert!(hidden.transforms_changed(&values));
    let restored = WorldMesh::build_with_varps(&scene, &mut cache, 0, &values)?;
    assert_eq!(
        visible
            .triangles
            .iter()
            .map(|t| t.vertices)
            .collect::<Vec<_>>(),
        restored
            .triangles
            .iter()
            .map(|t| t.vertices)
            .collect::<Vec<_>>()
    );
    println!("Object {id}: varp {base}, selector {second}, {} visible triangles; hide/restore and dependency tracking passed",visible.triangles.len());
    Ok(())
}
