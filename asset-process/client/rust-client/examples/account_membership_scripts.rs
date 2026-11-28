use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    social::{FriendUpdate, IgnoreUpdate, Social},
    vm::{Host, Object, Stacks},
    Result,
};
fn text(r: &mut Reader) -> String {
    let n = r.u32().unwrap() as usize;
    String::from_utf8(r.bytes(n).unwrap().to_vec()).unwrap()
}
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let mut r = Reader::new(include_bytes!("../tests/fixtures/social-command-jvm.bin"));
    for sample in 0..r.u32().unwrap() {
        let op = r.u32().unwrap() as u16;
        let name = text(&mut r);
        let rank = r.u32().unwrap() as i32;
        let member = r.u8().unwrap() != 0;
        let nf = r.u32().unwrap();
        let ni = r.u32().unwrap();
        let mut state = Social::default();
        state
            .apply_friends(
                (0..nf)
                    .map(|i| FriendUpdate {
                        rename: false,
                        current: format!("Friend{i}"),
                        previous: format!("OldFriend{i}"),
                        world: 0,
                        rank: 0,
                        flags: 0,
                    })
                    .collect(),
                301,
                0,
            )
            .unwrap();
        state
            .apply_ignores(
                (0..ni)
                    .map(|i| IgnoreUpdate::Upsert {
                        rename: false,
                        current: format!("Ignore{i}"),
                        previous: format!("OldIgnore{i}"),
                    })
                    .collect(),
            )
            .unwrap();
        client.social = state;
        client.data.account_members = member;
        client.data.members = !member;
        client.outbound.clear();
        client.chat = Default::default();
        let mut stacks = Stacks::default();
        stacks.push_object(Object::String(name.clone()))?;
        if op == 3604 {
            stacks.push_int(rank)?;
        }
        client.extended(op, false, &mut stacks)?;
        let result = gielinor_client::social::Command {
            packet: client.outbound.pop_front(),
            message: client.chat.line(30, 0).map(|m| m.text.clone()),
            changed: false,
        };
        assert!(client.outbound.is_empty());
        let mut world = Stacks::default();
        client.extended(3312, false, &mut world)?;
        assert_eq!(world.ints(), [i32::from(!member)]);
        let state = &client.social;
        let sent = r.u8().unwrap() != 0;
        assert_eq!(
            result.packet.is_some(),
            sent,
            "sample {sample} op {op} {name}"
        );
        if let Some(packet) = result.packet {
            assert_eq!(packet.opcode, r.u32().unwrap() as u16);
            let n = r.u32().unwrap() as usize;
            assert_eq!(packet.payload, r.bytes(n).unwrap());
        }
        assert_eq!(state.friends.len(), r.u32().unwrap() as usize);
        assert_eq!(state.ignores.len(), r.u32().unwrap() as usize);
        let message = r.u8().unwrap() != 0;
        assert_eq!(result.message.is_some(), message, "message {sample}");
        if message {
            assert_eq!(result.message.unwrap(), text(&mut r));
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("400 original social command cases passed through ClientState with opposite account/world membership");
    Ok(())
}
