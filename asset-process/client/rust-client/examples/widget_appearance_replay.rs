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
        .and_then(|p| p.appearance.as_ref().map(|a| gielinor_client::widget_appearance::pose(a.animations[0], &p.pose.state)));
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
    widget.model_angles = [150, 64, 0];
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
    c.interface(Update::PlayerObject {
        widget: 0,
        item: 4151,
    })?;
    let equipped = draw(&mut c, &mut renderer)?;
    assert_ne!(initial, equipped, "packet item equip changes body raster");
    assert_eq!(
        c.player_state
            .as_ref()
            .unwrap()
            .local_actor()
            .unwrap()
            .appearance
            .as_ref()
            .unwrap()
            .appearance
            .equipment,
        base.equipment
    );
    script(&mut c, 1207, &[1], true)?;
    assert_eq!(
        initial,
        draw(&mut c, &mut renderer)?,
        "script recopy discards widget edits"
    );
    script(&mut c, 1208, &[4151], true)?;
    assert_eq!(
        equipped,
        draw(&mut c, &mut renderer)?,
        "packet/script item equivalence"
    );
    c.interface(Update::PlayerColor {
        widget: 0,
        index: 1,
        color: 1,
    })?;
    let recolored = draw(&mut c, &mut renderer)?;
    assert_ne!(equipped, recolored);
    script(&mut c, 1209, &[1, 0], false)?;
    assert_eq!(equipped, draw(&mut c, &mut renderer)?);
    script(&mut c, 1209, &[1, i32::MAX], true)?;
    assert_eq!(
        c.interfaces.runtime[&0]
            .player_appearance
            .as_ref()
            .unwrap()
            .colors[1],
        i32::MAX
    );
    draw(&mut c, &mut renderer)?;
    script(&mut c, 1209, &[1, 0], false)?;
    c.interface(Update::PlayerBody { widget: 0, body: 1 })?;
    let female = draw(&mut c, &mut renderer)?;
    assert_ne!(equipped, female);
    script(&mut c, 1210, &[0], true)?;
    assert_eq!(
        equipped,
        draw(&mut c, &mut renderer)?,
        "body return restores original kits and keeps weapon"
    );
    let p = c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();
    p.appearance.as_mut().unwrap().appearance.equipment[3] = 4151 + 2048;
    script(&mut c, 1207, &[0], false)?;
    assert_eq!(
        initial,
        draw(&mut c, &mut renderer)?,
        "copy_objects false uses base equipment"
    );
    script(&mut c, 1207, &[1], false)?;
    assert_eq!(
        equipped,
        draw(&mut c, &mut renderer)?,
        "copy_objects true keeps equipment"
    );
    let p = c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();
    p.appearance.as_mut().unwrap().animations[0] = 808;
    p.pose.state.id = 808;
    p.pose.state.frame = 4;
    let animated = draw(&mut c, &mut renderer)?;
    assert_ne!(equipped, animated, "idle definition with current pose frame animates model7");
    let p = c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();
    p.pose.state.id = 819;
    assert_eq!(animated, draw(&mut c, &mut renderer)?, "walking pose ID does not replace model7 idle definition");
    let p = c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();
    p.freezes.active = true;
    p.freezes.display_pose = Some(gielinor_client::actor_sequence::State { id: 808, frame: 0, ..Default::default() });
    assert_eq!(animated, draw(&mut c, &mut renderer)?, "model7 ignores frozen pose snapshot");
    c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap().freezes = Default::default();
    let p = c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();
    p.pose.state.reset();
    p.appearance.as_mut().unwrap().appearance.npc = Some(1);
    script(&mut c, 1207, &[1], false)?;
    assert_ne!(
        equipped,
        draw(&mut c, &mut renderer)?,
        "NPC transformation body preview"
    );
    // Original opcode fixture runs through ClientState's actual script host.
    let mut vectors = gielinor_client::buffer::Reader::new(include_bytes!(
        "../tests/fixtures/widget-model-source-jvm.bin"
    ));
    for case in 0..vectors.u32()? {
        let opcode = vectors.u32()? as u16;
        let id = vectors.u32()? as i32;
        let dot = vectors.u8()? != 0;
        assert_eq!(vectors.u32()?, 1);
        assert_eq!(vectors.u32()?, 0);
        let kind = vectors.u32()?;
        let expected_id = vectors.u32()? as i32;
        let mut stack = Stacks::default();
        stack.push_int(id)?;
        c.extended(opcode, dot, &mut stack)?;
        assert!(stack.pop_int().is_err());
        let actual = match c.interfaces.runtime[&0].model {
            ModelSource::Npc(id) => (2, id),
            ModelSource::NpcBody(id) => (9, id),
            ModelSource::Object(id) => (8, id),
            _ => panic!("model source case {case}"),
        };
        assert_eq!(actual, (kind, expected_id), "model source case {case}");
    }
    assert_eq!(vectors.remaining(), 0);
    let scaled = c
        .npcs
        .values()
        .filter(|n| {
            n.id <= u16::MAX as u32
                && n.scale != [128, 128]
                && n.transforms.is_none()
                && !n.models.is_empty()
        })
        .map(|n| n.id)
        .min()
        .unwrap();
    for npc in [1, scaled] {
        let p = c.player_state.as_mut().unwrap().root_actor_mut(1).unwrap();
        p.appearance.as_mut().unwrap().appearance.npc = Some(npc as u16);
        p.pose.state.id = 808;
        p.pose.state.frame = 4;
        script(&mut c, 1207, &[1], false)?;
        let reference = draw(&mut c, &mut renderer)?;
        script(&mut c, 1215, &[npc as i32], true)?;
        c.interfaces.widgets.get_mut(&0).unwrap().sequences = [808; 2];
        c.interfaces.runtime.get_mut(&0).unwrap().frame = 4;
        assert_eq!(
            reference,
            draw(&mut c, &mut renderer)?,
            "NPC body/local transformed body {npc}"
        );
    }
    c.player_state = None;
    draw(&mut c, &mut renderer)?; // NPC model9 does not require a local player.
    script(&mut c, 1215, &[-1], false)?;
    assert_eq!(c.interfaces.runtime[&0].model, ModelSource::NpcBody(-1));
    assert!(c.outbound.is_empty());
    println!("Widget appearance: packet/script snapshots, base/equipment copies, independent edits, signed colors, body restore, local pose,4096 original model selectors and scaled/animated NPC body rasters passed");
    Ok(())
}
