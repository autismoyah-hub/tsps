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

    use gielinor_client::{buffer::Reader,object::Operation,spell_action::Selection};
    let original=client.npc_state.npcs[&index].id as u32;
    let template=client.npcs[&original].clone();
    let mut r=Reader::new(include_bytes!("../tests/fixtures/npc-hidden-menu-jvm.bin"));
    for case in 0..r.u32()? {
        let hidden=r.u8()?!=0;let spell=r.u8()?!=0;let spell_mask=r.u8()?;
        assert_eq!(r.u32()?,0);let mode=r.u8()?;let level=r.u32()? as i32;let _local=r.u32()?;let combat_override=r.u32()? as i32;let mask=r.u8()?;
        let follower=r.u8()?!=0;let _mine=r.u8()?!=0;let low=r.u8()?!=0;let low_setting=r.u8()?!=0;
        let expected=(0..r.u32()?).map(|_|{let code=r.u32()?;let label=r.string()?;Ok((code,label))}).collect::<gielinor_client::Result<Vec<_>>>()?;
        let mut def=template.clone();def.transforms=None;def.transform_varbit=None;def.transform_varp=None;def.combat=level;def.follower=follower;def.low_priority=low;def.interactable=true;
        def.operations=(0..5).map(|i|Operation{index:i,sub:None,condition:None,text:if i==1||i==3{"Attack".into()}else{format!("Action{i}")}}).collect();client.npcs.insert(original,def);
        let npc=client.npc_state.npcs.get_mut(&index).unwrap();npc.appearance.operations=mask;npc.appearance.combat=combat_override;
        client.hidden_operations[2]=hidden;client.preferences.follower_ops_low_priority=low_setting;client.menu_settings.npc_attack=mode as i32;
        client.selected_spell=spell.then(||Selection{widget:0,logical:0,child:-1,item:-1,mask:spell_mask,option:"Cast".into(),target:"Spell".into()});
        let actual=live.context_menu(&mut client,point)?.into_iter().filter_map(|e|match e.action {
          Action::Npc{index:got,operation,..} if got==index=>Some((8+operation as u32+if low&&low_setting{2000}else{0},e.option)),
          Action::ExamineNpc(got) if got==index=>Some((1003,e.option)),
          Action::CastWorld{kind:1,id,..} if id==index=>Some((8,e.option)),_=>None}).collect::<Vec<_>>();
        // Native context menus apply priority sorting after the Java builder trace.
        let mut expected=expected;gielinor_client::menu_sort::prioritize(&mut expected,|e|e.0 as i32);
        assert_eq!(actual.iter().map(|e|e.0).collect::<Vec<_>>(),expected.iter().map(|e|e.0).collect::<Vec<_>>(),"NPC hidden case {case}");
    }
    assert_eq!(r.remaining(),0);println!("128 live NPC hidden-operation and spell cases at {point:?}");Ok(())
}
