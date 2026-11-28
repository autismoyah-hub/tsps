use gielinor_client::{
    buffer::Reader,
    player_appearance,
    player_info::{State, View},
    Result,
};
fn text(r: &mut Reader<'_>) -> Result<String> {
    let len = r.u32()? as usize;
    Ok(std::str::from_utf8(r.bytes(len)?).unwrap().to_owned())
}
fn main() -> Result<()> {
    use gielinor_client::{
        client_state::ClientState,
        disk_cache::DiskCache,
        vm::{Control, Host, Object, Stacks},
    };
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let mut r = Reader::new(include_bytes!("../tests/fixtures/named-player-action-jvm.bin"));
    for case in 0..r.u32()? {
        let operation = r.u32()? as i32;
        let target = text(&mut r)?;
        let selected = r.u32()? as u16;
        let order = [r.u32()? as u16, r.u32()? as u16, r.u32()? as u16];
        let (mut state, _) = State::initialize(&[0; 4608], 1)?;
        let template = state.players[1].as_ref().unwrap().clone();
        state.local_view = selected;
        state.high = order.to_vec();
        for view in [0, 7] {
            let mut actors = std::collections::BTreeMap::new();
            for index in order {
                let mut actor = template.clone();
                if r.u8()? != 0 {
                    actor.appearance = Some(player_appearance::Update {
                        appearance: Default::default(),
                        skull: -1,
                        overhead: -1,
                        animations: [-1; 7],
                        name: text(&mut r)?,
                        combat: 3,
                        skill: 0,
                        hidden: false,
                        force_refresh: false,
                        actions: Default::default(),
                        team: 0,
                    });
                }
                actors.insert(index, actor);
            }
            state.views.insert(
                view,
                View {
                    base: [0; 2],
                    dimensions: [104; 2],
                    players: actors,
                },
            );
        }
        let expected = if r.u8()? != 0 {
            let opcode = r.u32()? as u16;
            let len = r.u32()? as usize;
            Some(gielinor_client::protocol::Packet {
                opcode,
                payload: r.bytes(len)?.to_vec(),
            })
        } else {
            None
        };
        let message = if r.u8()? != 0 {
            Some(text(&mut r)?)
        } else {
            None
        };
        client.player_state = Some(state);
        client.active_world = 900;
        client.map_flag = Some(12345);
        client.selected_spell = Some(gielinor_client::spell_action::Selection {
            widget: 42,
            logical: 42,
            child: 0,
            item: 0,
            mask: 1,
            option: "Cast".into(),
            target: "test".into(),
        });
        client.outbound.clear();
        let count = client.chat.count(4);
        let mut stacks = Stacks::default();
        stacks.push_int(operation)?;
        stacks.push_object(Object::String(target))?;
        let dispatch = client.extended(3107, false, &mut stacks);
        assert_eq!(dispatch?, Control::Continue, "3107 case {case}");
        assert!(stacks.ints().is_empty() && stacks.objects().is_empty());
        assert_eq!(client.active_world, 900);
        assert_eq!(client.map_flag, Some(12345));
        assert_eq!(client.selected_spell.as_ref().unwrap().widget, 42);
        let packet = client.outbound.pop_front();
        assert!(client.outbound.is_empty());
        let actual_message = if message.is_some() {
            assert_eq!(client.chat.count(4), (count + 1).min(100));
            Some(client.chat.line(4, 0).unwrap().text.clone())
        } else {
            assert_eq!(client.chat.count(4), count);
            None
        };
        let actual = (packet, actual_message);
        assert_eq!(actual.0, expected, "case {case}");
        assert_eq!(actual.1, message, "case {case}");
    }
    assert_eq!(r.remaining(), 0);
    println!("Actual ClientState: 512 named-player actions; local selected view, packet bytes, messages and preserved spell/map state");
    Ok(())
}
