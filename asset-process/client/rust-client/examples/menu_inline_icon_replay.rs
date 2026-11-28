use gielinor_client::{
    buffer::Reader,
    context_menu::{Entry, Menu},
    crypto::crc32,
    raster::Raster,
};
fn root_menu() {
    let mut cache =
        gielinor_client::disk_cache::DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)
            .unwrap();
    let mut renderer = gielinor_client::interface_render::Renderer::load(&mut cache, 0.8).unwrap();
    let loaded_icons = renderer.text_icons.clone();
    let font = renderer.menu_font(&mut cache).unwrap();
    let mut r = Reader::new(include_bytes!("../tests/fixtures/menu-inline-icon-jvm.bin"));
    let icons = loaded_icons;
    for case in 0..r.u32().unwrap() {
        let count = r.u32().unwrap();
        let point = std::array::from_fn(|_| r.u32().unwrap() as i32);
        let scroll = r.u32().unwrap() as i32;
        let mut entries = Vec::new();
        for _ in 0..count {
            let len = r.u16().unwrap() as usize;
            let option = String::from_utf8(r.bytes(len).unwrap().to_vec()).unwrap();
            let len = r.u16().unwrap() as usize;
            let target = String::from_utf8(r.bytes(len).unwrap().to_vec()).unwrap();
            entries.push(Entry {
                option,
                target,
                action: (),
                submenu: r.u8().unwrap() != 0,
                children: Vec::new(),
            });
        }
        let mut menu = Menu::open_with_icons(entries, point, [320, 240], &font, icons.clone());
        menu.scroll = scroll;
        let bounds: [i32; 4] = std::array::from_fn(|_| r.u32().unwrap() as i32);
        assert_eq!(menu.bounds, bounds, "bounds {case}");
        let pointer = std::array::from_fn(|_| r.u32().unwrap() as i32);
        let hit = r.u32().unwrap() as i32;
        assert_eq!(
            menu.select(pointer).map_or(-1, |i| i as i32),
            hit,
            "hit {case}"
        );
        let mut raster = Raster::new(320, 240, 320 * 240).unwrap();
        raster.clear(0xff223344);
        menu.draw(&mut raster, &font, pointer).unwrap();
        assert_eq!(
            crc32(
                &raster
                    .pixels()
                    .iter()
                    .flat_map(|v| v.to_be_bytes())
                    .collect::<Vec<_>>()
            ),
            r.u32().unwrap(),
            "pixels {case}"
        );
    }
    assert_eq!(r.remaining(), 0);
}

fn child_menu() {
    let mut cache =
        gielinor_client::disk_cache::DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)
            .unwrap();
    let mut renderer = gielinor_client::interface_render::Renderer::load(&mut cache, 0.8).unwrap();
    let loaded_icons = renderer.text_icons.clone();
    let font = renderer.menu_font(&mut cache).unwrap();
    let mut r = Reader::new(include_bytes!("../tests/fixtures/menu-hover-icon-jvm.bin"));
    let icons = loaded_icons;
    for case in 0..r.u32().unwrap() {
        let point = std::array::from_fn(|_| r.u32().unwrap() as i32);
        let entries = (0..4)
            .map(|i| Entry {
                option: format!("<img=1>Action{i}"),
                target: "<img=0>Target".into(),
                action: i,
                submenu: false,
                children: (0..3)
                    .map(|j| Entry {
                        option: format!("<img=59>Sub{j}"),
                        target: "<img=-0>".into(),
                        action: 100 + i * 10 + j,
                        submenu: false,
                        children: Vec::new(),
                    })
                    .collect(),
            })
            .collect();
        let mut menu = Menu::open_with_icons(entries, point, [320, 240], &font, icons.clone());
        let mut pointer = [0; 2];
        for step in 0..48 {
            pointer = std::array::from_fn(|_| r.u32().unwrap() as i32);
            let inside = r.u8().unwrap() != 0;
            assert_eq!(
                menu.hover(pointer, step, [320, 240], &font),
                inside,
                "inside {case} {step}"
            );
            let active = r.u32().unwrap() as i32;
            assert_eq!(
                menu.active.as_ref().map_or(-1, |(i, _)| *i as i32),
                active,
                "active {case} {step}"
            );
            if active >= 0 {
                let bounds: [i32; 4] = std::array::from_fn(|_| r.u32().unwrap() as i32);
                assert_eq!(
                    menu.active.as_ref().unwrap().1.bounds,
                    bounds,
                    "child bounds {case} {step}"
                );
            }
        }
        let mut raster = Raster::new(320, 240, 320 * 240).unwrap();
        raster.clear(0xff223344);
        menu.draw(&mut raster, &font, pointer).unwrap();
        assert_eq!(
            crc32(
                &raster
                    .pixels()
                    .iter()
                    .flat_map(|v| v.to_be_bytes())
                    .collect::<Vec<_>>()
            ),
            r.u32().unwrap(),
            "pixels {case}"
        );
    }
    assert_eq!(r.remaining(), 0);
}

fn main() {
    root_menu();
    child_menu();
    println!("4096 root menu rasters plus 6144 submenu states / 128 rasters pass actual cache fonts and icons");
}
