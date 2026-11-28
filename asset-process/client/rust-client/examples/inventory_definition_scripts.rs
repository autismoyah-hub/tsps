use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    inventory_def::Definition,
    vm::{Host, Stacks},
    Result,
};
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let definitions = client.scripts.source.cache.group(2, 5, [0; 4])?;
    let count = definitions.len();
    for (id, bytes) in definitions {
        let expected = Definition::decode(&bytes)?.capacity;
        let mut s = Stacks::default();
        s.push_int(id as i32)?;
        client.extended(3304, false, &mut s)?;
        assert_eq!(s.ints(), [expected]);
    }
    for (op, args) in [(3304, vec![i32::MAX]), (6528, vec![i32::MAX, 0x00ffffff])] {
        let mut s = Stacks::default();
        for v in args {
            s.push_int(v)?;
        }
        client.extended(op, false, &mut s)?;
        assert_eq!(s.ints(), [0]);
    }
    println!("{count} cached inventory capacity routes plus absent inventory/parameter defaults passed through ClientState");
    Ok(())
}
