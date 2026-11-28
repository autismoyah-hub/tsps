//! Real-cache packet dispatch isolates the local scene copy from active packet world.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    game_packet,
    player_info::{State, View},
    protocol::Packet,
    Result,
};
fn apply(c: &mut ClientState, id: i32, delay: u8) -> Result<()> {
    let id = id as u16;
    c.apply(
        game_packet::decode(&Packet {
            opcode: 119,
            payload: vec![delay.wrapping_neg(), id as u8, (id >> 8) as u8],
        })?
        .unwrap(),
    )
}
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    apply(&mut c, 65534, 255)?; // No local player: ignore even an unavailable sequence.
    let (mut state, _) = State::initialize(&vec![0; 4608], 42)?;
    let p = state.local_actor().unwrap().clone();
    for id in [0, 7] {
        state.views.insert(
            id,
            View {
                base: [0, 0],
                dimensions: [16, 16],
                players: [(42, p.clone())].into(),
            },
        );
    }
    c.player_state = Some(state);
    for local in [0, 7] {
        c.active_world = if local == 0 { 7 } else { 0 };
        let state = c.player_state.as_mut().unwrap();
        state.local_view = local;
        let p = state.local_actor_mut().unwrap();
        p.path.len = 4;
        apply(&mut c, 808, 23)?;
        let state = c.player_state.as_ref().unwrap();
        let p = state.local_actor().unwrap();
        assert_eq!(
            (p.sequence.id, p.sequence.delay, p.path.path_at_start),
            (808, 23, 4)
        );
        assert_eq!(state.views[&c.active_world].players[&42].sequence.id, -1);
        assert_eq!(state.players[42].as_ref().unwrap().sequence.id, -1);
        apply(&mut c, -1, 2)?;
        assert_eq!(
            c.player_state
                .as_ref()
                .unwrap()
                .local_actor()
                .unwrap()
                .sequence
                .id,
            -1
        );
    }
    for (view, index) in [(999, 42), (7, 99)] {
        let s = c.player_state.as_mut().unwrap();
        s.local_view = view;
        s.local = index;
        apply(&mut c, 65534, 255)?;
    }
    println!("Local animation dispatch: root/child isolation, clear, missing actor/view and real-cache sequence passed");
    Ok(())
}
