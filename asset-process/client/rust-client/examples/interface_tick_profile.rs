//! Measure actual cached interface event work before changing its tick cadence.
use gielinor_client::{client_state::ClientState,disk_cache::DiskCache,live_runtime::Connection};
use std::time::{Duration,Instant};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let mut client=ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5",64<<20)?)?;
 let mut live=Connection::connect(&mut client,"127.0.0.1:43617","rustclient239","work/kotlin-world/test-account.password","work/kotlin-world/.data/client.key")?;
 client.canvas=[765,503];let deadline=Instant::now()+Duration::from_secs(90);
 loop {
  live.poll(&mut client)?;
  if !live.scripts_busy() && client.interfaces.scripts.is_empty() {
   client.tick_interfaces(false)?;
   let layout=client.interfaces.layout(client.canvas)?;
   if live.world.is_some() && layout.order.iter().any(|id|!layout.widgets[id].hidden && client.interfaces.widgets[id].actions.iter().any(|s|s=="Inventory")) {break;}
  }
  assert!(Instant::now()<deadline,"interface readiness timeout");std::thread::sleep(Duration::from_millis(20));
 }
 println!("Ready: {} widgets; {} timer listeners",client.interfaces.widgets.len(),client.interfaces.runtime.keys().filter(|&&id|client.interfaces.listener(id,1408).is_some()).count());
 let start=Instant::now();
 for _ in 0..20 {std::hint::black_box(client.interfaces.layout(client.canvas)?);}
 println!("Layout mean: {:.3} ms",start.elapsed().as_secs_f64()*50.0);
 let layout=client.interfaces.layout(client.canvas)?;
 let events=client.interface_events.collect(&client.interfaces,&layout,&client.input,client.preferences.mouse_camera,false);
 for event in events {
  let start=Instant::now();let instructions=client.run_listener(&event.arguments,&event.context)?;
  println!("Listener widget {} script {:?}: {:.3} ms, {instructions} instructions",event.widget,event.arguments.first(),start.elapsed().as_secs_f64()*1000.0);
 }
 for point in [[0,0],[642,186],[581,229]] {
  client.input.pointer=point;let mut times=Vec::new();let mut counts=Vec::new();
  for _ in 0..20 {
   client.cycle+=1;client.input.begin_cycle();let start=Instant::now();
   let instructions=client.tick_interfaces(false)?;
   times.push(start.elapsed().as_secs_f64()*1000.0);counts.push(instructions);
  }
  times.sort_by(f64::total_cmp);println!("Pointer {point:?}: median {:.3} ms, max {:.3} ms, instructions {}..{}",times[10],times[19],counts.iter().min().unwrap(),counts.iter().max().unwrap());
 }
 for error in &client.script_failures {println!("Interface failure: {error}");}
 assert!(client.script_failures.is_empty(),"interface profile script failures");
 live.session.close();Ok(())
}
