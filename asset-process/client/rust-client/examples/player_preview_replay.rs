//! Packet/script ownership and real-cache model-7 previews.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_packet::Update,
    interface_render::Renderer,
    interface_state::ModelSource,
    player_info,
    player_model::Appearance,
    raster::Raster,
    vm::{Host, Stacks},
    widget::Widget,
    Result,
};
fn script(c: &mut ClientState, opcode: u16, args: &[i32], explicit: bool) -> Result<()> {
    let mut stack = Stacks::default();
    for &v in args {
        stack.push_int(v)?;
    }
    if explicit {
        stack.push_int(0)?;
    }
    c.extended(opcode + if explicit { 1000 } else { 0 }, false, &mut stack)?;
    assert!(stack.pop_int().is_err(), "unconsumed script arguments");
    Ok(())
}
fn draw(c: &mut ClientState, renderer: &mut Renderer) -> Result<Vec<u32>> {
    renderer.preview = c
        .player_state
        .as_ref()
        .and_then(|s| s.local_actor())
        .and_then(|p| gielinor_client::player_preview::Snapshot::capture(p, 50, Some(0)));
    renderer.player = c
        .player_state
        .as_ref()
        .and_then(|s| s.local_actor())
        .and_then(|p| p.appearance.as_ref())
        .map(|a| a.appearance.clone());
    renderer.player_pose = c
        .player_state
        .as_ref()
        .and_then(|s| s.local_actor())
        .and_then(|p| {
            p.appearance
                .as_ref()
                .map(|a| gielinor_client::widget_appearance::pose(a.animations[0], &p.pose.state))
        });
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
    assert_eq!(
        report.drawn,
        usize::from(renderer.preview.as_ref().is_some_and(|p| p.alpha != 255))
    );
    std::fs::write("work/widget-appearance.bmp", raster.bmp()?).unwrap();
    Ok(raster.pixels().to_vec())
}
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let kits = gielinor_client::identity_kit::Kit::load_all(&mut c.scripts.source.cache)?;
    let base = Appearance::standard(&kits, 0);
    let mut widget = Widget::empty(0, 6);
    widget.raw_size = [320, 360];
    widget.model_zoom = 700;
    widget.model_angles = [150, 0, 0];
    // Model-7 uses the local pose, ignoring this deliberately invalid widget sequence.
    widget.sequences = [i32::MAX; 2];
    c.interfaces.install_group(0, vec![widget])?;
    c.interfaces.root = Some(0);
    c.active_widgets = [Some(0); 2];
    c.interface(Update::PlayerSelf {
        widget: 0,
        copy_objects: false,
    })?;
    assert_eq!(
        c.interfaces.runtime[&0].model,
        ModelSource::None,
        "packet skips missing local player"
    );
    assert!(
        script(&mut c, 1208, &[4151], false).is_err(),
        "mutation needs snapshot"
    );
    let (mut players, _) = player_info::State::initialize(&vec![0; 4608], 1)?;
    players.root_actor_mut(1).unwrap().appearance =
        Some(gielinor_client::player_appearance::Update {
            appearance: base.clone(),
            skull: -1,
            overhead: -1,
            animations: [-1; 7],
            name: "Preview".into(),
            combat: 3,
            skill: 0,
            hidden: false,
            force_refresh: false,
            actions: Default::default(),
            team: 0,
        });
    c.player_state = Some(players);
    c.interface(Update::PlayerSelf {
        widget: 0,
        copy_objects: true,
    })?;
    let mut renderer = Renderer::load(&mut c.scripts.source.cache, 0.8)?;
    let initial = draw(&mut c, &mut renderer)?;
    c.interfaces.widgets.get_mut(&0).unwrap().content = 328;
    assert_eq!(
        initial,
        draw(&mut c, &mut renderer)?,
        "content328 full body equals composition body"
    );
    let p = c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();
    p.appearance.as_mut().unwrap().animations[0] = 808;
    p.pose.state.id = 808;
    p.pose.state.frame = 4;
    let posed = draw(&mut c, &mut renderer)?;
    assert_ne!(initial, posed);
    c.interfaces.widgets.get_mut(&0).unwrap().content = 0;
    assert_eq!(
        posed,
        draw(&mut c, &mut renderer)?,
        "same local pose as model7"
    );
    c.interfaces.widgets.get_mut(&0).unwrap().content = 328;
    let p = c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();
    p.sequence.id = 808;
    p.sequence.frame = 4;
    p.appearance.as_mut().unwrap().animations[0] = 808;
    assert_eq!(
        posed,
        draw(&mut c, &mut renderer)?,
        "action suppresses duplicate idle"
    );
    let p = c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();
    p.freezes.active = true;
    p.freezes.display = Some(p.sequence.clone());
    p.freezes.display_pose = Some(p.pose.state.clone());
    p.sequence.frame = 0;
    p.pose.state.frame = 0;
    assert_eq!(
        posed,
        draw(&mut c, &mut renderer)?,
        "frozen snapshots survive current clock movement"
    );
    let p = c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();
    p.freezes = Default::default();
    p.sequence.reset();
    p.pose.state.reset();
    p.idle_fallback.state.id = 808;
    p.idle_fallback.state.frame = 4;
    assert_eq!(
        posed,
        draw(&mut c, &mut renderer)?,
        "fallback retains the idle frame when current clocks are absent"
    );
    c.player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .idle_fallback
        .state
        .loading = 1;
    assert!(
        draw(&mut c, &mut renderer)?
            .iter()
            .all(|&v| v == 0xff302b25),
        "loading fallback hides the actor during its grace period"
    );
    c.player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .idle_fallback
        .state
        .loading = 30;
    assert_eq!(
        posed,
        draw(&mut c, &mut renderer)?,
        "fallback grace period ends at thirty ticks"
    );
    let p = c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();
    p.idle_fallback = Default::default();
    p.visual.tint = gielinor_client::actor_visual::Tint {
        start: 0,
        end: 100,
        hsl: [25, 5, 60],
        weight: 100,
    };
    assert_ne!(initial, draw(&mut c, &mut renderer)?, "active actor tint");
    c.player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .visual
        .tint
        .end = 50;
    assert_eq!(
        initial,
        draw(&mut c, &mut renderer)?,
        "tint ends exclusively"
    );
    c.player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .visual
        .transparency
        .get_or_insert_with(Default::default)
        .set(0, 0, 1, 0, 255, true);
    let blank = draw(&mut c, &mut renderer)?;
    assert!(blank.iter().all(|&v| v == 0xff302b25));
    c.player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .visual
        .transparency
        .get_or_insert_with(Default::default)
        .set(0, 0, 1, 0, 0, true);
    let mut models = gielinor_client::item_model::Models::default();
    let model = base.mesh(
        &mut c.scripts.source.cache,
        &mut models,
        &kits,
        &c.items,
        false,
    )?;
    let colors = model.light(64, 850, [-30, -50, -30])?;
    let p = c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();
    p.attachment = Some(gielinor_client::location_merge::Attachment {
        placement: gielinor_client::location_merge::Placement {
            start: 50,
            end: 51,
            centre: [p.path.position[0] + 64, 0, p.path.position[1]],
            bounds: [0; 4],
        },
        model,
        colors,
    });
    assert_ne!(
        initial,
        draw(&mut c, &mut renderer)?,
        "active attachment shares body frame"
    );
    c.player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap()
        .attachment
        .as_mut()
        .unwrap()
        .placement
        .end = 50;
    assert_eq!(
        initial,
        draw(&mut c, &mut renderer)?,
        "attachment end restores body exactly"
    );
    let p = c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();
    p.appearance.as_mut().unwrap().appearance.npc = Some(1);
    let npc = draw(&mut c, &mut renderer)?;
    assert_ne!(npc, initial);
    c.interface(Update::PlayerSelf {
        widget: 0,
        copy_objects: true,
    })?;
    c.interfaces.widgets.get_mut(&0).unwrap().content = 0;
    assert_eq!(
        npc,
        draw(&mut c, &mut renderer)?,
        "NPC body transformation equals model7"
    );
    c.interfaces.widgets.get_mut(&0).unwrap().content = 328;
    c.player_state = None;
    assert_eq!(
        blank,
        draw(&mut c, &mut renderer)?,
        "null local actor is empty"
    );
    println!("Local preview: full body, poses, idle suppression, frozen clocks, tint, opacity, attachment boundaries, NPC transformation and null actor passed");
    Ok(())
}
