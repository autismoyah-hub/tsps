use super::*;
use std::net::TcpListener;

fn pair() -> (Session, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (peer, _) = listener.accept().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut session = Session::from_connected(stream, Duration::from_secs(5)).unwrap();
    session.pending.clear();
    session.login = None;
    session.game = Some(protocol::Decoder::new(None));
    (session, peer)
}
#[test]
fn socket_flush_stages_whole_game_frames_in_fifo_batches() {
    let (mut session, mut peer) = pair();
    let opcode = crate::protocol239::CLIENT_PACKETS
        .iter()
        .find(|p| p.size == -2)
        .unwrap()
        .opcode as u16;
    let first = vec![7; 2997];
    let second = vec![9; 2497];
    session.send(opcode, &first).unwrap();
    session.send(opcode, &second).unwrap();
    session.send(89, &[]).unwrap();
    assert_eq!(session.game_outgoing.pending_bytes(), 5501);
    session.flush().unwrap();
    let mut bytes = vec![0; 3000];
    peer.read_exact(&mut bytes).unwrap();
    assert_eq!(
        bytes,
        protocol::encode_client(opcode, &first, &mut None).unwrap()
    );
    assert_eq!(session.game_outgoing.pending_bytes(), 2501);
    session.flush().unwrap();
    let mut bytes = vec![0; 2501];
    peer.read_exact(&mut bytes).unwrap();
    let mut expected = protocol::encode_client(opcode, &second, &mut None).unwrap();
    expected.extend(protocol::encode_client(89, &[], &mut None).unwrap());
    assert_eq!(bytes, expected);
    assert!(session.game_outgoing.is_empty());
}
#[test]
fn oversized_head_retains_following_frames_without_closing_socket() {
    let (mut session, _peer) = pair();
    let opcode = crate::protocol239::CLIENT_PACKETS
        .iter()
        .find(|p| p.size == -2)
        .unwrap()
        .opcode as u16;
    session.send(opcode, &vec![1; 4998]).unwrap();
    session.send(89, &[]).unwrap();
    for _ in 0..3 {
        session.flush().unwrap();
        assert!(!session.is_closed());
        assert_eq!(session.game_outgoing.pending_bytes(), 5002);
        assert!(session.pending.is_empty());
    }
    session.close();
    assert_eq!(session.game_outgoing.pending_bytes(), 0);
    assert!(session.game_outgoing.is_empty());
}
#[test]
fn unfinished_raw_transport_bytes_precede_game_batch() {
    let (mut session, mut peer) = pair();
    session.queue(&[14]);
    session.send(89, &[]).unwrap();
    session.flush().unwrap();
    let mut byte = [0];
    peer.read_exact(&mut byte).unwrap();
    assert_eq!(byte, [14]);
    assert_eq!(session.game_outgoing.pending_bytes(), 1);
    session.flush().unwrap();
    peer.read_exact(&mut byte).unwrap();
    assert_eq!(byte, [89]);
    assert!(session.game_outgoing.is_empty());
}

#[test]
fn partial_transport_write_completes_before_the_next_batch() {
    let (mut session, mut peer) = pair();
    session.pending = (0..100).collect();
    session
        .stream
        .as_mut()
        .unwrap()
        .write_all(&session.pending[..37])
        .unwrap();
    session.sent = 37;
    session.send(89, &[]).unwrap();
    assert!(session.ensure_capacity(QUEUE_LIMIT - 64).is_ok());
    assert!(session.ensure_capacity(QUEUE_LIMIT - 63).is_err());
    session.flush().unwrap();
    let mut bytes = [0; 100];
    peer.read_exact(&mut bytes).unwrap();
    assert_eq!(bytes.to_vec(), (0..100).collect::<Vec<u8>>());
    assert_eq!(session.game_outgoing.pending_bytes(), 1);
    session.flush().unwrap();
    let mut opcode = [0];
    peer.read_exact(&mut opcode).unwrap();
    assert_eq!(opcode, [89]);
    assert!(session.game_outgoing.is_empty());
}

#[test]
fn game_cycles_emit_on_51_and_zero_tick_polls_do_not_advance() {
    let (mut session, mut peer) = pair();
    session.advance_keepalive(50, true).unwrap();
    assert_eq!(session.keepalive.cycles(), 50);
    session.advance_keepalive(0, true).unwrap();
    session.flush().unwrap();
    assert_eq!(session.keepalive.cycles(), 50);
    session.advance_keepalive(1, true).unwrap();
    let mut byte = [0];
    peer.read_exact(&mut byte).unwrap();
    assert_eq!(byte, [89]);
    assert_eq!(session.keepalive.cycles(), 0);
    session.advance_keepalive(103, true).unwrap();
    let mut bytes = [0; 2];
    peer.read_exact(&mut bytes).unwrap();
    assert_eq!(bytes, [89, 89]);
    assert_eq!(session.keepalive.cycles(), 1);
}

#[test]
fn ordinary_write_resets_idle_but_enqueue_does_not() {
    let (mut session, mut peer) = pair();
    session.advance_keepalive(49, true).unwrap();
    let opcode = crate::protocol239::CLIENT_PACKETS
        .iter()
        .find(|p| p.size == -2)
        .unwrap()
        .opcode as u16;
    session.send(opcode, &[7, 8]).unwrap();
    assert_eq!(session.keepalive.cycles(), 49);
    session.advance_keepalive(1, true).unwrap();
    let mut bytes = [0; 5];
    peer.read_exact(&mut bytes).unwrap();
    assert_eq!(
        bytes.to_vec(),
        protocol::encode_client(opcode, &[7, 8], &mut None).unwrap()
    );
    assert_eq!(session.keepalive.cycles(), 0);
    session.advance_keepalive(50, true).unwrap();
    assert_eq!(session.keepalive.cycles(), 50);
    session.advance_keepalive(1, true).unwrap();
    let mut byte = [0];
    peer.read_exact(&mut byte).unwrap();
    assert_eq!(byte, [89]);
}

#[test]
fn loading_uses_50_cycles_and_disconnected_does_not_advance() {
    let (mut session, mut peer) = pair();
    session.advance_keepalive(49, false).unwrap();
    assert_eq!(session.keepalive.cycles(), 49);
    session.advance_keepalive(1, false).unwrap();
    let mut byte = [0];
    peer.read_exact(&mut byte).unwrap();
    assert_eq!(byte, [89]);
    assert_eq!(session.keepalive.cycles(), 0);
    session.close();
    session.advance_keepalive(150, true).unwrap();
    assert_eq!(session.keepalive.cycles(), 0);
}

#[test]
fn unsuccessful_flush_keeps_idle_count_and_oversized_head_resets_it() {
    let (mut session, _peer) = pair();
    session.advance_keepalive(17, true).unwrap();
    session.send(89, &[]).unwrap();
    let stream = session.stream.take();
    assert!(session.flush().is_err());
    assert_eq!(session.keepalive.cycles(), 17);
    session.stream = stream;
    session.flush().unwrap();
    assert_eq!(session.keepalive.cycles(), 0);
    session.advance_keepalive(17, true).unwrap();
    session.game_outgoing.push(vec![0; 5001]);
    session.flush().unwrap();
    assert_eq!(session.keepalive.cycles(), 0);
    assert_eq!(session.game_outgoing.pending_bytes(), 5001);
}

fn deliver(session: &mut Session, peer: &mut TcpStream, bytes: &[u8]) -> Vec<Event> {
    let received = session.last_received;
    peer.write_all(bytes).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let events = session.poll().unwrap();
        if session.last_received != received {
            return events;
        }
        assert!(Instant::now() < deadline, "loopback delivery stalled");
        std::thread::yield_now();
    }
}

#[test]
fn payload_trickle_does_not_postpone_gameplay_timeout() {
    let (mut session, mut peer) = pair();
    session.advance_receive_timeout(700, true).unwrap();
    assert!(deliver(&mut session, &mut peer, &[65]).is_empty());
    assert_eq!(session.receive_timeout.cycles(), 0);
    session.advance_receive_timeout(250, true).unwrap();
    for byte in [0, 0] {
        assert!(deliver(&mut session, &mut peer, &[byte]).is_empty());
        session.advance_receive_timeout(250, true).unwrap();
    }
    assert_eq!(session.receive_timeout.cycles(), 750);
    assert!(deliver(&mut session, &mut peer, &[0]).is_empty());
    assert!(session.advance_receive_timeout(1, true).is_err());
    assert!(session.is_closed());
}

#[test]
fn complete_body_resets_deadline_and_loading_does_not_age_it() {
    let (mut session, mut peer) = pair();
    assert!(deliver(&mut session, &mut peer, &[65, 0, 0, 0]).is_empty());
    session.advance_receive_timeout(750, true).unwrap();
    let events = deliver(&mut session, &mut peer, &[7]);
    assert_eq!(
        events,
        vec![Event::Packet(protocol::Packet {
            opcode: 65,
            payload: vec![0, 0, 0, 7]
        })]
    );
    assert_eq!(session.receive_timeout.cycles(), 0);
    session.advance_receive_timeout(5000, false).unwrap();
    session.advance_receive_timeout(0, true).unwrap();
    assert_eq!(session.receive_timeout.cycles(), 0);
    session.last_received = Instant::now() - Duration::from_secs(120);
    assert!(session.poll().unwrap().is_empty());
    assert!(!session.is_closed());
    session.advance_receive_timeout(750, true).unwrap();
    assert!(session.advance_receive_timeout(1, true).is_err());
}
