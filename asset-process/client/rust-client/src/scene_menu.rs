//! Scene actor contributions retain builder boundaries for Java's 400-entry guard.
use crate::context_menu::{Action,Entry};
use std::ops::Range;
#[derive(Clone,Debug)]
pub struct ActorContribution {
 pub entries:Range<usize>,
 pub walk_target:Option<String>,
}
pub fn append(entries:&mut Vec<Entry<Action>>,world:&[Entry<Action>],actors:&[ActorContribution]) {
 let mut cursor=0;
 for actor in actors {
  entries.extend_from_slice(&world[cursor..actor.entries.start]);
  // The Java count includes Cancel; the Rust action list does not.
  if entries.len()<399 {
   entries.extend_from_slice(&world[actor.entries.clone()]);
   if let Some(target)=&actor.walk_target {
    if let Some(walk)=entries.iter_mut().find(|e|matches!(e.action.scene().1,Action::Walk(_))) {
     walk.target=target.clone();
    }
   }
  }
  cursor=actor.entries.end;
 }
 entries.extend_from_slice(&world[cursor..]);
}
