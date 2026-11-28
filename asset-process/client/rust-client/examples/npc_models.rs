use gielinor_client::{
    disk_cache::DiskCache,
    interface_render::Renderer,
    interface_state::{Interfaces, ModelSource},
    item::Definitions,
    item_model::Models,
    npc::Definition,
    npc_model,
    raster::Raster,
    widget::Widget,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    let defs = Definition::load_all(&mut cache)?;
    let mut models = Models::default();
    let mut bodies = 0;
    let mut heads = 0;
    let mut faces = 0;
    let mut ids = defs.keys().copied().collect::<Vec<_>>();
    ids.sort();
    let mut shown = Vec::new();
    for &id in &ids {
        let d = &defs[&id];
        for head in [false, true] {
            if let Some(mut m) = npc_model::mesh(&mut cache, &mut models, d, head, None)? {
                if head {
                    heads += 1;
                    if shown.len() < 8 && !m.faces.is_empty() && d.transforms.is_none() {
                        shown.push(id);
                    }
                } else {
                    bodies += 1;
                }
                let _ = m.light(
                    if head { 64 } else { d.ambient + 64 },
                    if head { 768 } else { d.contrast + 850 },
                    if head {
                        [-50, -10, -50]
                    } else {
                        [-30, -50, -30]
                    },
                )?;
                if !head {
                    npc_model::resize(&mut m, d);
                }
                faces += m.faces.len();
            }
        }
    }
    println!("NPC meshes: {bodies} bodies, {heads} heads, {faces} faces");
    let items = Definitions::load(&mut cache)?;
    let mut state = Interfaces::default();
    state.install_group(
        0,
        shown
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let mut w = Widget::empty(i as u32, 6);
                w.raw_position = [i as i32 % 4 * 190, i as i32 / 4 * 240];
                w.raw_size = [190, 240];
                w.model_angles = [0, 192, 0];
                w.model_zoom = 650;
                w.model_offset = [0, 90];
                w
            })
            .collect(),
    )?;
    state.root = Some(0);
    for (i, &id) in shown.iter().enumerate() {
        state.runtime.get_mut(&(i as u32)).unwrap().model = ModelSource::Npc(id as i32);
    }
    let mut raster = Raster::new(760, 480, 1 << 20)?;
    raster.clear(0xff302b25);
    let layout = state.layout([760, 480])?;
    let mut renderer = Renderer::load(&mut cache, 0.8)?;
    let report = renderer.draw(&mut cache, &items, &state, &layout, &mut raster)?;
    assert!(report.pending.is_empty());
    assert_eq!(report.drawn, shown.len());
    std::fs::write(&a[2], raster.bmp()?)?;
    println!("NPC chatheads {shown:?}: {report:?}");
    Ok(())
}
