//! Exercise widget queries on real login interfaces, with no synthetic server updates.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    live_runtime::Connection,
};
use std::time::{Duration, Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let mut live = Connection::connect(
        &mut client,
        "127.0.0.1:43617",
        "rustclient239",
        "work/kotlin-world/test-account.password",
        "work/kotlin-world/.data/client.key",
    )?;
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        live.poll(&mut client)?;
        if live.world.is_some()
            && live.session.is_established()
            && !live.scripts_busy()
            && client.interfaces.scripts.is_empty()
            && !client.interfaces.children.is_empty()
            && client
                .interfaces
                .widgets
                .values()
                .any(|w| w.actions.iter().any(|action| action == "Inventory"))
        {
            break;
        }
        assert!(Instant::now() < deadline, "login/interface timeout");
        std::thread::sleep(Duration::from_millis(20));
    }
    client.canvas = [765, 503];
    if let Some((target, plane, height)) = live.camera_follow_target(&client) {
        client.camera.tick_follow(
            &live.world.as_ref().unwrap().0,
            target,
            plane,
            client.preferences.camera_follow_height,
        );
        client
            .camera
            .set_target_height(height, client.preferences.camera_follow_height);
    }
    client.camera.pitch = 2048;
    client.camera.yaw = 0;
    live.paint(&mut client, 765, 503)?;

    use gielinor_client::{widget::Widget,context_menu::Action};
    let group=65533;let base=(group as u32)<<16;
    let install=|client:&mut ClientState,count:u32,before:bool,barrier:bool|->gielinor_client::Result<()> {
        let mut widgets=Vec::new();
        for i in 0..count {let mut w=Widget::empty(base+if before{i}else{i+1},4);w.button=1;w.button_text="Seed".into();w.raw_size=[765,503];widgets.push(w);}
        let mut viewport=Widget::empty(base+if before{count}else{0},4);viewport.content=1337;viewport.raw_size=[765,503];viewport.no_click_through=barrier;widgets.push(viewport);
        client.interfaces=Default::default();client.interfaces.install_group(group,widgets)?;client.interfaces.root=Some(group);Ok(())
    };
    install(&mut client,0,true,false)?;live.paint(&mut client,765,503)?;
    let is_actor=|action:Action|matches!(action.scene().1,Action::Npc{..}|Action::ExamineNpc(_)|Action::Player{..});
    let mut chosen=None;
    'scan:for y in (24..480).step_by(8) {for x in (24..740).step_by(8) {
        client.input.pointer=[x,y];
        if live.context_menu(&mut client,[x,y])?.iter().any(|e|is_actor(e.action)) {chosen=Some([x,y]);break 'scan;}
    }}
    let point=chosen.expect("visible native actor menu");client.input.pointer=point;
    for (count,before,barrier,expected) in [(0,true,false,true),(399,true,false,false),(450,true,false,false),(399,false,false,true),(399,true,true,true)] {
        install(&mut client,count,before,barrier)?;
        let menu=live.context_menu(&mut client,point)?;
        assert_eq!(menu.iter().any(|e|is_actor(e.action)),expected,"count {count} before {before} barrier {barrier}");
        assert!(menu.len()<=499);
    }
    println!("5 live actor-menu admission cases including earlier/later UI and viewport barrier at {point:?}");
    Ok(())
}
