//! Real melee packet/synchronization/render acceptance on Agent 1's isolated server.
use gielinor_client::{client_state::ClientState,disk_cache::DiskCache,live_runtime::Connection,context_menu::Action};
use std::{io::{BufRead,BufReader,Write},net::TcpStream,time::{Duration,Instant},collections::HashSet};
fn teleport(tile: [i32; 2]) -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = TcpStream::connect("127.0.0.1:43618")?;
    socket.set_read_timeout(Some(Duration::from_secs(5)))?;
    writeln!(socket, "cmd rustclient239 tele 0 {} {} {} {}", tile[0]>>6, tile[1]>>6, tile[0]&63, tile[1]&63)?;
    let mut reply=String::new(); BufReader::new(socket).read_line(&mut reply)?;
    if !reply.starts_with("OK") { return Err(format!("isolated teleport rejected: {reply}").into()); }
    Ok(())
}
fn step(client:&mut ClientState,live:&mut Connection,next:&mut Instant)->Result<(),Box<dyn std::error::Error>> {
    live.poll(client)?;
    while Instant::now()>=*next {
        client.tick_actor_sequences()?;
        if let (Some((scene,_)),Some((target,plane)))=(&live.world,client.camera_focus()) {
            client.camera.tick_follow(scene,target,plane,client.preferences.camera_follow_height);
            client.camera.tick_locked(scene,plane);
        }
        *next+=Duration::from_millis(20);
    }
    Ok(())
}
fn run()->Result<(),Box<dyn std::error::Error>> {
    let mut client=ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5",64<<20)?)?;
    let mut live=Connection::connect(&mut client,"127.0.0.1:43617","rustclient239",
        "work/kotlin-world/test-account.password","work/kotlin-world/.data/client.key")?;
    client.canvas=[765,503];client.camera.pitch=2048;
    let mut next=Instant::now();let deadline=Instant::now()+Duration::from_secs(90);
    while !live.session.is_established() || live.world.is_none() || live.scripts_busy() {
        step(&mut client,&mut live,&mut next)?;assert!(Instant::now()<deadline,"login timeout");
        std::thread::sleep(Duration::from_millis(20));
    }
    teleport([3233,3230])?;
    let deadline=Instant::now()+Duration::from_secs(45);
    loop {
        step(&mut client,&mut live,&mut next)?;
        if client.data.position[1..]==[3233,3230] && live.world.as_ref().is_some_and(|(s,_)|s.base_x<3300) && !live.scripts_busy() { break; }
        assert!(Instant::now()<deadline,"combat region timeout");std::thread::sleep(Duration::from_millis(20));
    }
    let (index,operation,name,tile)=client.npc_state.npcs.iter().filter_map(|(&index,n)| {
        let d=client.npcs.get(&(n.id as u32))?;
        if !(1..=5).contains(&d.combat) {return None;}
        let op=d.operations.iter().find(|o|o.sub.is_none()&&o.condition.is_none()&&o.text.eq_ignore_ascii_case("Attack"))?;
        Some((index,op.index+1,d.name.clone(),n.position))
    }).min_by_key(|(_,_,_,p)|(p[0]-3233).abs()+(p[1]-3230).abs()).ok_or("no low-level attackable NPC in loaded region")?;
    let scene=&live.world.as_ref().unwrap().0;
    let local=[tile[0]-scene.base_x,tile[1]-scene.base_y];
    let direction=[[1,0],[-1,0],[0,1],[0,-1]].into_iter()
        .find(|&d|gielinor_client::route::can_step(&scene.collision[0],local,1,d)).ok_or("no open adjacent target tile")?;
    let stand=[tile[0]+direction[0],tile[1]+direction[1]];teleport(stand)?;
    let deadline=Instant::now()+Duration::from_secs(20);
    while client.data.position[1..]!=stand || live.scripts_busy() {
        step(&mut client,&mut live,&mut next)?;assert!(Instant::now()<deadline,"adjacent teleport timeout");
        std::thread::sleep(Duration::from_millis(20));
    }
    client.game_messages.clear();
    let before=client.outbound.len();
    client.menu_action(Action::Npc{index,operation,sub:0})?;
    assert!(client.outbound.len()>before,"Attack action emitted no packet");
    println!("Attack {name} index {index}, operation {operation}, NPC tile {tile:?}, player tile {stand:?}; queued opcodes {:?}",client.outbound.iter().map(|p|p.opcode).collect::<Vec<_>>());
    let started=Instant::now();let mut hits=HashSet::new();let mut animations=HashSet::new();let mut bars=HashSet::new();let mut frames=0;let mut npc_damage=0;let mut player_damage=0;let mut removed=false;
    while started.elapsed()<Duration::from_secs(35) {
        step(&mut client,&mut live,&mut next)?;
        if let Some(p)=client.player_state.as_ref().and_then(|s|s.local_actor()) {
            if p.sequence.id>=0 {animations.insert(p.sequence.id);}
            for h in &p.overlays.hits.slots {
                if hits.insert((false,h.kind,h.value,h.end)) {player_damage+=h.value.max(0);println!("Player hit kind {} value {}",h.kind,h.value);}
            }
        }
        if let Some(n)=client.npc_state.npcs.get(&index) {
            for h in &n.overlays.hits.slots {
                if hits.insert((true,h.kind,h.value,h.end)) {npc_damage+=h.value.max(0);println!("NPC hit kind {} value {}",h.kind,h.value);}
            }
            for b in &n.overlays.bars.entries {for u in &b.updates {bars.insert((b.definition.id,u.start,u.fill));}}
        } else {removed=true;}
        if live.world.is_some() {
            let raster=live.paint(&mut client,765,503)?;frames+=1;
            if frames==10 || frames%50==0 {std::fs::write("work/live-melee-combat.bmp",raster.bmp()?)?;}
        }
        if (started.elapsed()>Duration::from_secs(15) || removed) && npc_damage>0 && hits.iter().filter(|h|h.0).count()>=2 && !animations.is_empty() && !bars.is_empty() {break;}
        std::thread::sleep(Duration::from_millis(20));
    }
    let npc_hits=hits.iter().filter(|h|h.0).count();
    println!("Combat observations: {frames} frames, {npc_hits} NPC hits, NPC damage {npc_damage}, player damage {player_damage}, {} health updates, animations {animations:?}, removed {removed}, {:?}",bars.len(),started.elapsed());
    for message in &client.game_messages {println!("Combat message: {}",message.text);}
    if npc_hits<2 || npc_damage==0 || animations.is_empty() || bars.is_empty() {return Err("melee acceptance lacked confirmed hit/health/animation evidence".into());}
    teleport([10422,123])?;live.session.close();println!("Real melee acceptance passed; dedicated account returned to hub");Ok(())
}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let result=std::panic::catch_unwind(run);let restore=teleport([10422,123]);
    match result {Ok(result)=>result.and(restore),Err(panic)=>std::panic::resume_unwind(panic)}
}
