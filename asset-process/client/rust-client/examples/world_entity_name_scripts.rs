//! Query every cached child-world name through the native script host.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    vm::{Host, Object, Stacks},
    world_entity::Definition,
    Result,
};
fn query(client: &mut ClientState, id: i32) -> Result<String> {
    let mut stacks = Stacks::default();
    stacks.push_int(id)?;
    client.extended(3339, false, &mut stacks)?;
    match stacks.pop_object()? {
        Object::String(name) => Ok(name),
        _ => panic!("entity name"),
    }
}
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let records = client.scripts.source.cache.group(2, 72, [0; 4])?;
    let mut expected = Vec::new();
    for (id, bytes) in records {
        let name = Definition::decode(id as i32, &bytes)?.name;
        assert_eq!(query(&mut client, id as i32)?, name);
        assert_eq!(query(&mut client, id as i32)?, name);
        expected.push((id, name));
    }
    assert_eq!(query(&mut client, -1)?, "");
    assert_eq!(query(&mut client, i32::MAX)?, "null");
    client.prepare_login()?;
    for (id, name) in &expected {
        assert_eq!(query(&mut client, *id as i32)?, *name);
    }
    println!("{} child-world configuration names, cached repeats, absent sentinel/default and login reset passed",expected.len());
    Ok(())
}
