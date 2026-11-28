//! Object widget model8 against the existing object mesh/animation pipeline.
use gielinor_client::{
    animation_runtime::{Animations, Clock},
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_model,
    interface_render::Renderer,
    item_icon::Icons,
    object::Definition,
    object_model::Models,
    raster::Raster,
    vm::{Host, Stacks},
    widget::Widget,
    Result,
};
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let defs = Definition::load_all(&mut c.scripts.source.cache)?;
    let mut ids = Vec::new();
    for category in 0..5 {
        let id = defs
            .values()
            .filter(|d| category == 4 || !d.model_ids(10).is_empty())
            .filter(|d| match category {
                0 => true,
                1 => d.animation.is_some(),
                2 => d.contour >= 0,
                3 => d.scale != [128; 3],
                _ => d.transforms.is_some(),
            })
            .map(|d| d.id)
            .min()
            .unwrap_or_else(|| panic!("missing object category {category}"));
        ids.push(id);
    }
    ids.sort();
    ids.dedup();
    let mut renderer = Renderer::load(&mut c.scripts.source.cache, 0.8)?;
    let icons = Icons::load(&mut c.scripts.source.cache, 0.8)?;
    let mut models = Models::default();
    let mut animations = Animations::load(&mut c.scripts.source.cache)?;
    let mut widget = Widget::empty(0, 6);
    widget.raw_size = [320, 360];
    widget.model_zoom = 700;
    widget.model_angles = [150, 64, 0];
    c.interfaces.install_group(0, vec![widget])?;
    c.interfaces.root = Some(0);
    c.active_widgets = [Some(0); 2];
    let mut cases = 0;
    let mut animated_changes = 0;
    for id in &ids {
        let d = &defs[id];
        let mut stack = Stacks::default();
        stack.push_int(*id as i32)?;
        stack.push_int(0)?;
        c.extended(2214, false, &mut stack)?;
        assert!(stack.pop_int().is_err());
        let Some(base) = models.object(&mut c.scripts.source.cache, d, 10, 0)? else {
            assert!(d.transforms.is_some(), "unexpected empty base {id}");
            let mut raster = Raster::new(320, 360, 320 * 360)?;
            raster.clear(0xff302b25);
            let layout = c.interfaces.layout([320, 360])?;
            let report = renderer.draw(
                &mut c.scripts.source.cache,
                &c.items,
                &c.interfaces,
                &layout,
                &mut raster,
            )?;
            assert_eq!(report.drawn, 0);
            assert!(report.pending.is_empty());
            assert!(raster.pixels().iter().all(|&p| p == 0xff302b25));
            println!("Morph-capable object {id}: empty base remains empty in widget preview");
            cases += 1;
            continue;
        };
        let colors = base.light(64 + d.ambient as i32, 768 + d.contrast, [-50, -10, -50])?;
        let mut variants = vec![(-1, 0)];
        if let Some(sequence) = d.animation {
            let s = &animations.sequences[&(sequence as u32)];
            let count = if s.cached_animation.is_some() {
                s.total_length as usize
            } else {
                s.frames.len()
            };
            variants.push((sequence as i32, 0));
            variants.push((sequence as i32, count.saturating_sub(1).min(4)));
        }
        let mut first = None;
        for (sequence, frame) in variants {
            c.interfaces.widgets.get_mut(&0).unwrap().sequences = [sequence; 2];
            c.interfaces.runtime.get_mut(&0).unwrap().frame = frame;
            let mut model = (*base).clone();
            if sequence >= 0 {
                animations.apply(
                    &mut c.scripts.source.cache,
                    &mut model,
                    sequence as u32,
                    &Clock {
                        frame,
                        ..Default::default()
                    },
                )?;
            }
            let vertices = interface_model::vertices(&model, [150, 64, 0], [0; 2], 700, false)?;
            let sprite = icons.draw_model(&model, &colors, &vertices, [320, 360], None)?;
            let mut expected = Raster::new(320, 360, 320 * 360)?;
            expected.clear(0xff302b25);
            sprite.draw(&mut expected, 0, 0, 256)?;
            let mut actual = Raster::new(320, 360, 320 * 360)?;
            actual.clear(0xff302b25);
            let layout = c.interfaces.layout([320, 360])?;
            let report = renderer.draw(
                &mut c.scripts.source.cache,
                &c.items,
                &c.interfaces,
                &layout,
                &mut actual,
            )?;
            assert_eq!(report.drawn, 1);
            assert!(report.pending.is_empty(), "{report:?}");
            assert_eq!(
                actual.pixels(),
                expected.pixels(),
                "object {id} sequence {sequence} frame {frame}"
            );
            if let Some(before) = &first {
                if before != actual.pixels() {
                    animated_changes += 1;
                }
            } else {
                first = Some(actual.pixels().to_vec());
            }
            std::fs::write("work/widget-object.bmp", actual.bmp()?).unwrap();
            cases += 1;
        }
    }
    assert!(animated_changes > 0, "animated object stayed static");
    assert!(c.outbound.is_empty());
    println!("Object widget: {cases} static/animated raster comparisons across IDs {ids:?}, including scaled, contoured and morph-capable definitions; {animated_changes} animation changes");
    Ok(())
}
