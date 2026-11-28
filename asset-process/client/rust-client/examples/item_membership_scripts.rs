//! World-context item names/actions/parameters through the actual ClientState script host.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    object::Param,
    param_def,
    vm::{Host, Object, Stacks, Value},
    Result,
};
fn name(client: &mut ClientState, id: u32) -> Result<String> {
    let mut s = Stacks::default();
    s.push_int(id as i32)?;
    client.extended(4200, false, &mut s)?;
    match s.pop_object()? {
        Object::String(s) => Ok(s),
        _ => panic!("item name"),
    }
}
fn query(client: &mut ClientState, id: u32, param: u32) -> Result<Value> {
    let bytes = client.scripts.source.cache.file(2, 11, param, [0; 4])?;
    let meta = param_def::Definition::decode(&bytes)?;
    let mut s = Stacks::default();
    s.push_int(id as i32)?;
    s.push_int(param as i32)?;
    client.extended(6515, false, &mut s)?;
    Ok(match meta.default {
        Value::Int(_) => Value::Int(s.pop_int()?),
        Value::Long(_) => Value::Long(s.pop_long()?),
        Value::Object(_) => Value::Object(s.pop_object()?),
    })
}
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Long(a), Value::Long(b)) => a == b,
        (Value::Object(Object::String(a)), Value::Object(Object::String(b))) => a == b,
        (Value::Object(Object::Null), Value::Object(Object::Null)) => true,
        _ => false,
    }
}
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    assert!(!client.data.members);
    client.set_members(true)?;
    let (id, original_name, params) = client
        .items
        .items
        .iter()
        .filter(|(_, d)| {
            d.members
                && d.note[1] == -1
                && d.bought[1] == -1
                && d.placeholder[1] == -1
                && !d.params.is_empty()
        })
        .min_by_key(|(id, _)| *id)
        .map(|(id, d)| (*id, d.name.clone(), d.params.clone()))
        .expect("cache member item with parameters");
    assert_eq!(name(&mut client, id)?, original_name);
    let mut expected = Vec::new();
    for (key, param) in params {
        let meta =
            param_def::Definition::decode(&client.scripts.source.cache.file(2, 11, key, [0; 4])?)?;
        let value = match param {
            Param::Int(n) => Value::Int(n),
            Param::Long(n) => Value::Long(n),
            Param::String(s) => Value::Object(Object::String(s)),
        };
        assert!(same(&query(&mut client, id, key)?, &value));
        expected.push((
            key,
            if meta.auto_disable {
                meta.default
            } else {
                value
            },
        ));
    }
    client.set_members(false)?;
    assert_eq!(name(&mut client, id)?, format!("{original_name} (Members)"));
    for (key, value) in &expected {
        assert!(same(&query(&mut client, id, *key)?, value));
    }
    client.prepare_login()?;
    assert!(!client.data.members);
    assert_eq!(name(&mut client, id)?, format!("{original_name} (Members)"));
    client.set_members(true)?;
    assert_eq!(name(&mut client, id)?, original_name);
    client.prepare_login()?;
    assert!(client.data.members);
    assert_eq!(name(&mut client, id)?, original_name);
    println!("Item {id} member/free/member names and {} parameter routes survive ClientState login resets",expected.len());
    Ok(())
}
