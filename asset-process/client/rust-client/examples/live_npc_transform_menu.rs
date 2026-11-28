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

    let original=client.npc_state.npcs[&index].id as u32;
    let template=client.npcs[&original].clone();
    for id in 60000..=60002 {assert!(!client.npcs.contains_key(&id));let mut definition=template.clone();definition.id=id;definition.name=format!("Variant{id}");definition.combat=0;definition.transforms=None;definition.transform_varbit=None;definition.transform_varp=None;definition.operations=vec![gielinor_client::object::Operation{index:0,sub:None,condition:None,text:format!("Talk{id}")}];client.npcs.insert(id,definition);}
    let base=client.npcs.get_mut(&60000).unwrap();base.transforms=Some(vec![Some(60001),Some(60002),None]);base.transform_varp=Some(0);
    let npc=client.npc_state.npcs.get_mut(&index).unwrap();npc.id=60000;npc.appearance.name.clear();npc.appearance.operations=31;npc.appearance.combat=-1;
    client.scripts.variables.set_player(0,0);live.paint(&mut client,765,503)?;
    for (value,expected) in [(0,Some(60001)),(1,Some(60002)),(2,None),(0,Some(60001))] {
        client.scripts.variables.set_player(0,value);
        let menu=live.context_menu(&mut client,point)?;
        let rows=menu.iter().filter(|e|matches!(e.action,Action::Npc{index:got,..}|Action::ExamineNpc(got) if got==index)).collect::<Vec<_>>();
        if let Some(id)=expected {assert!(!rows.is_empty());assert!(rows.iter().all(|e|e.target.contains(&format!("Variant{id}"))),"varp {value} retains stale rendered definition: {:?}",rows.iter().map(|e|&e.target).collect::<Vec<_>>());}
        else {assert!(rows.is_empty(),"null transform must remove menu immediately");}
    }
    println!("4 live NPC transform menu transitions without repaint at {point:?}");
    Ok(())
}
