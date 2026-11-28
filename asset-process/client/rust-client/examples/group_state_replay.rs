//! Actual-cache group definitions, packet dispatch and script query integration.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    game_packet,
    group_state::Value,
    protocol::Packet,
    vm::{Host, Object, Stacks},
    writer::Writer,
    Result,
};
fn apply(c: &mut ClientState, opcode: u16, payload: Vec<u8>) -> Result<()> {
    c.apply(game_packet::decode(&Packet { opcode, payload })?.unwrap())
}
fn query(c: &mut ClientState, opcode: u16, key: i32, identity: i64) -> Result<Value> {
    let mut s = Stacks::default();
    s.push_int(key)?;
    s.push_long(identity)?;
    c.extended(opcode, false, &mut s)?;
    Ok(match (key as u32 >> 16) & 3 {
        0 => Value::Int(s.pop_int()?),
        1 => Value::Long(s.pop_long()?),
        _ => {
            let Object::String(v) = s.pop_object()? else {
                panic!("query string")
            };
            Value::String(v)
        }
    })
}
fn write(w: &mut Writer, v: &Value, compact: bool) -> Result<()> {
    match v {
        Value::Int(v) => {
            if compact {
                let mut n = ((*v as u32) << 1) ^ ((*v >> 31) as u32);
                while n > 127 {
                    w.u8((n as u8 & 127) | 128);
                    n >>= 7;
                }
                w.u8(n as u8);
            } else {
                w.u32(*v as u32);
            }
        }
        Value::Long(v) => w.u64(*v as u64),
        Value::String(v) => w.string(v)?,
        Value::Null => {}
    }
    Ok(())
}
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    c.groups.load_definitions(&mut c.scripts.source.cache)?;
    let cache_definitions = c.groups.definitions.len();
    if cache_definitions == 0 {
        let mut r = gielinor_client::buffer::Reader::new(include_bytes!(
            "../tests/fixtures/group-state-jvm.bin"
        ));
        for id in 0..r.u32()? {
            let len = r.u32()? as usize;
            c.groups.definitions.insert(
                id as u16,
                gielinor_client::group_state::Definition::decode(r.bytes(len)?)?,
            );
        }
    }
    let definitions = c.groups.definitions.clone();
    let mut variables = 0;
    for (&id, def) in &definitions {
        let uid = 123456 + id as i64;
        let slot = 255;
        let mut w = Writer::new();
        w.u8(slot);
        w.unsigned_smart(id + 1)?;
        w.u64(uid as u64);
        for values in &def.defaults {
            for v in values {
                write(&mut w, v, true)?;
            }
        }
        apply(&mut c, 132, w.into_vec())?;
        assert_eq!(c.groups.slots[&slot].values, def.defaults);
        for scope in 0..2 {
            for (index, default) in def.defaults[scope].iter().enumerate() {
                let kind = def.kinds[scope][index];
                let key = ((id as u32) << 20)
                    | ((scope as u32) << 18)
                    | ((kind as u32) << 16)
                    | index as u32;
                assert_eq!(
                    &query(
                        &mut c,
                        if scope == 0 { 8560 } else { 8570 },
                        key as i32,
                        -999
                    )?,
                    default
                );
                let value = match kind {
                    0 => Value::Int(-1234567),
                    1 => Value::Long(-9876543210),
                    _ => Value::String("Group replay".into()),
                };
                let mut w = Writer::new();
                w.u8(slot);
                w.u32(key);
                write(&mut w, &value, false)?;
                apply(
                    &mut c,
                    match kind {
                        0 => 140,
                        1 => 133,
                        _ => 136,
                    },
                    w.into_vec(),
                )?;
                assert_eq!(
                    query(
                        &mut c,
                        if scope == 0 { 8560 } else { 8570 },
                        key as i32,
                        uid
                    )?,
                    value
                );
                variables += 1;
            }
        }
        apply(&mut c, 132, vec![slot, 0])?;
        assert!(!c.groups.slots.contains_key(&slot));
    }
    assert!(c.script_failures.is_empty(), "group callbacks: {:?}", c.script_failures);
    // Reconnect response15 preserves group membership; fresh login clears it.
    c.groups.slots.insert(
        3,
        gielinor_client::group_state::Group {
            definition: 0,
            identity: 91,
            values: Default::default(),
        },
    );
    c.reconnect_players(&vec![0; 4608], 42)?;
    assert_eq!(c.groups.slots[&3].identity, 91);
    c.prepare_login()?;
    assert!(c.groups.slots.is_empty());
    println!("Group client replay: {cache_definitions} cached definitions, {} exercised definitions, {variables} typed variables, packet/script queries and reconnect/login lifetime passed",definitions.len());
    Ok(())
}
