use gielinor_client::*;
use interface_state::Interfaces;
use interface_packet::{SubInterface, Update};
use widget::Widget;
fn main() -> Result<()> {
    let mut cache = disk_cache::DiskCache::open("work/kotlin-world/.data/cache/js5",64<<20)?;
    let items = item::Definitions { items: Default::default() };
    let mut state = Interfaces::default();
    let parent = 65534u32 << 16;
    state.install_group(65534, vec![Widget::empty(parent,0)])?;
    state.install_group(65533, vec![Widget::empty(65533u32<<16,4)])?;
    state.root = Some(65534);
    state.runtime.get_mut(&parent).unwrap().listeners.insert(1424,(vec![vm::Value::Int(999)],None));
    state.children.insert(parent,SubInterface {parent, group:65533, kind:0});
    state.apply(&mut cache,&items,&mut Default::default(),Update::OpenSub(SubInterface {parent,group:162,kind:0}))?;
    assert!(!state.children.contains_key(&parent),"replacement must await close callback");
    assert!(!state.widgets.keys().any(|id| id>>16 == 162),"group loaded before callback");
    let event = state.next_event_with_cache(&mut cache)?.expect("close callback");
    assert!(matches!(event.arguments[0], vm::Value::Int(999)));
    assert!(state.next_event().is_err(), "cacheless drain must report a pending load");
    assert!(!state.children.contains_key(&parent));
    let mut callbacks = 1;
    while state.next_event_with_cache(&mut cache)?.is_some() {callbacks += 1;}
    assert_eq!(state.children[&parent].group,162);
    let count = state.runtime.values().filter(|r|r.logical_id>>16 == 162).count();
    assert!(count > 0);
    state.layout([765,503])?;
    println!("Deferred actual-cache group162 load: {count} widgets, {callbacks} callbacks, mounted layout valid");
    Ok(())
}
