//! Exercise the actual ClientState batch, including error restoration.
use gielinor_client::{client_state::ClientState,disk_cache::DiskCache,input::{Input,Update}};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let mut client=ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5",64<<20)?)?;
 client.cycle=100;client.logout_timer=20;client.data.reboot_timer=20;
 client.input.apply(Update::Pointer([90,80]));
 client.input.apply(Update::Character(120));
 let mut pending=Input::default();pending.apply(Update::Key(25,true));pending.apply(Update::Character(119));
 client.tick_interfaces_elapsed(0,&mut pending,false)?;
 assert_eq!(pending.key_events,[(25,0),(-1,119)]);
 assert_eq!(client.logout_timer,20);
 client.tick_interfaces_elapsed(5,&mut pending,false)?;
 assert_eq!(client.cycle,100);assert_eq!(client.logout_timer,15);assert_eq!(client.data.reboot_timer,15);
 assert_eq!(client.input.pointer,[90,80]);assert_eq!(client.input.text,[120]);
 assert!(pending.key_events.is_empty());assert!(pending.held.contains(&25));
 pending.apply(Update::Character(122));client.canvas=[-1,503];
 assert!(client.tick_interfaces_elapsed(3,&mut pending,false).is_err());
 assert_eq!(client.cycle,100);assert_eq!(client.input.pointer,[90,80]);assert_eq!(client.input.text,[120]);
 assert!(pending.key_events.is_empty());assert!(pending.held.contains(&25));
 println!("Interface batch: zero-tick retention, five timer ticks, held-state preservation and error restoration passed");
 Ok(())
}
