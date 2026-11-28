//! Real-cache editor dispatch and body preview; no server/account mutation.
use gielinor_client::{
    client_state::ClientState, context_menu::Action, disk_cache::DiskCache,
    interface_render::Renderer, raster::Raster, widget::Widget, Result,
};
fn draw(c: &mut ClientState, renderer: &mut Renderer) -> Result<Vec<u32>> {
    renderer.designer = c.appearance_editor.clone();
    renderer.cycle = c.cycle;
    let layout = c.interfaces.layout([320, 360])?;
    let mut raster = Raster::new(320, 360, 320 * 360)?;
    raster.clear(0xff302b25);
    let report = renderer.draw(
        &mut c.scripts.source.cache,
        &c.items,
        &c.interfaces,
        &layout,
        &mut raster,
    )?;
    assert!(report.pending.is_empty(), "{report:?}");
    assert_eq!(report.drawn, 1);
    std::fs::write("work/appearance-editor.bmp", raster.bmp()?).unwrap();
    Ok(raster.pixels().to_vec())
}
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let mut preview = Widget::empty(0, 6);
    preview.content = 327;
    preview.raw_size = [320, 360];
    preview.model_zoom = 700;
    let mut button = Widget::empty(1, 3);
    button.flags = 2;
    button.content = 325;
    c.interfaces.install_group(0, vec![preview, button])?;
    c.interfaces.root = Some(0);
    let mut renderer = Renderer::load(&mut c.scripts.source.cache, 0.8)?;
    let initial = draw(&mut c, &mut renderer)?;
    assert!(!c.interface_operation(1, 1, None)?);
    assert_eq!(c.appearance_editor.as_ref().unwrap().gender, 1);
    let female = draw(&mut c, &mut renderer)?;
    let changed = initial.iter().zip(&female).filter(|(a, b)| a != b).count();
    assert!(changed > 100, "gender preview did not change");
    c.interfaces.widgets.get_mut(&1).unwrap().content = 324;
    c.menu_action(Action::Button { id: 1, kind: 1 })?;
    assert_eq!(initial, draw(&mut c, &mut renderer)?, "gender round trip");
    c.interfaces.widgets.get_mut(&1).unwrap().content = 317;
    c.menu_action(Action::Button { id: 1, kind: 1 })?;
    assert_ne!(initial, draw(&mut c, &mut renderer)?, "color preview");
    c.interfaces.widgets.get_mut(&1).unwrap().content = 316;
    assert!(!c.interface_operation(1, 1, None)?);
    assert_eq!(initial, draw(&mut c, &mut renderer)?, "color round trip");
    c.cycle = 40;
    assert_ne!(
        initial,
        draw(&mut c, &mut renderer)?,
        "simulation cycle preview rotation"
    );
    for content in 300..=325 {
        c.interfaces.widgets.get_mut(&1).unwrap().content = content;
        assert!(!c.interface_operation(1, 1, None)?);
        c.menu_action(Action::Button { id: 1, kind: 1 })?;
    }
    assert!(
        c.outbound.is_empty(),
        "editor emitted generic widget packets"
    );
    c.interfaces.widgets.get_mut(&1).unwrap().content = 326;
    let before = c.appearance_editor.as_ref().unwrap().design_bytes();
    assert!(c.interface_operation(1, 1, None).is_err());
    assert!(c.menu_action(Action::Button { id: 1, kind: 1 }).is_err());
    assert_eq!(before, c.appearance_editor.as_ref().unwrap().design_bytes());
    assert!(c.outbound.is_empty());
    c.prepare_login()?;
    assert!(c.appearance_editor.is_none());
    // ga.aa caches the first button pair even when its rectangle is clipped out.
    let ids = c
        .scripts
        .source
        .cache
        .reference(8)?
        .groups
        .iter()
        .take(2)
        .map(|g| g.id as i32)
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);
    let mut clipped = Widget::empty(0, 5);
    clipped.content = 324;
    clipped.sprites = [ids[0], ids[1]];
    clipped.raw_position = [-64, 0];
    clipped.raw_size = [32, 32];
    let mut visible = Widget::empty(1, 5);
    visible.content = 324;
    visible.sprites = [ids[1], ids[0]];
    visible.raw_size = [64, 64];
    c.interfaces.install_group(0, vec![clipped, visible])?;
    c.interfaces.root = Some(0);
    let mut renderer = Renderer::load(&mut c.scripts.source.cache, 0.8)?;
    let selected = draw(&mut c, &mut renderer)?;
    c.interfaces.widgets.get_mut(&1).unwrap().content = 0;
    c.interfaces.widgets.get_mut(&1).unwrap().sprites[0] = ids[1];
    assert_eq!(
        selected,
        draw(&mut c, &mut renderer)?,
        "clipped first sprite cache"
    );
    c.interfaces.widgets.get_mut(&1).unwrap().content = 324;
    c.interfaces.widgets.get_mut(&1).unwrap().if3 = false;
    c.interfaces.widgets.get_mut(&1).unwrap().sprites[1] = ids[0];
    let legacy = draw(&mut c, &mut renderer)?;
    c.interfaces.widgets.get_mut(&1).unwrap().content = 0;
    assert_eq!(
        legacy,
        draw(&mut c, &mut renderer)?,
        "legacy false condition keeps secondary sprite"
    );
    assert_ne!(selected, legacy, "distinct cache sprite fixtures");
    println!("Editor: all26 controls through both dispatch paths, no outbound packets, gender {changed} pixels, exact gender/color restore, animated yaw, submit limitation and login reset passed");
    Ok(())
}
