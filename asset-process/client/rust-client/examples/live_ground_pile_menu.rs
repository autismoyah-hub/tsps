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

    install(&mut client,0,true,false)?;
    let item=|id|gielinor_client::zone::Item{id,quantity:1,operations:31,visible:0,despawn:i32::MAX,ownership:0,private:false,custom:None};
    let mut chosen=None;
    'scan: for dz in -3..=3 {for dx in -3..=3 {
        let tile=[client.data.position[1]+dx,client.data.position[2]+dz];
        let key=[client.data.position[0],tile[0],tile[1]];
        client.zone.piles.clear();client.zone.piles.insert(key,vec![item(1277)]);
        live.paint(&mut client,765,503)?;
        if let Some((point,_))=live.pick_where_view(|view,tag|view==0&&tag&0x1000000000000000!=0&&((tag>>32)as u16)==1277) {chosen=Some((tile,key,point));break 'scan;}
    }}
    let (tile,key,point)=chosen.expect("local ground item model pick");
    client.input.pointer=point;
    let ids=vec![1277,1279,1205,556,558,1277];
    client.zone.piles.insert(key,ids.iter().copied().map(item).collect());
    let menu=live.context_menu(&mut client,point)?;
    let actual=menu.iter().filter_map(|e|match e.action {Action::Examine{kind:2,id,tile:at} if at==tile.map(|v|v as u16)=>Some(id),_=>None}).collect::<Vec<_>>();
    assert_eq!(actual,ids.iter().rev().copied().collect::<Vec<_>>(),"complete pile includes unrendered and duplicate item IDs");
    client.hidden_operations[1]=true;
    let hidden=live.context_menu(&mut client,point)?;
    assert!(!hidden.iter().any(|e|matches!(e.action,Action::Ground{..})));
    assert_eq!(hidden.iter().filter(|e|matches!(e.action,Action::Examine{kind:2,..})).count(),ids.len());
    client.hidden_operations[1]=false;
    client.selected_spell=Some(gielinor_client::spell_action::Selection{widget:0,logical:0,child:-1,item:-1,mask:1,option:"Cast".into(),target:"Spell".into()});
    let actual=live.context_menu(&mut client,point)?.into_iter().filter_map(|e|match e.action{Action::CastWorld{kind:3,id,..}=>Some(id),_=>None}).collect::<Vec<_>>();
    assert_eq!(actual,ids.iter().rev().copied().collect::<Vec<_>>(),"spell targets include every pile entry");
    client.selected_spell.as_mut().unwrap().mask=0;
    assert!(!live.context_menu(&mut client,point)?.iter().any(|e|matches!(e.action,Action::CastWorld{kind:3,..})));
    println!("4 live ground-pile cases including unrendered/duplicate items and spell targeting at {point:?}");
    Ok(())
}
