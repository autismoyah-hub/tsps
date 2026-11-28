use gielinor_client::{
    disk_cache::DiskCache, identity_kit::Kit, interface_model, item::Definitions, item_icon::Icons,
    item_model::Models, player_model::Appearance, raster::Raster,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    let kits = Kit::load_all(&mut cache)?;
    let items = Definitions::load(&mut cache)?;
    let mut models = Models::default();
    let icons = Icons::load(&mut cache, 0.8)?;
    let mut raster = Raster::new(960, 640, 1 << 20)?;
    raster.clear(0xff302b25);
    for i in 0..8 {
        let mut player = Appearance::standard(&kits, (i % 2) as u8);
        player.colors = [i as u8, 2, 3, 1, 0];
        if i >= 4 {
            for (slot, id) in [(3, 4151), (4, 1127), (5, 1187), (7, 1079), (10, 11840)] {
                player.equipment[slot] = id + 2048;
            }
        }
        let head = i % 4 >= 2;
        let m = player.mesh(&mut cache, &mut models, &kits, &items, head)?;
        let colors = m.light(
            64,
            if head { 768 } else { 850 },
            if head {
                [-50, -10, -50]
            } else {
                [-30, -50, -30]
            },
        )?;
        let vertices =
            interface_model::vertices(&m, [0, 192, 0], [0, 0], if head { 350 } else { 700 }, true)?;
        let sprite = icons.draw_model(&m, &colors, &vertices, [240, 320], None)?;
        sprite.draw(&mut raster, (i % 4) * 240, (i / 4) * 320, 256)?;
        println!(
            "player {i}: head={head}, {} vertices {} faces",
            m.vertices.len(),
            m.faces.len()
        );
    }
    std::fs::write(&a[2], raster.bmp()?)?;
    Ok(())
}
