use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    map_composite::Composite,
    map_lookup,
    vm::{Control, Host, Object, Stacks},
    Result,
};
fn command(c: &mut ClientState, op: u16, args: &[i32]) -> Result<Stacks> {
    let mut s = Stacks::default();
    for &v in args {
        s.push_int(v)?;
    }
    assert_eq!(c.extended(op, false, &mut s)?, Control::Continue);
    Ok(s)
}
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    command(&mut c, 6602, &[0])?;
    assert_eq!(command(&mut c, 6639, &[])?.ints(), &[-1, -1]);
    let expected =
        map_lookup::area_at(c.world_map.areas.as_ref().unwrap(), [0, 3200, 3200]).unwrap_or(-1);
    assert_eq!(
        command(&mut c, 6623, &[3200 << 14 | 3200])?.ints(),
        &[expected]
    );
    command(&mut c, 6693, &[-1])?;
    let defs = c.world_map.elements.as_ref().unwrap().clone();
    for (id, def) in &defs {
        let s = command(&mut c, 6693, &[*id as i32])?;
        match &s.objects()[0] {
            Object::String(s) => assert_eq!(s, def.name.as_deref().unwrap_or("")),
            _ => panic!("name expected"),
        };
        assert_eq!(
            command(&mut c, 6694, &[*id as i32])?.ints(),
            &[def.text_size as i32]
        );
        assert_eq!(
            command(&mut c, 6695, &[*id as i32])?.ints(),
            &[def.category]
        );
        assert_eq!(
            command(&mut c, 6696, &[*id as i32])?.ints(),
            &[def.sprites[0].map_or(-1, |id| id as i32)]
        );
    }
    let composite = Composite::decode(&c.scripts.source.cache.file(19, 1, 0, [0; 4])?, true)?;
    let icon = composite
        .icons
        .iter()
        .find(|i| {
            defs.get(&(i.element as u32))
                .is_some_and(|d| d.world_map && d.sprites[0].is_some())
        })
        .unwrap();
    command(&mut c, 6607, &[icon.coord[1] << 14 | icon.coord[2]])?;
    c.world_map.zoom = 4.0;
    c.world_map.target_zoom = 4.0;
    let image = c
        .world_map
        .detailed_image(&mut c.scripts.source.cache, true, [256, 256], 0.8)?
        .unwrap();
    command(&mut c, 6633, &[icon.element, 0])?;
    assert_eq!(command(&mut c, 6636, &[icon.element])?.ints(), &[0]);
    let hidden = c
        .world_map
        .detailed_image(&mut c.scripts.source.cache, true, [256, 256], 0.8)?
        .unwrap();
    assert_ne!(image.pixels, hidden.pixels);
    command(&mut c, 6633, &[icon.element, 1])?;
    let restored = c
        .world_map
        .detailed_image(&mut c.scripts.source.cache, true, [256, 256], 0.8)?
        .unwrap();
    assert_eq!(image.pixels, restored.pixels);
    command(&mut c, 6624, &[1])?;
    command(&mut c, 6626, &[2])?;
    command(&mut c, 6629, &[icon.element])?;
    let flash = c
        .world_map
        .detailed_image(&mut c.scripts.source.cache, true, [256, 256], 0.8)?
        .unwrap();
    assert_eq!(c.world_map.settings.flash_cycle, 1);
    assert!(c.world_map.settings.flashing.is_some());
    assert!(!std::sync::Arc::ptr_eq(&restored, &flash));
    command(&mut c, 6632, &[0])?;
    c.world_map
        .detailed_image(&mut c.scripts.source.cache, true, [256, 256], 0.8)?;
    assert!(c.world_map.settings.flashing.is_none());
    assert_eq!(c.world_map.settings.flash_count, 1);
    let first = command(&mut c, 6639, &[])?.ints().to_vec();
    assert!(first[0] >= 0);
    assert_eq!(command(&mut c, 6638, &first)?.ints(), &[first[1]]);
    let mut count = 1;
    loop {
        let next = command(&mut c, 6640, &[])?.ints().to_vec();
        if next == [-1, -1] {
            break;
        }
        count += 1;
        assert!(count < 10000);
    }
    assert_eq!(command(&mut c, 6640, &[])?.ints(), &[-1, -1]);
    command(&mut c, 6602, &[1])?;
    command(&mut c, 6602, &[0])?;
    assert_eq!(command(&mut c, 6639, &[])?.ints(), &[-1, -1]);
    assert_eq!(command(&mut c, 6638, &first)?.ints(), &[-1]);
    let settings = c.world_map.settings.clone();
    c.prepare_login()?;
    assert_eq!(c.world_map.settings, settings);
    assert_eq!(command(&mut c, 6635, &[])?.ints(), &[0]);
    println!("Map lookup, {} element metadata sets, visible icon filter/restore, flash frame/expiry while hidden, icon iteration/nearest/exhaustion/area reset, and login retention passed",defs.len());
    Ok(())
}
