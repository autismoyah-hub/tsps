use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    player_appearance::Update,
    player_info::{State, View},
    player_model::Appearance,
    vm::{Host, Object, Stacks},
    Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut c = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/appearance-string-jvm.bin"
    ));
    assert_eq!(r.u32()?, 72);
    for _ in 0..72 {
        let view = r.u32()? as u16;
        let condition = r.u32()?;
        let extra = r.u32()? as i32 as i8;
        let null_at = r.u32()? as i32;
        let null = r.u8()? != 0;
        let expected = if null {
            None
        } else {
            let n = r.u16()? as usize;
            Some(String::from_utf8(r.bytes(n)?.to_vec()).unwrap())
        };
        let query = r.u32()? as i32;
        let objects = r.u32()? as usize;
        let ints = r.u32()? as usize;
        let (mut state, _) = State::initialize(&vec![0; 4608], 3)?;
        let mut player = state.players[3].take().unwrap();
        if condition > 2 {
            player.appearance = Some(Update {
                appearance: Appearance {
                    extra,
                    ..Default::default()
                },
                skull: -1,
                overhead: -1,
                animations: [-1; 7],
                name: String::new(),
                combat: 1,
                skill: 0,
                hidden: false,
                force_refresh: false,
                actions: Default::default(),
                team: 0,
            });
        }
        if view == 0 {
            if condition > 1 {
                state.players[3] = Some(player);
            }
        } else if condition > 0 {
            let mut child = View {
                base: [0, 0],
                dimensions: [104, 104],
                players: Default::default(),
            };
            if condition > 1 {
                child.players.insert(3, player);
            }
            state.views.insert(view, child);
        }
        state.local_view = view;
        c.player_state = Some(state);
        c.active_world = if view == 0 { 7 } else { 0 };
        let mut s = Stacks::default();
        for (i, text) in ["first", "second", "third"].iter().enumerate() {
            s.push_object(if i as i32 == null_at {
                Object::Null
            } else {
                Object::String((*text).into())
            })?;
        }
        c.extended(4123, false, &mut s)?;
        c.extended(4124, false, &mut s)?;
        assert_eq!(s.objects().len(), objects);
        assert_eq!(s.ints().len(), ints);
        assert_eq!(s.ints()[0], query);
        match (&s.objects()[0], expected) {
            (Object::Null, None) => (),
            (Object::String(actual), Some(expected)) => assert_eq!(actual, &expected),
            _ => panic!("selection mismatch"),
        }
    }
    for (opcode, value, selector, expected) in [
        (4125, i64::MIN, None, "-9223372036854775808"),
        (4126, 1234567, Some("x"), "1,234,567"),
        (4126, i64::MIN, Some(","), "-"),
    ] {
        let mut s = Stacks::default();
        s.push_long(value)?;
        if let Some(selector) = selector {
            s.push_object(Object::String(selector.into()))?;
        }
        c.extended(opcode, false, &mut s)?;
        assert!(matches!(&s.objects()[0],Object::String(v) if v==expected));
    }
    let mut parsed=Stacks::default();
    parsed.push_int(51)?;parsed.push_int(89)?;parsed.pop_int()?;
    parsed.push_object(Object::String("bad".into()))?;
    c.extended(4127,false,&mut parsed)?;
    assert_eq!(parsed.ints(),[51,89,0,0]);
    let mut parsed=Stacks::default();parsed.push_object(Object::String("-2147483648".into()))?;
    c.extended(4127,false,&mut parsed)?;assert_eq!(parsed.ints(),[i32::MIN,1]);
    println!("ClientState4127 failed-parse retained slot and signed boundary dispatch pass");
    println!("Actual-cache ClientState:72 original appearance cases through4123/4124 with divergent packet/local views;4125/4126 extrema and grouping dispatch pass");
    Ok(())
}
