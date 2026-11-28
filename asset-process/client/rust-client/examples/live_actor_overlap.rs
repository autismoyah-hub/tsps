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

    let mut chosen=None;
    'scan:for y in (24..480).step_by(8) {for x in (24..740).step_by(8) {
        client.input.pointer=[x,y];
        let menu=live.context_menu(&mut client,[x,y])?;
        let indices=menu.iter().filter_map(|e|if let Action::Npc{index,..}=e.action {Some(index)}else{None}).collect::<std::collections::HashSet<_>>();
        if indices.len()!=1 {continue;}
        for e in menu {
            if let Action::Npc{index,..}=e.action {
                if client.npc_state.npcs.get(&index).and_then(|n|client.npcs.get(&(n.id as u32))).is_some_and(|d|d.size==1) {
                    chosen=Some(([x,y],index));break 'scan;
                }
            }
        }
    }}
    let (point,index)=chosen.expect("visible root size-one NPC menu");client.input.pointer=point;
    let extra=60000;
    assert!(!client.npc_state.npcs.contains_key(&extra));
    let source=client.npc_state.npcs.get_mut(&index).unwrap();
    source.path.position=source.path.position.map(|v|(v&!127)+64);
    let center=source.path.position;let definition_id=source.id;
    let mut neighbor=source.clone();neighbor.appearance.name="Overlap fixture".into();
    client.npc_state.npcs.insert(extra,neighbor);client.npc_state.indices.push(extra);
    let has_neighbor=|client:&mut ClientState,live:&mut Connection|->gielinor_client::Result<bool>{
        Ok(live.context_menu(client,point)?.iter().any(|e|matches!(e.action,Action::Npc{index,..}|Action::ExamineNpc(index) if index==extra)))
    };
    assert!(has_neighbor(&mut client,&mut live)?,"same-center NPC absent from model picks must contribute menu");
    client.npc_state.npcs.get_mut(&index).unwrap().path.position[0]+=1;
    assert!(!has_neighbor(&mut client,&mut live)?,"off-center picked NPC must not expand");
    client.npc_state.npcs.get_mut(&index).unwrap().path.position=center;
    client.npc_state.npcs.get_mut(&extra).unwrap().path.position[1]+=128;
    assert!(!has_neighbor(&mut client,&mut live)?,"different-center neighbor must not expand");
    client.npc_state.npcs.get_mut(&extra).unwrap().path.position=center;
    client.npcs.get_mut(&(definition_id as u32)).unwrap().size=2;
    assert!(!has_neighbor(&mut client,&mut live)?,"large picked NPC must not expand");
    println!("4 live overlap menu cases using an unrendered local NPC at {point:?}");
    Ok(())
}
