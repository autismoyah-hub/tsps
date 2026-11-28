//! Private adapter admission helpers. Root-view visible menus remain authoritative.
use crate::{context_menu::{Action, Entry}, live_runtime::Connection, client_state::ClientState, Result};
/// Menu probes per command, including the original interior pixel.
pub const MAX_PROBES:usize=8;
/// Refusal diagnostics: tried points and menu labels kept per point.
const SEEN_POINTS:usize=3;
const SEEN_OPTIONS:usize=6;
pub fn matches_tag(action: Action, view:u16, tag:u64, base:[i32;2])->bool {
 if view!=0 {return false;}
 let tile=[((tag>>14)&16383) as i32+base[0],(tag&16383) as i32+base[1]];
 let same=|target:[u16;2]|tile==target.map(i32::from);
 match action {
  Action::Walk(target)=>tag>>63==1 && same(target),
  Action::Npc{index,..}|Action::ExamineNpc(index)=>tag>>60==4 && tag as u16==index,
  Action::Object{id,tile:target,..}|Action::CastWorld{kind:0,id,tile:target}=>tag>>60==0 && (tag>>32).checked_sub(1)==Some(u64::from(id)) && same(target),
  Action::Ground{tile:target,..}=>tag>>60==1 && same(target),
  _=>false,
 }
}
pub fn menu_contains(entries:&[Entry<Action>], wanted:Action)->bool {
 entries.iter().any(|e|(!e.submenu && e.children.is_empty() && !e.option.trim().is_empty() && e.action==wanted) || menu_contains(&e.children,wanted))
}
/// Native right-click menus prepend Cancel before the context-specific entries.
/// The private adapter must use the same final menu, not bypass its admission.
fn native_menu_entries(mut entries:Vec<Entry<Action>>)->Vec<Entry<Action>> {
 entries.insert(0,Entry{option:"Cancel".into(),target:String::new(),action:Action::Cancel,submenu:false,children:Vec::new()});
 entries
}
fn widget_point(client:&ClientState,id:u32)->Result<Option<[i32;2]>> {
 let layout=client.interfaces.layout(client.canvas)?;
 Ok(layout.widgets.get(&id).filter(|p|!p.hidden).and_then(|p|{
  let lo=[p.absolute[0].max(p.clip[0]),p.absolute[1].max(p.clip[1])];
  let hi=[(p.absolute[0]+p.geometry.size[0]).min(p.clip[2]),(p.absolute[1]+p.geometry.size[1]).min(p.clip[3])];
  (lo[0]<hi[0]&&lo[1]<hi[1]).then_some([(lo[0]+hi[0])/2,(lo[1]+hi[1])/2])
 }))
}
fn dispatch_point(input:&mut crate::input::Input,point:[i32;2]) {
 input.apply(crate::input::Update::Pointer(point));
 // The adapter directly dispatches an admitted menu action rather than a
 // physical button event. Deferred scene walking still reads its click point.
 input.press_position=point;
}
/// Terrain under authored floors or scene models is hidden from the visible pick
/// buffer, but still selects through the ground buffer that "Walk here" reads.
/// Partly covered targets yield further separated pixels, capped at MAX_PROBES.
fn locate(action:Action,size:[usize;2],visible:(&[u64],&[u16]),ground:(&[u64],&[u16]),base:[i32;2])->Vec<([usize;2],u64)> {
 let (tags,views)=if matches!(action,Action::Walk(_)) {ground} else {visible};
 crate::pick_probe::candidates(size,tags,views,|view,tag|matches_tag(action,view,tag,base),MAX_PROBES)
}
/// Native menus are built at the pointer, so the pointer moves to each candidate
/// before its menu is read. A probe that admits nothing restores the pointer.
/// Refused probes record their leading native menu labels for diagnostics.
fn admit_first<S>(state:&mut S,input:fn(&mut S)->&mut crate::input::Input,points:&[[i32;2]],action:Action,
  mut menu:impl FnMut(&mut S,[i32;2])->Result<Vec<Entry<Action>>>,
  mut dispatch:impl FnMut(&mut S,[i32;2])->Result<()>,seen:&mut Vec<Seen>)->Result<Option<Admission>> {
 let (original,original_press)=(input(state).pointer,input(state).press_position);
 // Every path that does not complete an admitted dispatch restores both.
 let restore=|state:&mut S|{let i=input(state);i.apply(crate::input::Update::Pointer(original));i.press_position=original_press;};
 for (index,&point) in points.iter().take(MAX_PROBES).enumerate() {
  input(state).apply(crate::input::Update::Pointer(point));
  let entries=match menu(state,point) {
   Ok(entries)=>native_menu_entries(entries),
   Err(error)=>{restore(state);return Err(error);}
  };
  if !menu_contains(&entries,action) {
   if seen.len()<SEEN_POINTS {
    let options=entries.iter().take(SEEN_OPTIONS).map(|e|format!("{} {}",e.option,e.target).trim().to_owned()).collect();
    seen.push(Seen{point,options});
   }
   continue;
  }
  // Draw-time terrain selection consumes this normal pointer. Never convert
  // Walk's requested tile into a direct movement packet.
  dispatch_point(input(state),point);
  if let Err(error)=dispatch(state,point) {restore(state);return Err(error);}
  return Ok(Some(Admission{index,point}));
 }
 restore(state);
 Ok(None)
}
/// The probe whose native menu admitted and dispatched the action, in candidate
/// order (0 is the interior pixel). Observational only: recorded after dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Admission { pub index:usize, pub point:[i32;2] }
/// One refused probe: its point and the leading native menu labels there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seen { pub point:[i32;2], pub options:Vec<String> }
pub enum Dispatch {
 /// `seen` holds the bounded refused probes that preceded the admitted one.
 Admitted(Admission,Vec<Seen>),
 Refused(Vec<Seen>),
}
fn client_input<'a>(state:&'a mut (&mut ClientState,&mut Connection))->&'a mut crate::input::Input {&mut state.0.input}
pub fn dispatch_visible(client:&mut ClientState, live:&mut Connection, action:Action)->Result<Dispatch> {
 let points:Vec<_>=match action {
  Action::Cancel=>vec![client.input.pointer],
  Action::Close=>{
   let ids:Vec<_>=client.interfaces.widgets.iter().filter(|(_,w)|w.button==3).map(|(&id,_)|id).collect();
   let mut points=Vec::new();for id in ids {if let Some(point)=widget_point(client,id)? {points.push(point);}}points
  }
  Action::Interface{id,..}|Action::Continue(id)|Action::SelectSpell(id)|Action::CastInterface(id)|Action::Button{id,..}=>widget_point(client,id)?.into_iter().collect(),
  _=>live.world.as_ref().zip(live.private_pick_frame()).map(|((scene,_),(frame,origin))|
   locate(action,[frame.width,frame.height],(&frame.tags,&frame.views),(&frame.ground_tags,&frame.ground_views),[scene.base_x,scene.base_y])
    .into_iter().map(|([x,y],_)|[origin[0]+x as i32,origin[1]+y as i32]).collect()).unwrap_or_default(),
 };
 let mut seen=Vec::new();
 Ok(match admit_first(&mut (client,live),client_input,&points,action,
  |(client,live),point|live.context_menu(client,point),
  |(client,_),point|client.menu_action_at(action,point),&mut seen)? {
  Some(admission)=>Dispatch::Admitted(admission,seen),
  None=>Dispatch::Refused(seen),
 })
}
// ---------------------------------------------------------------------------
// IF3 modal close (steelborder close controls), delivered at consume time.
// Design: reports/lead-b-20261024-1844/client-adapter/if3-close-design.md §5-6.
// ---------------------------------------------------------------------------
/// Polls a delivery may wait for its first ticking frame before refusing.
pub const CLOSE_ARM_POLLS:u32=50;
/// Polls after the release tick before an unclosed modal is refused.
pub const CLOSE_RESULT_POLLS:u32=150;
/// Polls a delivered press may wait for its release tick (for example while a
/// script chain keeps the interface tick from running) before the watchdog
/// releases the owned button and refuses. Always more than one poll, so a
/// successful press tick is never released in the same frame.
pub const CLOSE_PRESS_POLLS:u32=50;
/// Opcode of the client CLOSE_MODAL packet queued by `close_interfaces`.
const CLOSE_MODAL:u16=95;
/// [clientscript,closebutton_over]/[clientscript,closebutton_leave] (27/28): the
/// close control's own visual hover listeners from [proc,closebutton_action].
const CLOSE_HOVER:[(u16,i32);2]=[(1403,27),(1404,28)];
/// [clientscript,closebutton_click] (29), the onclick that [proc,closebutton_action]
/// (26) sets on steelborder close children: exactly `if_close; return`, with a
/// single script-id argument and no locals or switches. Anything that reaches 3103
/// conditionally, alongside side effects or unreachably is not a close control.
pub fn close_only(values:&[crate::vm::Value],script:&crate::script::Script)->bool {
 matches!(values,[crate::vm::Value::Int(_)])
  && (script.int_arguments,script.string_arguments,script.long_arguments)==(0,0,0)
  && (script.local_ints,script.local_strings,script.local_longs)==(0,0,0)
  && script.switches.is_empty()
  && script.instructions.len()==2
  && script.instructions[0].opcode==3103
  && script.instructions[1].opcode==21
}
fn listener_script(client:&ClientState,id:u32,opcode:u16)->Option<Option<i32>> {
 client.interfaces.listener(id,opcode).map(|(values,_)|match values.first() {Some(&crate::vm::Value::Int(script))=>Some(script),_=>None})
}
fn close_control(client:&mut ClientState,id:u32)->Result<bool> {
 if !client.interfaces.widgets.get(&id).is_some_and(|w|w.if3) {return Ok(false);}
 let Some((values,_))=client.interfaces.listener(id,1400) else {return Ok(false);};
 let Some(&crate::vm::Value::Int(script))=values.first() else {return Ok(false);};
 let Ok(script)=crate::vm::Host::script(&mut client.scripts,script) else {return Ok(false);};
 Ok(close_only(&values,&script))
}
fn modal_groups(client:&ClientState)->std::collections::HashSet<u16> {
 client.interfaces.children.values().filter(|s|s.kind==0||s.kind==3).map(|s|s.group).collect()
}
fn group_of(client:&ClientState,id:u32)->Option<u16> {
 client.interfaces.runtime.get(&id).map(|r|(r.logical_id>>16) as u16)
}
fn clipped_centre(layout:&crate::widget_layout::Layout,id:u32)->Option<[i32;2]> {
 layout.widgets.get(&id).filter(|p|!p.hidden).and_then(|p|{
  let lo=[p.absolute[0].max(p.clip[0]),p.absolute[1].max(p.clip[1])];
  let hi=[(p.absolute[0]+p.geometry.size[0]).min(p.clip[2]),(p.absolute[1]+p.geometry.size[1]).min(p.clip[3])];
  (lo[0]<hi[0]&&lo[1]<hi[1]).then_some([(lo[0]+hi[0])/2,(lo[1]+hi[1])/2])
 })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CloseTarget { pub widget:u32, pub instance:u64, pub group:u16, pub point:[i32;2] }
/// The first exact close-only control of an open modal (layout order) whose
/// clipped centre passes `validate_press` with the current tick pointer.
pub fn close_target(client:&mut ClientState,pointer:[i32;2])->Result<Option<CloseTarget>> {
 let groups=modal_groups(client);
 if groups.is_empty() || client.interfaces.closing_modal {return Ok(None);}
 let layout=client.interfaces.layout(client.canvas)?;
 for &id in &layout.order {
  let Some(group)=group_of(client,id).filter(|g|groups.contains(g)) else {continue;};
  let Some(point)=clipped_centre(&layout,id) else {continue;};
  if !close_control(client,id)? {continue;}
  let target=CloseTarget{widget:id,instance:client.interfaces.runtime[&id].instance,group,point};
  if validate_press(client,target,pointer)?.is_ok() {return Ok(Some(target));}
 }
 Ok(None)
}
/// Revalidated immediately before the interface tick that consumes the press.
/// Ok(ids) lists every press-set widget/instance whose 1400 will be queued.
pub fn validate_press(client:&mut ClientState,target:CloseTarget,pointer:[i32;2])->Result<std::result::Result<Vec<(u32,u64)>,&'static str>> {
 if client.interfaces.closing_modal || !modal_groups(client).contains(&target.group) {return Ok(Err("target modal is no longer open"));}
 if client.menu.is_some() || client.widget_drag.is_some() {return Ok(Err("a menu or widget drag is active"));}
 // A left press that hits no editor blurs a focused one (dispatch_editor_pointer ->
 // blur_editor), queueing its 1436 callback outside the observed close window.
 if client.interfaces.focused_editor().is_some() {return Ok(Err("an editor has focus"));}
 if client.interfaces.runtime.get(&target.widget).is_none_or(|r|r.instance!=target.instance) {return Ok(Err("close control was replaced"));}
 let layout=client.interfaces.layout(client.canvas)?;
 if clipped_centre(&layout,target.widget)!=Some(target.point) {return Ok(Err("close control moved or is hidden"));}
 // Every hit widget receives the press (interface_events.rs collect), ancestors included.
 let hit:Vec<u32>=layout.hit_test(target.point).collect();
 let Some(&top)=hit.first() else {return Ok(Err("close control is not under its point"));};
 if group_of(client,top)!=Some(target.group) || !close_control(client,top)? {return Ok(Err("close control is not topmost"));}
 let mut ids=Vec::new();
 for &id in &hit {
  let Some(w)=client.interfaces.widgets.get(&id) else {continue;};
  if !w.if3 {continue;}
  if matches!(w.kind,11|12) {return Ok(Err("press point has an editor"));}
  if [1401,1411,1402].iter().any(|&op|client.interfaces.listener(id,op).is_some()) {return Ok(Err("press path has hold or release listeners"));}
  if client.interfaces.listener(id,1400).is_some() {
   if group_of(client,id)!=Some(target.group) || !close_control(client,id)? {return Ok(Err("press path has another click listener"));}
   ids.push((id,client.interfaces.runtime[&id].instance));
  }
 }
 if !ids.iter().any(|&(id,_)|id==target.widget) {return Ok(Err("close control is not in the press set"));}
 // While the left button is held, collect queues 1401 for every widget under the
 // pointer (not only the press set), and 1411/1402 for any of them already in the
 // clicked state from earlier input; editors under the pointer receive drag
 // pointer events. Refuse all of these before delivering anything.
 let over:std::collections::HashSet<u32>=layout.hit_test(pointer).collect();
 for &id in &over {
  let Some(w)=client.interfaces.widgets.get(&id) else {continue;};
  if !w.if3 {continue;}
  if matches!(w.kind,11|12) {return Ok(Err("pointer is over an editor"));}
  if [1401,1411,1402].iter().any(|&op|client.interfaces.listener(id,op).is_some()) {return Ok(Err("pointer is over a hold or release listener"));}
 }
 // Hover follows the pointer, not the press. Allow only the close control's own
 // visual 27/28 where the pointer already overlaps the press set.
 for &id in hit.iter().filter(|id|over.contains(id)) {
  if !client.interfaces.widgets.get(&id).is_some_and(|w|w.if3) {continue;}
  let close=close_control(client,id)?;
  for op in [1403u16,1404,1412] {
   let Some(script)=listener_script(client,id,op) else {continue;};
   let allowed=close && CLOSE_HOVER.iter().any(|&(o,s)|o==op && script==Some(s));
   if !allowed {return Ok(Err("pointer is over the press set with hover listeners"));}
  }
 }
 Ok(Ok(ids))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CloseStage { Armed, Pressed, Released, Done }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClosePhase { Press, Release }
/// One IF3 close: a client-internal press on a ticking frame, its release on the
/// next ticking frame, and the bounded client-side observation of both ticks.
#[derive(Debug)]
pub struct CloseDriver {
 pub sequence:u64,
 pub target:CloseTarget,
 stage:CloseStage,
 phase:Option<ClosePhase>,
 polls:u32,
 mark:usize,
 expected:Vec<(u32,u64)>,
 pressed:Option<Vec<(u32,u64)>>,
 press_packets:Option<Vec<u16>>,
 release_packets:Option<Vec<u16>>,
 outcome:Option<&'static str>,
 /// A delivered left press this driver still owns (not yet released).
 holds:bool,
}
fn deliver(client:&mut ClientState,tick_input:&mut crate::input::Input,update:crate::input::Update)->Result<()> {
 // Same dispatch as a platform mouse message, but only the simulation view: the
 // throwaway render view keeps pending_clicks/default menu and telemetry out.
 let mut scratch=crate::input::Input::default();
 crate::input::dispatch(update,[tick_input,&mut scratch],Some(&mut client.interfaces),|_|None,|_|false)
}
impl CloseDriver {
 pub fn arm(sequence:u64,target:CloseTarget)->Self {
  Self{sequence,target,stage:CloseStage::Armed,phase:None,polls:0,mark:0,expected:Vec::new(),
   pressed:None,press_packets:None,release_packets:None,outcome:None,holds:false}
 }
 /// H-pre, immediately before `tick_interfaces_elapsed_with(ticks, ..)`. Delivers
 /// nothing when no internal tick will run (`ticks == 0`). Never fails: an
 /// admission or delivery error becomes a bounded refusal that releases any owned
 /// press. Returns the error text for the adapter to report.
 pub fn before_tick(&mut self,client:&mut ClientState,tick_input:&mut crate::input::Input,ticks:u32)->Option<String> {
  if ticks==0 {return None;}
  match self.try_before_tick(client,tick_input) {
   Ok(())=>None,
   Err(error)=>{self.fail(tick_input,"close admission or delivery failed");Some(error.to_string())}
  }
 }
 fn try_before_tick(&mut self,client:&mut ClientState,tick_input:&mut crate::input::Input)->Result<()> {
  match self.stage {
   CloseStage::Armed=>match validate_press(client,self.target,tick_input.pointer)? {
    Err(reason)=>{self.stage=CloseStage::Done;self.outcome=Some(reason);}
    Ok(ids)=>{
     self.expected=ids;
     self.mark=client.outbound.len();
     self.stage=CloseStage::Pressed;self.phase=Some(ClosePhase::Press);self.polls=0;self.holds=true;
     deliver(client,tick_input,crate::input::Update::ButtonAt(0,true,self.target.point))?;
    }
   },
   // The press tick has been observed: release on this later ticking frame only.
   CloseStage::Pressed if self.press_packets.is_some()=>{
    self.mark=client.outbound.len();
    self.stage=CloseStage::Released;self.phase=Some(ClosePhase::Release);self.polls=0;
    deliver(client,tick_input,crate::input::Update::ButtonAt(0,false,self.target.point))?;
    self.holds=false;
   }
   _=>{}
  }
  Ok(())
 }
 /// From the per-tick callback of `tick_interfaces_elapsed_with`, after every
 /// internal tick. Only the first internal tick after a delivery is observed: that
 /// is the tick that consumes the press or release, even in catch-up batches.
 pub fn observe_tick(&mut self,client:&ClientState) {
  let mark=self.mark;
  let added=||client.outbound.iter().skip(mark).map(|p|p.opcode).collect::<Vec<_>>();
  match self.phase.take() {
   Some(ClosePhase::Press)=>{
    self.press_packets=Some(added());
    self.pressed=Some(client.interface_events.last_pressed().to_vec());
   }
   Some(ClosePhase::Release)=>self.release_packets=Some(added()),
   None=>{}
  }
 }
 /// Clears only this driver's left-button state: releases a held left button
 /// and drops a pending left press. Other buttons and keys are untouched.
 fn release_owned(&mut self,tick_input:&mut crate::input::Input) {
  if !self.holds {return;}
  if tick_input.buttons&1!=0 {tick_input.apply(crate::input::Update::ButtonAt(0,false,self.target.point));}
  tick_input.pressed_buttons&=!1;
  self.holds=false;
 }
 fn fail(&mut self,tick_input:&mut crate::input::Input,reason:&'static str) {
  self.release_owned(tick_input);
  self.phase=None;
  if self.stage!=CloseStage::Done {self.stage=CloseStage::Done;self.outcome=Some(reason);}
 }
 /// H-post, after the whole tick batch returns: a delivery that no internal tick
 /// observed (the tick failed before its callback) is released and refused.
 pub fn after_frame(&mut self,tick_input:&mut crate::input::Input) {
  if self.phase.is_some() {self.fail(tick_input,"interface tick did not observe the close delivery");}
 }
 /// H-reset (title/reconnect) or stop: release any owned press and end the close.
 pub fn abort(&mut self,tick_input:&mut crate::input::Input) {self.fail(tick_input,"close aborted before completion");}
 /// After every adapter poll, ticking or not: a delivered press that has waited
 /// CLOSE_PRESS_POLLS polls without its release tick is released and refused.
 pub fn watchdog(&mut self,tick_input:&mut crate::input::Input) {
  if self.stage==CloseStage::Pressed && self.polls>=CLOSE_PRESS_POLLS {
   self.fail(tick_input,"close press timed out before a release tick");
  }
 }
 /// Once per adapter poll. Returns the final receipt status when decided.
 pub fn poll(&mut self,client:&ClientState)->Option<&'static str> {
  self.polls=self.polls.saturating_add(1);
  match self.stage {
   CloseStage::Done=>return self.outcome,
   CloseStage::Armed if self.polls>=CLOSE_ARM_POLLS=>self.finish("no interface tick before the close timed out"),
   CloseStage::Released if self.release_packets.is_some()=>self.observe(client),
   _=>None,
  }
 }
 fn finish(&mut self,status:&'static str)->Option<&'static str> {self.stage=CloseStage::Done;self.outcome=Some(status);self.outcome}
 fn observe(&mut self,client:&ClientState)->Option<&'static str> {
  let press=self.press_packets.as_deref().unwrap_or_default();
  if press.iter().filter(|&&op|op==CLOSE_MODAL).count()!=1 {return self.finish("close tick did not queue exactly one CLOSE_MODAL");}
  if press.len()!=1 {return self.finish("unexpected packets in the close press tick");}
  if !self.release_packets.as_deref().unwrap_or_default().is_empty() {return self.finish("unexpected packets in the close release tick");}
  let mut pressed=self.pressed.clone().unwrap_or_default();let mut expected=self.expected.clone();
  pressed.sort_unstable();expected.sort_unstable();
  if pressed!=expected {return self.finish("click was not queued for exactly the validated close controls");}
  let closing=client.interfaces.closing_modal;
  let alive=client.interfaces.runtime.get(&self.target.widget).is_some_and(|r|r.instance==self.target.instance);
  let open=modal_groups(client).contains(&self.target.group);
  if !closing && !alive && !open {return self.finish("accepted: CLOSE_MODAL queued client-side");}
  if !closing && !alive {return self.finish("modal was replaced before the close completed");}
  if self.polls>=CLOSE_RESULT_POLLS {return self.finish("close control did not close the modal");}
  None
 }
 pub fn is_done(&self)->bool {self.stage==CloseStage::Done}
 pub fn press_delivered(&self)->bool {self.stage==CloseStage::Pressed}
 /// Whether this driver still owns a delivered, unreleased left press.
 pub fn holds_press(&self)->bool {self.holds}
}
// ---------------------------------------------------------------------------
// IF3 native-menu close. [proc,steelborder] (228) builds the steelborder close as
// cc_setop(1,"Close") with cc_setonop = [clientscript,closebutton_click] (29) and no
// onclick, so the real client closes through the menu op, not a 1400 press.
// Analysis: reports/lead-b-20261024-1844/client-if3-close/cache-read-pr100-analysis.md.
// ---------------------------------------------------------------------------
/// Exact menu label of the steelborder close op.
pub const CLOSE_LABEL:&str="Close";
/// Widgets recorded in a refusal diagnostic before it is marked truncated.
pub const DIAGNOSTIC_WIDGETS:usize=256;
/// Close op candidates, and widgets above each, recorded in a refusal diagnostic.
pub const DIAGNOSTIC_CANDIDATES:usize=4;
pub const DIAGNOSTIC_ABOVE:usize=32;
pub const DIAGNOSTIC_MENU:usize=16;
/// How a listener's script classifies. Load failures stay distinct from "not close".
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScriptCheck { CloseOnly, NotClose(&'static str), Pending(String), Failed(String) }
/// Classifies the listener for `opcode` without running it.
pub fn listener_check(client:&mut ClientState,id:u32,opcode:u16)->Option<ScriptCheck> {
 let (values,_)=client.interfaces.listener(id,opcode)?;
 let Some(&crate::vm::Value::Int(script))=values.first() else {return Some(ScriptCheck::NotClose("listener has no script id"));};
 Some(match crate::vm::Host::script(&mut client.scripts,script) {
  Ok(script)=>if close_only(&values,&script) {ScriptCheck::CloseOnly} else {ScriptCheck::NotClose("script is not exactly if_close; return")},
  Err(error@crate::Error::PendingAsset{..})=>ScriptCheck::Pending(error.to_string()),
  Err(error)=>ScriptCheck::Failed(error.to_string()),
 })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuCloseTarget { pub widget:u32, pub instance:u64, pub group:u16, pub point:[i32;2], pub operation:u8 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuClose {
 /// No visible "Close" op in an open modal: the 1400 route may still apply.
 Absent,
 /// The unique candidate's onop script is not local yet; retry within a bound.
 Pending(String),
 /// A candidate exists but is not an admissible close-only menu op.
 Refused(String),
 Target(MenuCloseTarget),
}
/// Visible IF3 widgets of open modals with an exact "Close" op, in paint order:
/// (id, group, clipped centre, op index).
fn close_op_candidates(client:&ClientState,layout:&crate::widget_layout::Layout,groups:&std::collections::HashSet<u16>)->Vec<(u32,u16,[i32;2],usize)> {
 let mut found=Vec::new();
 for &id in &layout.order {
  let Some(group)=group_of(client,id).filter(|g|groups.contains(g)) else {continue;};
  let Some(widget)=client.interfaces.widgets.get(&id) else {continue;};
  if !widget.if3 {continue;}
  let Some(index)=widget.actions.iter().position(|a|a==CLOSE_LABEL) else {continue;};
  let Some(point)=clipped_centre(layout,id) else {continue;};
  found.push((id,group,point,index));
 }
 found
}
/// Why `id`, painted above a target, could take its click or add a menu entry;
/// None when inert. Conservative: any op, button, editor, scene/map content,
/// pointer listener, op mask or interaction flag counts. Bit 21 (spell/item use
/// target) only applies with a selection, which `menu_close_target` refuses.
fn overlay_blocker(client:&ClientState,id:u32)->Option<&'static str> {
 let Some(w)=client.interfaces.widgets.get(&id) else {return Some("no definition");};
 if w.actions.iter().any(|a|!a.is_empty()) {return Some("has ops");}
 if w.button!=0 {return Some("legacy button");}
 if matches!(w.kind,11|12) {return Some("editor");}
 if matches!(w.content,1337|1338|1400) {return Some("scene or map content");}
 if [1400u16,1401,1402,1409,1411,1412].iter().any(|&op|client.interfaces.listener(id,op).is_some()) {return Some("pointer listener");}
 if client.interfaces.operation_mask(id)!=0 {return Some("operation mask");}
 if client.interfaces.interaction_flags(id)&!(1<<21)!=0 {return Some("interaction flags");}
 None
}
fn inside(point:[i32;2],clip:[i32;4])->bool {point[0]>=clip[0]&&point[1]>=clip[1]&&point[0]<clip[2]&&point[1]<clip[3]}
/// How the native menu's own picking (`interface_menu_order`, Java ps.ap) sees
/// `id` at `point`: absent when a later IF3 no_click_through barrier clears it,
/// else the widgets painted above it there.
fn picked_above(client:&ClientState,layout:&crate::widget_layout::Layout,id:u32,point:[i32;2])->Option<Vec<u32>> {
 let picked=crate::interface_menu_order::widgets(&client.interfaces,layout,point,point);
 let at=picked.iter().position(|&p|p==id)?;
 Some(picked[at+1..].to_vec())
}
/// The unique visible IF3 widget of an open modal with an exact "Close" op, picked
/// at its clipped centre by the native menu order with only inert widgets above
/// it, whose onop (1409) is exactly close-only and whose op does not also transmit
/// IF_BUTTON. Resolved from runtime state, never from a fixed id.
pub fn menu_close_target(client:&mut ClientState)->Result<MenuClose> {
 let groups=modal_groups(client);
 if groups.is_empty() {return Ok(MenuClose::Absent);}
 if client.interfaces.closing_modal {return Ok(MenuClose::Refused("a modal is already closing".into()));}
 if client.menu.is_some() || client.widget_drag.is_some() {return Ok(MenuClose::Refused("a menu or widget drag is active".into()));}
 if client.selected_spell.is_some() {return Ok(MenuClose::Refused("a spell or item selection is active".into()));}
 let layout=client.interfaces.layout(client.canvas)?;
 let found=close_op_candidates(client,&layout,&groups);
 let (id,group,point,index)=match found.as_slice() {
  []=>return Ok(MenuClose::Absent),
  [only]=>*only,
  many=>return Ok(MenuClose::Refused(format!("{} visible Close ops in open modals",many.len()))),
 };
 if index>=32 {return Ok(MenuClose::Refused("Close op index out of range".into()));}
 // Inert overlays (title text, empty toplevel layers) never take a click in the
 // native client, so geometric topmost is not the rule; see interface_menu_order.
 let Some(above)=picked_above(client,&layout,id,point) else {
  return Ok(MenuClose::Refused("Close control is behind a click-through barrier at its centre".into()));
 };
 if let Some((cover,reason))=above.iter().find_map(|&a|overlay_blocker(client,a).map(|r|(a,r))) {
  return Ok(MenuClose::Refused(format!("Close control is covered by interactive widget {cover} ({reason})")));
 }
 match listener_check(client,id,1409) {
  None=>return Ok(MenuClose::Refused("Close op has no onop listener (server IF_BUTTON route unsupported)".into())),
  Some(ScriptCheck::CloseOnly)=>{}
  Some(ScriptCheck::NotClose(reason))=>return Ok(MenuClose::Refused(format!("Close onop: {reason}"))),
  Some(ScriptCheck::Pending(error))=>return Ok(MenuClose::Pending(error)),
  Some(ScriptCheck::Failed(error))=>return Ok(MenuClose::Refused(format!("Close onop script failed to load: {error}"))),
 }
 if client.interfaces.operation_mask(id)&(1<<index)!=0 {
  return Ok(MenuClose::Refused("Close op would also transmit IF_BUTTON (unsupported)".into()));
 }
 let instance=client.interfaces.runtime[&id].instance;
 Ok(MenuClose::Target(MenuCloseTarget{widget:id,instance,group,point,operation:index as u8+1}))
}
/// The client and native menu a menu close is admitted through.
pub trait MenuHost {
 fn client(&mut self)->&mut ClientState;
 fn menu(&mut self,point:[i32;2])->Result<Vec<Entry<Action>>>;
}
impl MenuHost for (&mut ClientState,&mut Connection) {
 fn client(&mut self)->&mut ClientState {&mut *self.0}
 fn menu(&mut self,point:[i32;2])->Result<Vec<Entry<Action>>> {self.1.context_menu(&mut *self.0,point)}
}
fn host_input<H:MenuHost>(host:&mut H)->&mut crate::input::Input {&mut host.client().input}
/// Revalidates the target, then admits its exact op through the real native menu
/// at its centre and dispatches it with `menu_action_at`, as choosing the option
/// does. Returns the opcodes this dispatch queued, or a refusal before dispatch.
pub fn dispatch_menu_close<H:MenuHost>(host:&mut H,target:MenuCloseTarget)->Result<std::result::Result<Vec<u16>,String>> {
 match menu_close_target(host.client())? {
  MenuClose::Target(current) if current==target=>{}
  _=>return Ok(Err("close control changed before dispatch".into())),
 }
 let action=Action::Interface{id:target.widget,operation:target.operation,sub:None};
 let mark=host.client().outbound.len();
 let mut seen=Vec::new();
 let admitted=admit_first(host,host_input::<H>,&[target.point],action,
  |host,point|host.menu(point),
  |host,point|host.client().menu_action_at(action,point),&mut seen)?;
 if admitted.is_none() {return Ok(Err("Close op is not in the native menu at its centre".into()));}
 Ok(Ok(host.client().outbound.iter().skip(mark).map(|p|p.opcode).collect()))
}
/// Read-only native menu at each Close op centre, for a refusal diagnostic. The
/// pointer moves there to build the menu, as admission does, and is restored with
/// the press point; nothing is dispatched.
pub fn close_menu_probe<H:MenuHost>(host:&mut H)->Result<String> {
 let groups=modal_groups(host.client());
 let layout={let client=host.client();client.interfaces.layout(client.canvas)?};
 let points:Vec<[i32;2]>=close_op_candidates(host.client(),&layout,&groups).into_iter()
  .take(DIAGNOSTIC_CANDIDATES).map(|(_,_,point,_)|point).collect();
 let (pointer,press)={let i=host_input::<H>(host);(i.pointer,i.press_position)};
 let mut out=Vec::new();
 for point in points {
  host_input::<H>(host).apply(crate::input::Update::Pointer(point));
  let entries=host.menu(point);
  let i=host_input::<H>(host);
  i.apply(crate::input::Update::Pointer(pointer));
  i.press_position=press;
  let entries=native_menu_entries(entries?);
  let rows:Vec<String>=entries.iter().take(DIAGNOSTIC_MENU).map(|e|format!("{{\"option\":{},\"target\":{},\"action\":{},\"children\":{}}}",
   json(&e.option),json(&e.target),json(&format!("{:?}",e.action)),e.children.len())).collect();
  out.push(format!("{{\"point\":[{},{}],\"total\":{},\"entries\":[{}]}}",point[0],point[1],entries.len(),rows.join(",")));
 }
 Ok(format!("[{}]",out.join(",")))
}
/// The menu close finished client-side: the pinned instance is gone and its modal
/// group is no longer mounted.
pub fn menu_close_done(client:&ClientState,target:MenuCloseTarget)->bool {
 client.interfaces.runtime.get(&target.widget).is_none_or(|r|r.instance!=target.instance)
  && !client.interfaces.closing_modal && !modal_groups(client).contains(&target.group)
}
/// Modal groups mounted now (kind 0 or 3), for exports.
pub fn open_modal_groups(client:&ClientState)->std::collections::HashSet<u16> {modal_groups(client)}
fn json(value:&str)->String {
 let mut out=String::from("\"");
 for c in value.chars() {
  match c {
   '"'=>out.push_str("\\\""),
   '\\'=>out.push_str("\\\\"),
   c if c<' '=>out.push_str(&format!("\\u{:04x}",c as u32)),
   c=>out.push(c),
  }
 }
 out.push('"');
 out
}
fn check_json(check:Option<ScriptCheck>)->String {
 match check {
  None=>"null".into(),
  Some(ScriptCheck::CloseOnly)=>"\"close_only\"".into(),
  Some(ScriptCheck::NotClose(reason))=>format!("{{\"not_close\":{}}}",json(reason)),
  Some(ScriptCheck::Pending(error))=>format!("{{\"pending\":{}}}",json(&error)),
  Some(ScriptCheck::Failed(error))=>format!("{{\"failed\":{}}}",json(&error)),
 }
}
/// Why the 1400 press route skips `id`; None when it would arm.
fn press_skip(client:&mut ClientState,layout:&crate::widget_layout::Layout,id:u32,pointer:[i32;2])->Result<Option<String>> {
 let Some(point)=clipped_centre(layout,id) else {return Ok(Some("no visible clipped centre".into()));};
 if !close_control(client,id)? {return Ok(Some("not a 1400 close-only control".into()));}
 let Some(group)=group_of(client,id) else {return Ok(Some("no group".into()));};
 let target=CloseTarget{widget:id,instance:client.interfaces.runtime[&id].instance,group,point};
 Ok(validate_press(client,target,pointer)?.err().map(str::to_owned))
}
/// Diagnostic row order: exact Close ops (and the resolved target) first, then
/// click/op listeners or actions, then the rest, so the cap never hides a close.
fn diagnostic_rank(client:&ClientState,id:u32,menu:&MenuClose)->u8 {
 let close=client.interfaces.widgets.get(&id).is_some_and(|w|w.actions.iter().any(|a|a==CLOSE_LABEL));
 if close || matches!(menu,MenuClose::Target(t) if t.widget==id) {return 0;}
 let listener=client.interfaces.listener(id,1400).is_some() || client.interfaces.listener(id,1409).is_some();
 if listener || client.interfaces.widgets.get(&id).is_some_and(|w|!w.actions.is_empty()) {1} else {2}
}
/// Bounded refusal diagnostic for every widget of the open modal groups. Loads
/// listener scripts to classify them; never runs a listener.
pub fn close_diagnostics(client:&mut ClientState,pointer:[i32;2],menu:&MenuClose)->Result<String> {
 let groups=modal_groups(client);
 let layout=client.interfaces.layout(client.canvas)?;
 let mounts:Vec<String>=client.interfaces.children.iter().filter(|(_,s)|groups.contains(&s.group))
  .map(|(&parent,s)|format!("{{\"parent\":{parent},\"group\":{},\"kind\":{}}}",s.group,s.kind)).collect();
 let mut ids:Vec<u32>=client.interfaces.runtime.keys().copied()
  .filter(|&id|group_of(client,id).is_some_and(|g|groups.contains(&g))).collect();
 ids.sort_unstable_by_key(|&id|(diagnostic_rank(client,id,menu),id));
 let total=ids.len();
 let omitted=total.saturating_sub(DIAGNOSTIC_WIDGETS);
 let truncated=omitted>0;
 let mut rows=Vec::new();
 for id in ids.into_iter().take(DIAGNOSTIC_WIDGETS) {
  let Some(widget)=client.interfaces.widgets.get(&id) else {continue;};
  let (if3,kind,hidden,parent)=(widget.if3,widget.kind,widget.hidden,widget.parent);
  let actions=widget.actions.iter().map(|a|json(a)).collect::<Vec<_>>().join(",");
  let runtime=&client.interfaces.runtime[&id];
  let (instance,logical,child)=(runtime.instance,runtime.logical_id,runtime.child_index);
  let placement=layout.widgets.get(&id);
  let (placed,placement_hidden)=(placement.is_some(),placement.is_some_and(|p|p.hidden));
  let listeners=[1400u16,1409,1401,1402,1411,1403,1404,1412].iter()
   .map(|&op|format!("\"{op}\":{}",match listener_script(client,id,op) {
    None=>"null".to_owned(), Some(None)=>"\"non-int\"".to_owned(), Some(Some(s))=>s.to_string()}))
   .collect::<Vec<_>>().join(",");
  let mask=client.interfaces.operation_mask(id);
  let onclick=check_json(listener_check(client,id,1400));
  let onop=check_json(listener_check(client,id,1409));
  let skip=press_skip(client,&layout,id,pointer)?;
  let centre=clipped_centre(&layout,id).map_or("null".to_owned(),|c|format!("[{},{}]",c[0],c[1]));
  rows.push(format!("{{\"id\":{id},\"instance\":{instance},\"logical_id\":{logical},\"child_index\":{child},\"parent\":{},\"if3\":{if3},\"kind\":{kind},\"definition_hidden\":{hidden},\"placed\":{placed},\"placement_hidden\":{placement_hidden},\"centre\":{centre},\"actions\":[{actions}],\"operation_mask\":{mask},\"listeners\":{{{listeners}}},\"onclick_script\":{onclick},\"onop_script\":{onop},\"press_route_skip\":{}}}",
   parent.map_or("null".to_owned(),|p|p.to_string()),skip.map_or("null".to_owned(),|s|json(&s))));
 }
 let menu=match menu {
  MenuClose::Absent=>"\"absent\"".to_owned(),
  MenuClose::Pending(error)=>format!("{{\"pending\":{}}}",json(error)),
  MenuClose::Refused(reason)=>format!("{{\"refused\":{}}}",json(reason)),
  MenuClose::Target(t)=>format!("{{\"target\":{},\"instance\":{},\"operation\":{}}}",t.widget,t.instance,t.operation),
 };
 // Picking at each Close op centre: the geometric topmost id, whether the native
 // menu order keeps the Close, and every placement painted above it there (paint
 // index, no_click_through, native-order membership, blocker), bounded.
 let mut picks=Vec::new();
 for (id,_,point,_) in close_op_candidates(client,&layout,&groups).into_iter().take(DIAGNOSTIC_CANDIDATES) {
  let topmost=layout.hit_test(point).next().map_or("null".to_owned(),|t|t.to_string());
  let picked=picked_above(client,&layout,id,point);
  let paint=layout.order.iter().position(|&o|o==id).unwrap_or(0);
  let above:Vec<(usize,u32)>=layout.order.iter().copied().enumerate().skip(paint+1)
   .filter(|(_,a)|!layout.widgets[a].hidden && inside(point,layout.widgets[a].clip)).collect();
  let pick_rows:Vec<String>=above.iter().take(DIAGNOSTIC_ABOVE).map(|&(index,a)|{
   let r=&client.interfaces.runtime[&a];
   let w=client.interfaces.widgets.get(&a);
   format!("{{\"paint_index\":{index},\"id\":{a},\"logical_id\":{},\"child_index\":{},\"if3\":{},\"kind\":{},\"no_click_through\":{},\"interaction_flags\":{},\"in_menu_order\":{},\"blocker\":{}}}",
    r.logical_id,r.child_index,w.is_some_and(|w|w.if3),w.map_or(-1,|w|i32::from(w.kind)),w.is_some_and(|w|w.no_click_through),
    client.interfaces.interaction_flags(a),picked.as_ref().is_some_and(|p|p.contains(&a)),
    overlay_blocker(client,a).map_or("null".to_owned(),json))
  }).collect();
  picks.push(format!("{{\"id\":{id},\"paint_index\":{paint},\"centre\":[{},{}],\"topmost\":{topmost},\"picked\":{},\"above_total\":{},\"above_omitted\":{},\"above\":[{}]}}",
   point[0],point[1],picked.is_some(),above.len(),above.len().saturating_sub(DIAGNOSTIC_ABOVE),pick_rows.join(",")));
 }
 let mut groups:Vec<_>=groups.into_iter().collect();
 groups.sort_unstable();
 Ok(format!("{{\"pointer\":[{},{}],\"menu_open\":{},\"widget_drag\":{},\"closing_modal\":{},\"modal_groups\":{groups:?},\"mounts\":[{}],\"menu_route\":{menu},\"dispatched\":false,\"close_picks\":[{}],\"total\":{total},\"omitted\":{omitted},\"truncated\":{truncated},\"widgets\":[\n{}\n]}}",
  pointer[0],pointer[1],client.menu.is_some(),client.widget_drag.is_some(),client.interfaces.closing_modal,mounts.join(","),picks.join(","),rows.join(",\n")))
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn admitted_action_point_survives_motion_without_synthesizing_buttons() {
  let mut input=crate::input::Input::default();
  dispatch_point(&mut input,[170,190]);
  input.apply(crate::input::Update::Pointer([400,450]));
  assert_eq!(input.press_position,[170,190]);
  assert_eq!((input.buttons,input.pressed_buttons,input.released_buttons),(0,0,0));
  dispatch_point(&mut input,[220,240]);
  assert_eq!(input.pointer,[220,240]);
  assert_eq!(input.press_position,[220,240]);
 }
 fn entry(action:Action)->Entry<Action> {Entry{option:"Use".into(),target:String::new(),action,submenu:false,children:Vec::new()}}
 struct Probe {input:crate::input::Input,menus:Vec<([i32;2],[i32;2])>,dispatched:Vec<[i32;2]>}
 fn probe_input(probe:&mut Probe)->&mut crate::input::Input {&mut probe.input}
 /// Stands in for world_pointer::target: content exists only while the pointer
 /// is not over a no-click-through panel at x>=500.
 fn probe(stale:[i32;2],points:&[[i32;2]],action:Action)->(Probe,bool) {
  let (state,admitted,_)=probe_seen(stale,points,action);
  (state,admitted)
 }
 fn probe_seen(stale:[i32;2],points:&[[i32;2]],action:Action)->(Probe,bool,Vec<Seen>) {
  let mut state=Probe{input:crate::input::Input::default(),menus:Vec::new(),dispatched:Vec::new()};
  state.input.apply(crate::input::Update::Pointer(stale));
  let mut seen=Vec::new();
  let admitted=admit_first(&mut state,probe_input,points,action,
   |s,point|{s.menus.push((point,s.input.pointer));Ok(if s.input.pointer[0]<500 && point[0]<500 {vec![entry(action)]} else {vec![entry(Action::Walk([1,2]))]})},
   |s,point|{s.dispatched.push(point);Ok(())},&mut seen).unwrap();
  if let Some(admission)=admitted {assert_eq!(state.dispatched.last(),Some(&admission.point));}
  (state,admitted.is_some(),seen)
 }
 #[test] fn menu_is_built_at_the_candidate_pointer_not_a_stale_panel_pointer() {
  let action=Action::Object{id:909,tile:[10418,105],operation:1,sub:0};
  // Pointer left over the side panel by an earlier inventory click.
  let (state,admitted)=probe([600,300],&[[200,150]],action);
  assert!(admitted);
  assert_eq!(state.menus,vec![([200,150],[200,150])]);
  assert_eq!(state.dispatched,vec![[200,150]]);
  assert_eq!((state.input.pointer,state.input.press_position),([200,150],[200,150]));
  assert_eq!((state.input.buttons,state.input.pressed_buttons),(0,0));
 }
 #[test] fn refused_candidates_restore_pointer_and_dispatch_nothing() {
  let action=Action::Npc{index:4,operation:1,sub:0};
  let (state,admitted)=probe([600,300],&[[700,100],[650,120]],action);
  assert!(!admitted);
  assert_eq!(state.menus,vec![([700,100],[700,100]),([650,120],[650,120])]);
  assert!(state.dispatched.is_empty());
  assert_eq!((state.input.pointer,state.input.press_position),([600,300],[0,0]));
  let (state,admitted)=probe([-1,-1],&[[700,100],[300,100]],action);
  assert!(admitted);
  assert_eq!(state.dispatched,vec![[300,100]]);
 }
 #[test] fn walk_locates_terrain_hidden_under_authored_floor_models() {
  let base=[10368,64];let size=[8,8];
  let terrain=1u64<<63|(52u64<<14)|46; // tile (10420,110), plane 0
  let floor=(60001u64<<32)|(52u64<<14)|46; // authored model drawn over it
  let views=vec![0u16;64];
  let visible=vec![floor;64];
  let mut ground=vec![0u64;64];
  for y in 1..7 {for x in 1..7 {ground[y*8+x]=terrain;}}
  let walk=Action::Walk([10420,110]);
  let found=locate(walk,size,(&visible,&views),(&ground,&views),base);
  assert!(found.iter().all(|&(_,tag)|tag==terrain) && !found.is_empty());
  let [x,y]=found[0].0;assert!((1..7).contains(&x) && (1..7).contains(&y));
  assert!(locate(Action::Walk([10421,110]),size,(&visible,&views),(&ground,&views),base).is_empty());
  // Scene targets still require the visible buffer, never covered terrain.
  let object=Action::Object{id:60000,tile:[10420,110],operation:1,sub:0};
  assert_eq!(locate(object,size,(&visible,&views),(&ground,&views),base).first().map(|&(_,tag)|tag),Some(floor));
  assert!(locate(object,size,(&ground,&views),(&visible,&views),base).is_empty());
  let child=vec![1u16;64];
  assert!(locate(walk,size,(&visible,&views),(&ground,&child),base).is_empty());
 }
 #[test] fn scene_targets_require_visible_root_and_exact_coordinates() {
  let tile=[10434,126]; let base=[10368,64]; let xy=(66u64<<14)|62;
  let action=Action::Object{id:64950,tile,operation:1,sub:0};
  let tag=(64951u64<<32)|xy;
  assert!(matches_tag(action,0,tag,base));
  assert!(!matches_tag(action,2,tag,base));
  assert!(!matches_tag(action,0,tag+1,base));
  assert!(!matches_tag(action,0,tag|1<<60,base));
  assert!(matches_tag(Action::Walk(tile),0,1<<63|xy,base));
  assert!(!matches_tag(Action::Walk(tile),0,tag,base));
 }
 #[test] fn native_menu_prepends_cancel_without_admitting_other_actions() {
  let empty=native_menu_entries(Vec::new());
  assert_eq!(empty.len(),1);
  assert!(menu_contains(&empty,Action::Cancel));
  assert!(!menu_contains(&empty,Action::ExamineNpc(12)));
  assert!(!menu_contains(&empty,Action::CastInterface(123)));
  let original=entry(Action::SelectSpell(123));
  let entries=native_menu_entries(vec![original]);
  assert_eq!(entries.len(),2);
  assert_eq!(entries[0].action,Action::Cancel);
  assert_eq!(entries[1].action,Action::SelectSpell(123));
 }
 #[test] fn headless_examine_keeps_root_actor_and_menu_admission() {
  let action=Action::ExamineNpc(12);
  assert!(matches_tag(action,0,(4u64<<60)|12,[0,0]));
  assert!(!matches_tag(action,1,(4u64<<60)|12,[0,0]));
  assert!(!matches_tag(action,0,(4u64<<60)|13,[0,0]));
  assert!(!matches_tag(action,0,12,[0,0]));
  assert!(!menu_contains(&[entry(Action::Npc{index:12,operation:1,sub:0})],action));
  for action in [Action::Cancel,Action::CastInterface(123),action] {
   assert!(!menu_contains(&[],action));
   assert!(menu_contains(&[entry(action)],action));
  }
 }
 #[test] fn menu_membership_cannot_invent_hidden_operations_or_child_views() {
  let action=Action::Npc{index:4,operation:2,sub:0};
  assert!(!menu_contains(&[entry(Action::Npc{index:4,operation:1,sub:0})],action));
  assert!(!menu_contains(&[entry(action.in_view(3))],action));
  let mut parent=entry(Action::Heading(1));parent.submenu=true;parent.children.push(entry(action));
  assert!(menu_contains(&[parent],action));
  let mut blank=entry(action);blank.option.clear();assert!(!menu_contains(&[blank],action));
 }
 #[test] fn covered_first_candidate_is_admitted_at_a_later_visible_point() {
  let action=Action::Object{id:64950,tile:[10434,126],operation:1,sub:0};
  // The interior pixel lies under the chatbox/panel (x>=500); later pixels are clear.
  let (state,admitted,seen)=probe_seen([10,10],&[[620,300],[540,310],[180,320],[150,330]],action);
  assert!(admitted);
  assert_eq!(state.menus.iter().map(|m|m.0).collect::<Vec<_>>(),vec![[620,300],[540,310],[180,320]]);
  assert_eq!(state.dispatched,vec![[180,320]]);
  assert_eq!((state.input.pointer,state.input.press_position),([180,320],[180,320]));
  assert_eq!((state.input.buttons,state.input.pressed_buttons),(0,0));
  assert_eq!(seen.iter().map(|s|s.point).collect::<Vec<_>>(),vec![[620,300],[540,310]]);
 }
 #[test] fn probes_stop_at_exactly_max_total_including_interior() {
  let action=Action::Npc{index:4,operation:1,sub:0};
  // Only the ninth point would be admitted; it must never be probed.
  let mut points:Vec<_>=(0..8).map(|i|[600+i*10,100]).collect();
  points.push([100,100]);points.push([110,100]);
  let (state,admitted,seen)=probe_seen([42,43],&points,action);
  assert!(!admitted);
  assert_eq!(state.menus.len(),MAX_PROBES);
  assert!(state.dispatched.is_empty());
  assert_eq!((state.input.pointer,state.input.press_position),([42,43],[0,0]));
  assert_eq!(seen.len(),SEEN_POINTS);
  for s in &seen {assert_eq!(s.options,vec!["Cancel".to_owned(),"Use".to_owned()]);}
  let (state,admitted,seen)=probe_seen([42,43],&[],action);
  assert!(!admitted && state.menus.is_empty() && seen.is_empty());
  assert_eq!(state.input.pointer,[42,43]);
 }
 #[test] fn refusal_diagnostics_keep_only_leading_labels() {
  let action=Action::Npc{index:4,operation:1,sub:0};
  let mut seen=Vec::new();
  let mut input=crate::input::Input::default();
  let many:Vec<_>=(0..10).map(|i|Entry{option:format!("Op{i}"),target:"<col=ffff00>Guard\"".into(),action:Action::Walk([i,0]),submenu:false,children:Vec::new()}).collect();
  fn same(input:&mut crate::input::Input)->&mut crate::input::Input {input}
  let admitted=admit_first(&mut input,same,&[[700,1]],action,|_,_|Ok(many.clone()),|_,_|Ok(()),&mut seen).unwrap();
  assert!(admitted.is_none());
  assert_eq!(seen.len(),1);
  assert_eq!(seen[0].options.len(),SEEN_OPTIONS);
  assert_eq!(seen[0].options[0],"Cancel");
  assert_eq!(seen[0].options[1],"Op0 <col=ffff00>Guard\"");
 }
 fn admission(stale:[i32;2],points:&[[i32;2]],action:Action)->(Probe,Option<Admission>,Vec<Seen>) {
  let mut state=Probe{input:crate::input::Input::default(),menus:Vec::new(),dispatched:Vec::new()};
  state.input.apply(crate::input::Update::Pointer(stale));
  let mut seen=Vec::new();
  let admitted=admit_first(&mut state,probe_input,points,action,
   |s,point|{s.menus.push((point,s.input.pointer));Ok(if point[0]<500 {vec![entry(action)]} else {vec![entry(Action::Walk([1,2]))]})},
   |s,point|{s.dispatched.push(point);Ok(())},&mut seen).unwrap();
  (state,admitted,seen)
 }
 #[test] fn admission_reports_candidate_index_point_and_preceding_refusals() {
  let action=Action::Object{id:909,tile:[10418,105],operation:1,sub:0};
  // Interior p0 under a panel, p1 under a panel, p2 visible.
  let (state,admitted,seen)=admission([10,10],&[[571,195],[560,170],[176,165],[150,160]],action);
  assert_eq!(admitted,Some(Admission{index:2,point:[176,165]}));
  assert_eq!(state.dispatched,vec![[176,165]]);
  assert_eq!(seen.iter().map(|s|s.point).collect::<Vec<_>>(),vec![[571,195],[560,170]]);
  for s in &seen {assert_eq!(s.options,vec!["Cancel".to_owned(),"Use".to_owned()]);}
  // Preceding refusals stay bounded even when admission happens late.
  let mut points:Vec<_>=(0..7).map(|i|[600+i*10,100]).collect();points.push([100,100]);
  let (state,admitted,seen)=admission([10,10],&points,action);
  assert_eq!(admitted,Some(Admission{index:7,point:[100,100]}));
  assert_eq!(state.menus.len(),MAX_PROBES);
  assert_eq!(seen.len(),SEEN_POINTS);
  assert_eq!(seen[0].point,[600,100]);
 }
 #[test] fn first_point_admission_has_index_zero_and_no_refusals() {
  let action=Action::Npc{index:4,operation:1,sub:0};
  let (state,admitted,seen)=admission([600,300],&[[200,150],[210,150]],action);
  assert_eq!(admitted,Some(Admission{index:0,point:[200,150]}));
  assert!(seen.is_empty());
  assert_eq!(state.menus.len(),1);
 }
 #[test] fn refusal_and_errors_produce_no_admission() {
  let action=Action::Npc{index:4,operation:1,sub:0};
  let (_,admitted,seen)=admission([600,300],&[[700,100],[650,120]],action);
  assert_eq!(admitted,None);
  assert_eq!(seen.len(),2);
  fn same(input:&mut crate::input::Input)->&mut crate::input::Input {input}
  let mut input=crate::input::Input::default();
  let result=admit_first(&mut input,same,&[[300,100]],action,
   |_,_|Ok(vec![entry(action)]),|_,_|Err(crate::Error::Invalid("dispatch failed")),&mut Vec::new());
  assert!(result.is_err(),"a failed dispatch is an error, never an admission");
  let result=admit_first(&mut input,same,&[[300,100]],action,
   |_,_|Err(crate::Error::Invalid("menu failed")),|_,_|panic!("no dispatch"),&mut Vec::new());
  assert!(result.is_err());
 }
 #[test] fn menu_or_dispatch_errors_restore_pointer_and_press_position() {
  let action=Action::Npc{index:4,operation:1,sub:0};
  let mut input=crate::input::Input::default();
  input.apply(crate::input::Update::ButtonAt(0,true,[11,12]));
  input.apply(crate::input::Update::Pointer([600,300]));
  fn same(input:&mut crate::input::Input)->&mut crate::input::Input {input}
  // The second probe's menu fails after the pointer has moved twice.
  let mut calls=0;
  let result=admit_first(&mut input,same,&[[700,100],[650,120],[300,100]],action,
   |_,_|{calls+=1;if calls==2 {Err(crate::Error::Invalid("menu failed"))} else {Ok(Vec::new())}},
   |_,_|panic!("nothing admitted"),&mut Vec::new());
  assert!(result.is_err());
  assert_eq!(calls,2);
  assert_eq!((input.pointer,input.press_position),([600,300],[11,12]));
  // An admitted probe whose dispatch fails is not an accepted action either.
  let result=admit_first(&mut input,same,&[[300,100]],action,
   |_,_|Ok(vec![entry(action)]),|_,_|Err(crate::Error::Invalid("dispatch failed")),&mut Vec::new());
  assert!(result.is_err());
  assert_eq!((input.pointer,input.press_position),([600,300],[11,12]));
 }

 fn script(ops:&[(u16,i32)])->crate::script::Script {
  crate::script::Script{name:None,local_ints:0,local_strings:0,local_longs:0,int_arguments:0,string_arguments:0,long_arguments:0,switches:vec![],
   instructions:ops.iter().map(|&(opcode,n)|crate::script::Instruction{opcode,operand:crate::script::Operand::Int(n)}).collect()}
 }
 #[test] fn only_the_direct_close_listener_is_a_close_control() {
  use crate::vm::Value;
  // [clientscript,closebutton_click] (29) decoded from the cache: if_close; return.
  let close=script(&[(3103,0),(21,0)]);
  assert!(close_only(&[Value::Int(29)],&close));
  assert!(!close_only(&[Value::Int(29),Value::Int(1)],&close),"extra listener argument");
  assert!(!close_only(&[],&close));
  let mut with_args=script(&[(3103,0),(21,0)]);with_args.int_arguments=1;with_args.local_ints=1;
  assert!(!close_only(&[Value::Int(29)],&with_args));
  let mut with_locals=script(&[(3103,0),(21,0)]);with_locals.local_strings=1;
  assert!(!close_only(&[Value::Int(29)],&with_locals));
  let mut with_switch=script(&[(3103,0),(21,0)]);with_switch.switches.push(Default::default());
  assert!(!close_only(&[Value::Int(29)],&with_switch));
  assert!(!close_only(&[Value::Int(1)],&script(&[(0,5),(2,0),(3103,0),(21,0)])),"varp side effect");
  assert!(!close_only(&[Value::Int(1)],&script(&[(40,26),(3103,0),(21,0)])),"gosub");
  assert!(!close_only(&[Value::Int(1)],&script(&[(1,0),(0,1),(7,1),(3103,0),(21,0)])),"conditional");
  assert!(!close_only(&[Value::Int(1)],&script(&[(21,0),(3103,0)])),"unreachable");
  assert!(!close_only(&[Value::Int(1)],&script(&[(3103,0),(3103,0)])),"double close");
  assert!(!close_only(&[Value::Int(1)],&script(&[(21,0)])));
 }
}
