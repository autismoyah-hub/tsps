//! Actual ClientState packet/script/listener integration with the local cache.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    game_packet,
    protocol::Packet,
    vm::{Host, Stacks, Value},
    widget::Widget,
    writer::Writer,
    Result,
};
fn apply(c: &mut ClientState, op: u16, payload: Vec<u8>) -> Result<()> {
    c.apply(
        game_packet::decode(&Packet {
            opcode: op,
            payload,
        })?
        .expect("market packet"),
    )
}
fn query(c: &mut ClientState, op: u16, args: &[i32]) -> Result<Vec<i32>> {
    let mut s = Stacks::default();
    for &v in args {
        s.push_int(v)?;
    }
    c.extended(op, false, &mut s)?;
    Ok(s.ints().to_vec())
}
fn events(c: &mut ClientState) -> Result<Vec<i32>> {
    let layout = c.interfaces.layout([765, 503])?;
    Ok(c.interface_events
        .collect(&c.interfaces, &layout, &c.input, false, false)
        .iter()
        .map(|e| {
            if let Value::Int(v) = e.arguments[0] {
                v
            } else {
                panic!("listener")
            }
        })
        .collect())
}
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        std::env::args().nth(1).expect("cache path"),
        64 << 20,
    )?)?;
    c.data.world = 302;
    for id in [0, 1277, 4151, 4152, 995] {
        let item = &c.items.items[&(id as u32)];
        let expected = [item.stockmarket as i32, item.searchable as i32];
        assert_eq!(query(&mut c, 3931, &[id])?, [expected[0]]);
        assert_eq!(query(&mut c, 3939, &[id])?, [expected[1]]);
    }
    let mut widget = Widget::empty(0, 0);
    widget.raw_size = [100, 100];
    c.interfaces.install_group(0, vec![widget])?;
    c.interfaces.root = Some(0);
    for op in [1425, 1426] {
        c.interfaces
            .runtime
            .get_mut(&0)
            .unwrap()
            .listeners
            .insert(op, (vec![Value::Int(op as i32)], None));
    }
    let mut w = Writer::new();
    w.u8(2);
    w.u8(10);
    w.u16(4151);
    for v in [123456, 7, 3, 370368] {
        w.u32(v);
    }
    apply(&mut c, 53, w.into_vec())?;
    assert_eq!(query(&mut c, 3903, &[2])?, [1]);
    assert_eq!(query(&mut c, 3904, &[2])?, [4151]);
    assert_eq!(query(&mut c, 3907, &[2])?, [3]);
    assert_eq!(query(&mut c, 3911, &[2])?, [1]);
    let mut w = Writer::new();
    w.u8(1);
    w.u64(1700000000000);
    w.u16(4151);
    w.u8(0);
    w.u16(3);
    for (name, world, price) in [("Rune", 301, 100), ("rune", 302, 200), ("Rune", 303, 100)] {
        w.string(name)?;
        w.string("Previous")?;
        w.u16(world);
        w.u64(1699999999000);
        w.u32(price);
        w.u32(7);
    }
    apply(&mut c, 56, w.into_vec())?;
    assert_eq!(events(&mut c)?, [1425, 1426]);
    assert!(events(&mut c)?.is_empty());
    assert_eq!(query(&mut c, 3919, &[])?, [3]);
    query(&mut c, 3916, &[1, 1])?;
    assert_eq!(query(&mut c, 3920, &[0])?, [302]);
    query(&mut c, 3915, &[1])?;
    assert_eq!(query(&mut c, 3920, &[0])?, [301]);
    assert_eq!(query(&mut c, 3920, &[1])?, [303]);
    let mut clear = vec![0x55; 20];
    clear[0] = 2;
    clear[1] = 0;
    apply(&mut c, 53, clear)?;
    assert_eq!(query(&mut c, 3910, &[2])?, [1]);
    assert_eq!(query(&mut c, 3904, &[2])?, [0]);
    apply(&mut c, 56, vec![0])?;
    assert_eq!(query(&mut c, 3919, &[])?, [0]);
    assert_eq!(events(&mut c)?, [1425, 1426]);
    c.market.offers[1].item = 4151;
    c.prepare_login()?;
    assert!(c.market.offers.iter().all(|v| v.item == 0));
    assert!(c.market.listings.is_none());
    println!("Market packet storage, host queries, stable/current-world sorts, coalesced listeners, clearing and login reset passed with real cache");
    Ok(())
}
