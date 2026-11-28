//! Offline packet-to-runtime integration against a real cache; no server or account mutation.
use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    game_packet,
    protocol::Packet,
    vm::{Host, Stacks},
    writer::Writer,
    Result,
};
fn apply(c: &mut ClientState, op: u16, payload: Vec<u8>) -> Result<()> {
    c.apply(
        game_packet::decode(&Packet {
            opcode: op,
            payload,
        })?
        .expect("implemented packet"),
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
fn frame(r: &mut Reader<'_>) -> Vec<u8> {
    let n = r.u32().unwrap() as usize;
    r.bytes(n).unwrap().to_vec()
}
fn slotted(raw: &[u8]) -> Vec<u8> {
    let mut p = vec![0];
    p.extend_from_slice(raw);
    p
}
fn bits(values: &[(u32, u8)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut bit = 0;
    for &(value, n) in values {
        for shift in (0..n).rev() {
            if bit % 8 == 0 {
                out.push(0);
            }
            out[bit / 8] |= (((value >> shift) & 1) as u8) << (7 - bit % 8);
            bit += 1;
        }
    }
    out
}
fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("cache path");
    let mut c = ClientState::load(DiskCache::open(path, 64 << 20)?)?;
    let elements = gielinor_client::map_element::Definition::load_all(&mut c.scripts.source.cache)?;
    println!(
        "Decoded {} map elements, {} visible on minimap",
        elements.len(),
        elements.values().filter(|e| e.minimap).count()
    );
    let assets = gielinor_client::minimap::Assets::load(&mut c.scripts.source.cache)?;
    println!(
        "Loaded minimap compass {:?} and {} marker sprites from graphics defaults",
        assets.compass.canvas,
        assets.dots.len()
    );
    let mut hints = Reader::new(include_bytes!("../tests/fixtures/hints-jvm.bin"));
    for case in 0..hints.u32()? {
        if hints.u8()? != 0 {
            c.hint.kind = 0;
        }
        c.data.position[0] = hints.u32()? as i32;
        apply(&mut c, 50, hints.bytes(6)?.to_vec())?;
        let h = &c.hint;
        let actual = [
            h.kind as i32,
            h.index as i32,
            h.tile[0] as i32,
            h.tile[1] as i32,
            h.offset[0],
            h.offset[1],
            h.height as i32,
            h.plane,
        ];
        let expected: [i32; 8] = std::array::from_fn(|_| hints.u32().unwrap() as i32);
        assert_eq!(actual, expected, "runtime hint {case}");
        hints.u32()?;
    }
    assert_eq!(hints.remaining(), 0);
    println!("Verified 1024 hint packets through ClientState; loaded {} destination/hint sprites and edge {:?}",assets.flags.len(),assets.edge.canvas);
    apply(&mut c, 36, vec![])?;
    assert_eq!(query(&mut c, 3600, &[])?, [-1]);
    let mut r = Reader::new(include_bytes!("../tests/fixtures/friends-jvm.bin"));
    r.u32()?;
    apply(&mut c, 111, frame(&mut r))?;
    assert_eq!(query(&mut c, 3600, &[])?, [c.social.friends.len() as i32]);
    let mut w = Writer::new();
    w.u8(0);
    w.string("Alice")?;
    w.string("OldAlice")?;
    w.string("")?;
    apply(&mut c, 42, w.into_vec())?;
    let mut w = Writer::new();
    w.unsigned_smart(6)?;
    w.u8(1);
    w.string("OldAlice")?;
    w.string("Filtered")?;
    let message = w.into_vec();
    apply(&mut c, 74, message.clone())?;
    assert!(c.game_messages.is_empty());
    let mut w = Writer::new();
    w.u8(4);
    w.string("Alice")?;
    apply(&mut c, 42, w.into_vec())?;
    apply(&mut c, 74, message)?;
    assert_eq!(c.game_messages.len(), 1);
    assert_eq!(c.chat.count(6), 1);
    let mut r = Reader::new(include_bytes!("../tests/fixtures/clan-delta-jvm.bin"));
    let initial = frame(&mut r);
    r.u32()?;
    let delta = frame(&mut r);
    apply(&mut c, 81, slotted(&initial))?;
    assert_eq!(query(&mut c, 3801, &[0])?, [1]);
    let count = query(&mut c, 3809, &[])?[0];
    apply(&mut c, 71, slotted(&delta))?;
    assert_eq!(query(&mut c, 3809, &[])?, [count + 1]);
    apply(&mut c, 71, slotted(&delta))?;
    assert!(c.outbound.is_empty());
    let mut future = delta.clone();
    future[8..12].copy_from_slice(&3i32.to_be_bytes());
    apply(&mut c, 71, slotted(&future))?;
    let p = c.outbound.pop_front().unwrap();
    assert_eq!((p.opcode, p.payload), (2, vec![0]));
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/clan-channel-delta-jvm.bin"
    ));
    let initial = frame(&mut r);
    r.u32()?;
    let delta = frame(&mut r);
    apply(&mut c, 69, slotted(&initial))?;
    assert_eq!(query(&mut c, 3851, &[0])?, [1]);
    let count = query(&mut c, 3855, &[])?[0];
    apply(&mut c, 95, slotted(&delta))?;
    assert_eq!(query(&mut c, 3855, &[])?, [count + 1]);
    apply(&mut c, 95, slotted(&delta))?;
    assert!(c.outbound.is_empty());
    let mut future = delta.clone();
    future[8..16].copy_from_slice(&3u64.to_be_bytes());
    apply(&mut c, 95, slotted(&future))?;
    let p = c.outbound.pop_front().unwrap();
    assert_eq!((p.opcode, p.payload), (112, vec![0]));
    // A clear snapshot must not invalidate the already selected object.
    apply(&mut c, 69, vec![0])?;
    assert_eq!(query(&mut c, 3851, &[0])?, [0]);
    assert_eq!(query(&mut c, 3855, &[])?, [count + 1]);
    let mut w = Writer::new();
    w.u64(0);
    w.u32(1);
    w.u8(8);
    w.u32(7);
    w.u32(123);
    w.u8(0);
    apply(&mut c, 71, slotted(&w.into_vec()))?;
    assert!(matches!(
        c.read(gielinor_client::vm::Variable::ClanSetting, 7)?,
        gielinor_client::vm::Value::Int(123)
    ));
    apply(&mut c, 69, slotted(&initial))?;
    let mut mute = Stacks::default();
    for v in [0, 1, 0] {
        mute.push_int(v)?;
    }
    assert!(c.extended(3821, false, &mut mute).is_err());
    assert_eq!(c.outbound.pop_front().unwrap().opcode, 96);
    assert_eq!(query(&mut c, 3890, &[])?, [0]);
    apply(&mut c, 88, vec![])?;
    assert_eq!(query(&mut c, 3890, &[])?, [1]);
    let definitions = c.scripts.source.cache.group(2, 47, [0; 4])?;
    let id = definitions
        .iter()
        .find_map(|(id, raw)| {
            (gielinor_client::clan_profile::Definition::decode(raw)
                .ok()?
                .kind
                == Some(0))
            .then_some(*id)
        })
        .expect("cache has an integer clan profile definition");
    assert!(matches!(
        c.read(gielinor_client::vm::Variable::Clan, id as i32)?,
        gielinor_client::vm::Value::Int(0)
    ));
    let mut w = Writer::new();
    w.u16(id as u16);
    w.u32(123456);
    apply(&mut c, 11, w.into_vec())?;
    assert!(matches!(
        c.read(gielinor_client::vm::Variable::Clan, id as i32)?,
        gielinor_client::vm::Value::Int(123456)
    ));
    apply(&mut c, 60, vec![])?;
    assert_eq!(query(&mut c, 3890, &[])?, [0]);
    assert!(c
        .read(gielinor_client::vm::Variable::Clan, id as i32)
        .is_err());
    // Send opcodes use the real cache codec and queue native packets without a socket.
    let lengths = c.scripts.source.cache.named_file(10, "huffman", "")?;
    let codec = gielinor_client::huffman::Huffman::new(&lengths)?;
    for op in [5008, 5009, 5010] {
        let mut stack = Stacks::default();
        if op == 5009 {
            stack.push_object(gielinor_client::vm::Object::String("Recipient".into()))?;
        }
        stack.push_object(gielinor_client::vm::Object::String("red:wave:Hello".into()))?;
        if op != 5009 {
            stack.push_int(3)?;
        }
        if op == 5010 {
            stack.push_int(2)?;
        }
        c.extended(op, false, &mut stack)?;
        let expected = if op == 5009 {
            gielinor_client::chat_packet::private_message("Recipient", "red:wave:Hello", &codec)?
        } else {
            gielinor_client::chat_packet::public_message(
                3,
                "red:wave:Hello",
                if op == 5010 { 2 } else { -1 },
                &codec,
            )?
        };
        assert_eq!(c.outbound.pop_front().unwrap(), expected);
        assert!(stack.ints().is_empty());
    }
    let echo = gielinor_client::chat_packet::private_message("Recipient", "hELLO <world>", &codec)?;
    apply(&mut c, 15, echo.payload)?;
    let message = c.chat.line(6, 0).unwrap();
    assert_eq!(
        (&*message.name, &*message.text),
        ("Recipient", "Hello <lt>world<gt>")
    );
    let mut incoming = Writer::new();
    incoming.string("Sender")?;
    incoming.u16(1);
    incoming.u24(123)?;
    incoming.u8(1);
    let compressed =
        gielinor_client::chat_packet::private_message("", "a PRIVATE message", &codec)?;
    incoming.bytes(&compressed.payload[1..]);
    let incoming = incoming.into_vec();
    apply(&mut c, 19, incoming.clone())?;
    apply(&mut c, 19, incoming)?;
    assert_eq!(c.chat.count(7), 1);
    assert_eq!(c.chat.line(7, 0).unwrap().name, "<img=0>Sender");
    // A full player-info packet delivers public chat before a later forced-speech block.
    let mut init = vec![((3200 << 14) | 3200, 30)];
    init.extend(std::iter::repeat_n((0, 18), 2046));
    c.initialize_players(&bits(&init), 1)?;
    c.player_state.as_mut().unwrap().players[1]
        .as_mut()
        .unwrap()
        .appearance = Some(gielinor_client::player_appearance::Update {
        appearance: Default::default(),
        skull: -1,
        overhead: -1,
        animations: [-1; 7],
        name: "Speaker".into(),
        combat: 3,
        skill: 0,
        hidden: false,
        force_refresh: false,
        actions: Default::default(),
        team: 0,
    });
    let mut wire = bits(&[(1, 1), (1, 1), (0, 2)]);
    wire.extend(bits(&[(0, 1), (3, 2), (2045, 11)]));
    let mut block = Writer::new();
    block.u8(12);
    block.u8(1);
    block.u16_add(0, false);
    block.transformed(0, gielinor_client::buffer::Transform::Neg);
    block.transformed(0, gielinor_client::buffer::Transform::Add);
    let compressed =
        gielinor_client::chat_packet::private_message("", "hELLO <public>", &codec)?.payload;
    block.transformed(
        (compressed.len() - 1) as u8,
        gielinor_client::buffer::Transform::Neg,
    );
    block.bytes(&compressed[1..].iter().rev().copied().collect::<Vec<_>>());
    block.string("~Forced text")?;
    wire.extend(block.into_vec());
    let destination = ((c.data.position[1] as u32) << 14) | c.data.position[2] as u32;
    apply(&mut c, 143, destination.to_be_bytes().to_vec())?;
    assert_eq!(c.map_flag, Some(destination));
    apply(&mut c, 28, wire)?;
    assert!(
        c.map_flag.is_none(),
        "destination cleared when local player reaches it"
    );

    assert_eq!(c.chat.count(2), 2);
    assert_eq!(c.chat.line(2, 0).unwrap().text, "Forced text");
    assert_eq!(c.chat.line(2, 1).unwrap().text, "Hello <lt>public<gt>");
    assert_eq!(c.chat.line(2, 1).unwrap().name, "Speaker");
    println!("Verified public and forced-player chat from wire packet 28 through ClientState in original block order");
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/operation-control-jvm.bin"
    ));
    for case in 0..r.u32()? {
        let op = r.u16()?;
        let payload = frame(&mut r);
        apply(&mut c, op, payload)?;
        for i in 0..3 {
            assert_eq!(c.hidden_operations[i], r.u8()? != 0, "hide op case {case}");
        }
        for i in 0..8 {
            let length = r.u32()? as i32;
            let label = if length < 0 {
                None
            } else {
                Some(String::from_utf8(r.bytes(length as usize)?.to_vec()).unwrap())
            };
            assert_eq!(c.player_operations[i], label, "label case {case}");
            assert_eq!(
                c.player_operation_priority[i],
                r.u8()? != 0,
                "priority case {case}"
            );
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("Verified 1536 operation-control packet transitions against the original packet pump");
    println!("Verified outgoing chat script dispatch with the real cache codec, private echo delivery, moderator message history and duplicate suppression");
    println!("Verified friend loading, aliases/ignore filtering, chat history, clan snapshots/deltas, selected-object identity, stale revisions, resync, management dispatch and typed profile packets through ClientState");
    Ok(())
}
