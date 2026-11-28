//! #88 full live-poll fixture: the production `Connection::poll_elapsed`
//! (session poll, deferred-packet loop, packet decode/apply, generation drain,
//! keepalive) over a real loopback `Session` that logs in with ISAAC, a real
//! `ClientState` (interfaces, VM, scripts) and a fixed synthetic disk cache.
//! Only `Connection::for_packet_tests` differs from production construction;
//! see its documentation for what it omits.
//!
//! These tests pin the current ordering for packet-synchronous work: a packet
//! that queues onLoad, lifecycle or a structural continuation closes the
//! deferred-packet gate until those entries have run. #88 Slice A tests
//! (marked "SLICE A") state the end-of-cycle editor-callback behaviour:
//! GameClient.aft applies the packet batch (ex) before draining ct (cq.ac),
//! so an editor callback chain must not hold unrelated packets, and a queued
//! editor callback never runs against a widget instance other than the one
//! that produced it (a bounded fail-closed usability decision, not Java
//! parity: Java runs stale static callbacks and its by-id ops reach a
//! replacement). They record actual applied state per poll. There are no wall-clock pass
//! thresholds: loops are bounded by poll counts and socket read timeouts.
//! Login prefix: like a real server, the peer writes the login reply together
//! with a first game packet (MESSAGE_GAME "fixture ready"), because revision
//! 239's login decoder only emits Success once the first game packet's
//! opcode/length bytes have arrived. Scenarios start from that baseline
//! (one game message, tick 0, nothing deferred or queued).
//! The logout test also preloads the committed title fixtures after login and
//! disables title music (`preload_title`), a render-only fixture difference,
//! so the production post-logout `title_input` path can run.
//! Each fixture listener binds `127.0.0.1:0` and writes one bounded
//! `unit-listener/1` line with the OS-assigned address to stderr (#221), so the
//! supervisor's held stderr retains the actual port; game ports are refused.
//!
//! Coverage gaps, explicit (not tested here):
//! - IF_RESYNC_V2: its reference body is not fully traced
//!   (packet-structure-and-fixture-design.md), so no expectation is pinned;
//! - a PendingAsset arriving in the middle of an active callback chain;
//! - interleaving with world rebuild (`world_job`) deferral.
use super::Connection;
use crate::{
    client_state::ClientState,
    crypto::{crc32, Isaac},
    disk_cache::DiskCache,
    login::{Authentication, Credentials, Platform, Request},
    protocol,
    protocol239::SERVER_PACKETS,
    rsa::RsaPublicKey,
    session::{Event, Session},
    writer::Writer,
};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const ROOT_GROUP: u16 = 100;
const SUB_GROUP: u16 = 101;
const ROOT_W: u32 = (ROOT_GROUP as u32) << 16;
const ROOT_ALT: u32 = ROOT_W | 1;
const G_TEXT: u32 = ((SUB_GROUP as u32) << 16) | 1;
const G_EDIT: u32 = ((SUB_GROUP as u32) << 16) | 2;

/// Root onLoad: varp ROOT_V = 1; installs LIFE as ROOT_W's 1424 listener (op 2424).
const ROOT_ONLOAD: u16 = 20;
/// Sub-interface onLoad: varp ONLOAD_V = 7; installs CHAIN as G_EDIT's 1436 listener (op 2436).
const ONLOAD: u16 = 21;
/// RUNCLIENTSCRIPT target: varp READ_V = varp ONLOAD_V.
const READ: u16 = 22;
/// RUNCLIENTSCRIPT target: focus then unfocus G_EDIT (op 2140), queuing its 1436 listener.
const START: u16 = 23;
/// 1436 listener: counts in CHAIN_COUNT and, until CHAIN_LIMIT, refocuses/unfocuses (self-requeue).
const CHAIN: u16 = 24;
/// 1424 lifecycle listener: LIFE_COUNT += 1.
const LIFE: u16 = 25;
/// Dynamic-editor 1436 listener: STALE_V += 1 (any run is observable).
const STALE_D: u16 = 26;
/// 1436 listener that always refocuses/unfocuses: a chain with no end.
const FOREVER: u16 = 27;
/// RUNCLIENTSCRIPT: create a dynamic editor at G_CONTAINER slot 0 with the
/// STALE_D listener, focus then unfocus it (queues its 1436 callback), then
/// replace that slot (op 100, non-strict) before any drain.
const MAKE_REPLACE: u16 = 28;
/// As MAKE_REPLACE, but detaches the slot (op 102 delete children) instead.
const MAKE_DETACH: u16 = 29;
/// RUNCLIENTSCRIPT: installs FOREVER as G_EDIT's 1436 listener and starts it.
const START_FOREVER: u16 = 30;
/// RUNCLIENTSCRIPT target: varp READ_ROOT_V = varp ROOT_V.
const READ_ROOT: u16 = 31;
/// 1436 listener whose script group the remote-cache fixture withholds: HELD_V += 1.
const HELD_CB: u16 = 32;
/// RUNCLIENTSCRIPT: installs HELD_CB as G_EDIT's 1436 listener and queues it.
const START_HELD: u16 = 33;
/// Nested dynamic editor 1436 listener: NEST_V += 1.
const NEST_CB: u16 = 34;
/// RUNCLIENTSCRIPT: dynamic layer (kind 0) at G_CONTAINER slot 0, then a
/// nested dynamic editor under it (op 106, slot 1) with NEST_CB, focused then
/// unfocused. The editor's `parent` is the dynamic layer, while its `dynamic`
/// slot is keyed by the static owner G_CONTAINER (create_child_inner).
const NEST_MAKE: u16 = 35;
/// As NEST_MAKE, then deletes G_CONTAINER's children (the layer and the editor).
const NEST_DETACH: u16 = 36;
/// As NEST_MAKE, then replaces the layer's slot (op 100, non-strict), which deletes the nested editor.
const NEST_REPLACE: u16 = 37;

const ROOT_V: u16 = 14;
const ONLOAD_V: u16 = 10;
const READ_V: u16 = 11;
const CHAIN_COUNT: u16 = 12;
const LIFE_COUNT: u16 = 13;
const STALE_V: u16 = 15;
const FOREVER_COUNT: u16 = 16;
const READ_ROOT_V: u16 = 17;
const HELD_V: u16 = 18;
const NEST_V: u16 = 19;
const CHAIN_LIMIT: i32 = 40;
const G_CONTAINER: u32 = (SUB_GROUP as u32) << 16;
/// Prefix of the report for a dropped end-of-cycle callback (Slice A guard).
const STALE_REPORT: &str = "stale end-of-cycle callback";

const OPEN_TOP: u16 = 96;
const OPEN_SUB: u16 = 7;
const CLOSE_SUB: u16 = 23;
const MOVE_SUB: u16 = 123;
const SET_TEXT: u16 = 80;
const RUN_CLIENT_SCRIPT: u16 = 114;
const SERVER_TICK: u16 = 83;
const LOGOUT: u16 = 57;
const WORLD_HOP: u16 = 91;

/// Setup/IO failure bounds only; no assertion depends on elapsed time.
const SETUP: Duration = Duration::from_secs(10);
/// Accepts one connection or fails the test after SETUP; the accepted socket
/// gets read and write timeouts so no peer can block forever.
fn accept_within(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + SETUP;
    loop {
        match listener.accept() {
            Ok((socket, _)) => {
                socket.set_nonblocking(false).unwrap();
                socket.set_read_timeout(Some(SETUP)).unwrap();
                socket.set_write_timeout(Some(SETUP)).unwrap();
                return socket;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "no connection within {SETUP:?}");
                thread::sleep(Duration::from_millis(1));
            }
            Err(error) => panic!("accept failed: {error}"),
        }
    }
}

// ---- listener evidence (#221) ----------------------------------------------

/// Schema tag of the one-line record each fixture listener writes to stderr.
const LISTENER_SCHEMA: &str = "unit-listener/1";
/// Upper bound of one record line, newline included.
const LISTENER_RECORD_MAX: usize = 128;
/// The only fixture labels a record may carry; no free-form text.
const LISTENER_FIXTURES: [&str; 3] = ["remote_cache", "logged_in", "listener_self_test"];

/// The record for one live fixture listener: schema, process id, fixture label and
/// the address the OS actually assigned. Refuses anything but an IPv4 loopback with
/// a nonzero port outside the game server ranges (43594/43595 and 43600..=43699):
/// a unit fixture never takes, or falls back to, a game port.
fn listener_record(fixture: &str, address: std::net::SocketAddr) -> Result<String, &'static str> {
    if !LISTENER_FIXTURES.iter().any(|&known| known == fixture) {
        return Err("unknown fixture label");
    }
    let std::net::SocketAddr::V4(v4) = address else { return Err("not IPv4") };
    if !v4.ip().is_loopback() {
        return Err("not loopback");
    }
    let port = v4.port();
    if port == 0 || port == 43594 || port == 43595 || (43600..=43699).contains(&port) {
        return Err("port 0 or a game server port");
    }
    let line = format!("{LISTENER_SCHEMA} pid={} fixture={fixture} addr={v4}\n", std::process::id());
    if line.len() > LISTENER_RECORD_MAX {
        return Err("record exceeds its bound");
    }
    Ok(line)
}

/// Writes one record to `out` in a single `write_all`. A refusal is `InvalidData`
/// and nothing is written; a write error is returned as is.
fn write_record(out: &mut impl Write, fixture: &str, address: std::net::SocketAddr) -> std::io::Result<()> {
    let line = listener_record(fixture, address)
        .map_err(|reason| std::io::Error::new(std::io::ErrorKind::InvalidData, reason))?;
    out.write_all(line.as_bytes())?;
    out.flush()
}

/// Binds a fixture listener on `127.0.0.1:0` and records the address the OS assigned.
/// On any error the listener is dropped unused, so no test proceeds on an unrecorded
/// listener.
fn bind_recorded(out: &mut impl Write, fixture: &str) -> std::io::Result<(TcpListener, std::net::SocketAddr)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    write_record(out, fixture, address)?;
    Ok((listener, address))
}

/// The packet fixtures' listener. The record goes straight to the process stderr
/// (not `eprintln!`, which libtest captures and drops for passing tests), so stdout's
/// libtest grammar is untouched. A failed record fails the test.
fn fixture_listener(fixture: &str) -> (TcpListener, std::net::SocketAddr) {
    bind_recorded(&mut std::io::stderr().lock(), fixture)
        .unwrap_or_else(|error| panic!("{LISTENER_SCHEMA} record for {fixture} failed: {error}"))
}

#[test]
fn a_listener_record_names_the_live_listeners_assigned_address() {
    let mut out = Vec::new();
    let (listener, address) = bind_recorded(&mut out, "listener_self_test").unwrap();
    // This listener is evidence too, so its record also reaches stderr.
    std::io::stderr().lock().write_all(&out).unwrap();
    assert_eq!(address, listener.local_addr().unwrap());
    assert_ne!(address.port(), 0);
    let line = String::from_utf8(out).unwrap();
    assert_eq!(line, format!("unit-listener/1 pid={} fixture=listener_self_test addr={address}\n", std::process::id()));
    assert!(line.len() <= LISTENER_RECORD_MAX);
    // The recorded address is the live listener: a client connecting there is accepted by it.
    let client = TcpStream::connect(address).unwrap();
    let accepted = accept_within(&listener);
    assert_eq!(accepted.peer_addr().unwrap(), client.local_addr().unwrap());
}

#[test]
fn a_listener_record_refuses_game_ports_wildcards_and_unknown_labels() {
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
    let loopback = |port: u16| SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    for port in [0, 43594, 43595, 43600, 43649, 43650, 43651, 43699] {
        assert!(listener_record("logged_in", loopback(port)).is_err(), "port {port}");
    }
    for port in [43593, 43596, 43599, 43700, 49152, 65535] {
        assert!(listener_record("logged_in", loopback(port)).is_ok(), "port {port}");
    }
    assert!(listener_record("logged_in", SocketAddr::from((Ipv4Addr::UNSPECIFIED, 50000))).is_err());
    assert!(listener_record("logged_in", SocketAddr::from((Ipv4Addr::new(10, 0, 0, 1), 50000))).is_err());
    assert!(listener_record("logged_in", SocketAddr::from((Ipv6Addr::LOCALHOST, 50000))).is_err());
    assert!(listener_record("logged_in C:\\x", loopback(50000)).is_err());
    assert!(listener_record("", loopback(50000)).is_err());
}

#[test]
fn a_failed_listener_record_is_an_error_not_a_silent_listener() {
    struct Broken;
    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "closed stderr"))
        }
        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }
    // A synthetic address: this test opens no socket, so no listener goes unrecorded.
    let address = std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 50000));
    let error = write_record(&mut Broken, "listener_self_test", address).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    // Refusals write nothing.
    let mut out = Vec::new();
    for (fixture, port) in [("free text", 50000), ("logged_in", 43651), ("logged_in", 0)] {
        let refused = std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, port));
        assert_eq!(write_record(&mut out, fixture, refused).unwrap_err().kind(), std::io::ErrorKind::InvalidData);
    }
    assert!(out.is_empty());
    write_record(&mut out, "logged_in", address).unwrap();
    assert_eq!(out, format!("unit-listener/1 pid={} fixture=logged_in addr=127.0.0.1:50000\n", std::process::id()).into_bytes());
}

// ---- fixed synthetic cache -------------------------------------------------

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn container(data: &[u8]) -> Vec<u8> {
    let mut result = vec![0];
    result.extend((data.len() as u32).to_be_bytes());
    result.extend(data);
    result
}
enum Op {
    I(u16, i32),
    S(&'static str),
    X(u16),
}
fn script(ops: &[Op]) -> Vec<u8> {
    let mut data = vec![0];
    for op in ops {
        match *op {
            Op::I(code, arg) => {
                data.extend(code.to_be_bytes());
                if code == 21 { data.push(0); } else { data.extend(arg.to_be_bytes()); }
            }
            Op::S(text) => { data.extend(3u16.to_be_bytes()); data.extend(text.as_bytes()); data.push(0); }
            Op::X(code) => { data.extend(code.to_be_bytes()); data.push(0); }
        }
    }
    data.extend((ops.len() as u32).to_be_bytes());
    data.extend([0; 12]);
    data.extend([0, 0, 1]);
    data
}
/// NEST_MAKE's body, then `tail`, then return.
fn nest(tail: &[Op]) -> Vec<Op> {
    use Op::*;
    let container = G_CONTAINER as i32;
    let mut ops = vec![
        I(0, container), I(0, 0), I(0, 0), I(0, 0), X(100),
        I(0, 12), I(0, 1), X(106),
        I(0, i32::from(NEST_CB)), S(""), X(1436),
        I(0, 1), X(1140), I(0, 0), X(1140),
    ];
    for op in tail {
        ops.push(match *op { I(code, arg) => I(code, arg), S(text) => S(text), X(code) => X(code) });
    }
    ops.push(I(21, 0));
    ops
}
fn scripts() -> BTreeMap<u16, Vec<u8>> {
    use Op::*;
    let (root_w, g_edit, container) = (ROOT_W as i32, G_EDIT as i32, G_CONTAINER as i32);
    let v = |id: u16| i32::from(id);
    BTreeMap::from([
        (ROOT_ONLOAD, script(&[I(0, 1), I(2, v(ROOT_V)), I(0, v(LIFE)), S(""), I(0, root_w), X(2424), I(21, 0)])),
        (ONLOAD, script(&[I(0, 7), I(2, v(ONLOAD_V)), I(0, v(CHAIN)), S(""), I(0, g_edit), X(2436), I(21, 0)])),
        (READ, script(&[I(1, v(ONLOAD_V)), I(2, v(READ_V)), I(21, 0)])),
        (START, script(&[I(0, 1), I(0, g_edit), X(2140), I(0, 0), I(0, g_edit), X(2140), I(21, 0)])),
        (CHAIN, script(&[
            I(1, v(CHAIN_COUNT)), I(0, 1), X(4000), I(2, v(CHAIN_COUNT)),
            I(1, v(CHAIN_COUNT)), I(0, CHAIN_LIMIT), I(32, 7),
            I(0, 1), I(0, g_edit), X(2140), I(0, 0), I(0, g_edit), X(2140), I(21, 0),
            I(21, 0),
        ])),
        (LIFE, script(&[I(1, v(LIFE_COUNT)), I(0, 1), X(4000), I(2, v(LIFE_COUNT)), I(21, 0)])),
        (STALE_D, script(&[I(1, v(STALE_V)), I(0, 1), X(4000), I(2, v(STALE_V)), I(21, 0)])),
        (FOREVER, script(&[
            I(1, v(FOREVER_COUNT)), I(0, 1), X(4000), I(2, v(FOREVER_COUNT)),
            I(0, 1), I(0, g_edit), X(2140), I(0, 0), I(0, g_edit), X(2140), I(21, 0),
        ])),
        (MAKE_REPLACE, script(&[
            I(0, container), I(0, 12), I(0, 0), I(0, 0), X(100),
            I(0, v(STALE_D)), S(""), X(1436),
            I(0, 1), X(1140), I(0, 0), X(1140),
            I(0, container), I(0, 12), I(0, 0), I(0, 0), X(100),
            I(21, 0),
        ])),
        (MAKE_DETACH, script(&[
            I(0, container), I(0, 12), I(0, 0), I(0, 0), X(100),
            I(0, v(STALE_D)), S(""), X(1436),
            I(0, 1), X(1140), I(0, 0), X(1140),
            I(0, container), X(102),
            I(21, 0),
        ])),
        (START_FOREVER, script(&[
            I(0, v(FOREVER)), S(""), I(0, g_edit), X(2436),
            I(0, 1), I(0, g_edit), X(2140), I(0, 0), I(0, g_edit), X(2140), I(21, 0),
        ])),
        (READ_ROOT, script(&[I(1, v(ROOT_V)), I(2, v(READ_ROOT_V)), I(21, 0)])),
        (HELD_CB, script(&[I(1, v(HELD_V)), I(0, 1), X(4000), I(2, v(HELD_V)), I(21, 0)])),
        (NEST_CB, script(&[I(1, v(NEST_V)), I(0, 1), X(4000), I(2, v(NEST_V)), I(21, 0)])),
        (NEST_MAKE, script(&nest(&[]))),
        (NEST_DETACH, script(&nest(&[I(0, container), X(102)]))),
        (NEST_REPLACE, script(&nest(&[I(0, container), I(0, 0), I(0, 0), I(0, 0), X(100)]))),
        (START_HELD, script(&[
            I(0, v(HELD_CB)), S(""), I(0, g_edit), X(2436),
            I(0, 1), I(0, g_edit), X(2140), I(0, 0), I(0, g_edit), X(2140), I(21, 0),
        ])),
    ])
}
/// One IF3 widget in the format `Widget::decode` reads.
fn widget(kind: u8, parent: Option<u16>, onload: Option<u16>) -> Vec<u8> {
    let mut w = Writer::new();
    w.u8(255); w.u8(kind); w.u16(0);
    for value in [0u16, 0, 10, 10] { w.u16(value); }
    for value in [0u8; 4] { w.u8(value); }
    w.u16(parent.unwrap_or(65535));
    w.u8(0);
    match kind {
        0 => { w.u16(0); w.u16(0); w.u8(0); }
        4 => { w.u16(65535); w.string("").unwrap(); w.u8(0); w.u8(0); w.u8(0); w.u8(0); w.u32(0); }
        _ => {}
    }
    w.u24(0).unwrap();
    w.string("").unwrap();
    w.u8(0); w.u8(0); w.u8(0); w.u8(0);
    w.string("").unwrap();
    for slot in 0..18 {
        match (slot, onload) {
            (0, Some(id)) => { w.u8(1); w.u8(0); w.u32(u32::from(id)); }
            _ => w.u8(0),
        }
    }
    for _ in 0..3 { w.u8(0); }
    w.into_vec()
}
/// Group bytes for one or more files: files followed by a one-chunk size table.
fn group(files: &[Vec<u8>]) -> (Vec<u8>, u16) {
    if files.len() == 1 { return (files[0].clone(), 1); }
    let mut data: Vec<u8> = files.concat();
    let mut previous = 0i64;
    for file in files {
        data.extend(((file.len() as i64 - previous) as i32).to_be_bytes());
        previous = file.len() as i64;
    }
    data.push(1);
    (data, files.len() as u16)
}
/// archive -> group -> (container bytes, file count).
fn archives() -> BTreeMap<u8, BTreeMap<u16, (Vec<u8>, u16)>> {
    let mut archives = BTreeMap::<u8, BTreeMap<u16, (Vec<u8>, u16)>>::new();
    let mut put = |archive: u8, id: u16, files: Vec<Vec<u8>>| {
        let (data, count) = group(&files);
        archives.entry(archive).or_default().insert(id, (container(&data), count));
    };
    for id in [3, 9, 10, 11, 12, 13, 16, 19, 32, 33] { put(2, id, vec![vec![0]]); }
    for (id, bytes) in scripts() { put(12, id, vec![bytes]); }
    let mut defaults = vec![2];
    for index in 0..11 { defaults.extend((if index == 7 { 0u16 } else { 32767 }).to_be_bytes()); }
    defaults.push(0);
    put(17, 3, vec![defaults]);
    let mut sprites = Vec::new();
    for color in 1..=8 { sprites.extend([0, color, 0]); }
    for color in 1..=8 { sprites.extend([0, 0, color]); }
    sprites.extend(16u16.to_be_bytes()); sprites.extend(16u16.to_be_bytes()); sprites.push(8);
    for value in [2u16, 3, 2, 1] { for _ in 0..8 { sprites.extend(value.to_be_bytes()); } }
    sprites.extend(8u16.to_be_bytes());
    put(8, 0, vec![sprites]);
    // The production texture loader (Texture::from_sprite) requires a square
    // 64x64 or 128x128 sheet, so the one texture uses its own 64x64 sheet
    // (archive 8 group 1: one full-canvas sprite, palette index 1).
    let mut texture_sheet = vec![0u8];
    texture_sheet.extend([1u8; 64 * 64]);
    texture_sheet.extend([0x80, 0x80, 0x80]);
    for value in [64u16, 64] { texture_sheet.extend(value.to_be_bytes()); }
    texture_sheet.push(1);
    for value in [0u16, 0, 64, 64, 1] { texture_sheet.extend(value.to_be_bytes()); }
    put(8, 1, vec![texture_sheet]);
    // Definition: sprite 1, average HSL 0, opaque, no animation.
    put(9, 0, vec![vec![0, 1, 0, 0, 1, 0, 0]]);
    put(3, ROOT_GROUP, vec![widget(0, None, Some(ROOT_ONLOAD)), widget(0, None, None)]);
    put(3, SUB_GROUP, vec![widget(0, None, Some(ONLOAD)), widget(4, Some(0), None), widget(12, Some(0), None)]);
    archives
}
fn references(archives: &BTreeMap<u8, BTreeMap<u16, (Vec<u8>, u16)>>) -> BTreeMap<u8, Vec<u8>> {
    archives.iter().map(|(&archive, groups)| {
        let mut reference = vec![6]; reference.extend(1u32.to_be_bytes()); reference.push(0);
        reference.extend((groups.len() as u16).to_be_bytes());
        let mut last = 0;
        for &id in groups.keys() { reference.extend((id - last).to_be_bytes()); last = id; }
        for (bytes, _) in groups.values() { reference.extend(crc32(bytes).to_be_bytes()); }
        for _ in groups { reference.extend(1u32.to_be_bytes()); }
        for (_, count) in groups.values() { reference.extend(count.to_be_bytes()); }
        for (_, count) in groups.values() {
            for file in 0..*count { reference.extend((if file == 0 { 0u16 } else { 1 }).to_be_bytes()); }
        }
        (archive, container(&reference))
    }).collect()
}
fn write_disk(withheld: &[(u8, u16)]) -> Fixture {
    // Tests run concurrently in one process and the clock can repeat at its
    // granularity, so a per-process counter keeps every fixture directory unique.
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("live-packet-{}-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
    std::fs::create_dir(&root).unwrap();
    let all = archives();
    let mut disk: BTreeMap<u8, BTreeMap<u16, Vec<u8>>> = all.iter().map(|(&archive, groups)| {
        (archive, groups.iter().filter(|(&id, _)| !withheld.contains(&(archive, id)))
            .map(|(&id, (bytes, _))| (id, bytes.clone())).collect())
    }).collect();
    disk.insert(255, references(&all).into_iter().map(|(a, r)| (a as u16, r)).collect());
    let mut data = vec![0u8; 520];
    for (archive, groups) in disk {
        let mut index = vec![0u8; (*groups.keys().last().unwrap() as usize + 1) * 6];
        for (id, mut bytes) in groups {
            if archive != 255 { bytes.extend(1u16.to_be_bytes()); }
            let first = data.len() / 520;
            index[id as usize * 6..id as usize * 6 + 3].copy_from_slice(&(bytes.len() as u32).to_be_bytes()[1..]);
            index[id as usize * 6 + 3..id as usize * 6 + 6].copy_from_slice(&(first as u32).to_be_bytes()[1..]);
            let chunks = bytes.len().div_ceil(512);
            for (chunk, part) in bytes.chunks(512).enumerate() {
                let next = if chunk + 1 < chunks { data.len() / 520 + 1 } else { 0 };
                let start = data.len();
                data.extend(id.to_be_bytes()); data.extend((chunk as u16).to_be_bytes());
                data.extend(&(next as u32).to_be_bytes()[1..]); data.push(archive); data.extend(part);
                data.resize(start + 520, 0);
            }
        }
        std::fs::write(root.join(format!("main_file_cache.idx{archive}")), index).unwrap();
    }
    std::fs::write(root.join("main_file_cache.dat2"), data).unwrap();
    Fixture(root)
}
fn wire(archive: u8, group: u16, container: &[u8]) -> Vec<u8> {
    let mut bytes = vec![archive, (group >> 8) as u8, group as u8];
    for &byte in container {
        if bytes.len() % 512 == 0 { bytes.push(255); }
        bytes.push(byte);
    }
    bytes
}
/// Disk cache plus a real JS5 recovery peer that serves master/reference
/// tables at once and the withheld group only after `release`.
///
/// The peer's join value is whether the held reply was written; a panic inside
/// the peer (handshake, unexpected request, held-reply write) is re-raised by
/// `join_peer`. A read or write error in the request loop is the expected end
/// of serving once the test drops its client and closes the JS5 stream.
fn remote_cache(withheld: (u8, u16)) -> (Fixture, ClientState, mpsc::Sender<()>, thread::JoinHandle<bool>) {
    let fixture = write_disk(&[withheld]);
    let all = archives();
    let references = references(&all);
    let mut master = vec![0; (*all.keys().last().unwrap() as usize + 1) * 8];
    for (&archive, reference) in &references {
        let offset = archive as usize * 8;
        master[offset..offset + 4].copy_from_slice(&crc32(reference).to_be_bytes());
        master[offset + 4..offset + 8].copy_from_slice(&1u32.to_be_bytes());
    }
    let held = all[&withheld.0][&withheld.1].0.clone();
    let (listener, address) = fixture_listener("remote_cache");
    let (release, released) = mpsc::channel();
    let worker = thread::spawn(move || {
        let mut stream = accept_within(&listener);
        let mut handshake = [0; 21];
        stream.read_exact(&mut handshake).unwrap();
        stream.write_all(&[0]).unwrap();
        let writer = std::sync::Arc::new(std::sync::Mutex::new(stream.try_clone().unwrap()));
        let held_writer = std::sync::Arc::clone(&writer);
        let held_reply = thread::spawn(move || {
            // No release within the bound (or a dropped sender) is "not written", which
            // the test asserts against; a released reply that fails to write panics.
            if released.recv_timeout(Duration::from_secs(20)).is_err() { return false; }
            held_writer.lock().unwrap().write_all(&wire(withheld.0, withheld.1, &held))
                .unwrap_or_else(|error| panic!("held JS5 reply {withheld:?} not written: {error}"));
            true
        });
        let mut request = [0; 4];
        while stream.read_exact(&mut request).is_ok() {
            let (archive, group) = (request[1], u16::from_be_bytes([request[2], request[3]]));
            let data = match (archive, group) {
                (255, 255) => container(&master),
                (255, archive) => references[&(archive as u8)].clone(),
                key if key == withheld => continue,
                other => panic!("unexpected JS5 request {other:?}"),
            };
            if writer.lock().unwrap().write_all(&wire(archive, group, &data)).is_err() { break; }
        }
        join_peer(held_reply)
    });
    let mut cache = DiskCache::open_remote(&fixture.0, 1 << 20, address).unwrap();
    cache.wait_for_recovery(true);
    let client = ClientState::load(cache).unwrap();
    (fixture, client, release, worker)
}
/// Joins a fixture peer thread, re-raising its panic with the original payload.
fn join_peer<T>(peer: thread::JoinHandle<T>) -> T {
    peer.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))
}

// ---- scripted game peer ----------------------------------------------------

/// The server side of the loopback connection after login: encodes server
/// packets with the production table and server ISAAC stream.
struct Peer {
    socket: TcpStream,
    cipher: Option<Isaac>,
}
/// Encodes server packets with the production table, advancing the server
/// ISAAC stream one word per opcode byte.
fn encode(cipher: &mut Option<Isaac>, packets: &[(u16, Vec<u8>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (opcode, payload) in packets {
        let spec = SERVER_PACKETS.iter().find(|s| s.opcode as u16 == *opcode).unwrap();
        bytes.extend(protocol::encode(spec, payload, cipher).unwrap());
    }
    bytes
}
impl Peer {
    fn send(&mut self, packets: &[(u16, Vec<u8>)]) {
        let bytes = encode(&mut self.cipher, packets);
        self.socket.write_all(&bytes).unwrap();
        self.socket.flush().unwrap();
    }
}
fn open_top(group: u16) -> (u16, Vec<u8>) { let mut w = Writer::new(); w.u16_add(group, false); (OPEN_TOP, w.into_vec()) }
fn open_sub(parent: u32, group: u16, kind: u8) -> (u16, Vec<u8>) {
    let mut w = Writer::new(); w.u16_add(group, true); w.u32_middle(parent); w.u8(kind); (OPEN_SUB, w.into_vec())
}
fn close_sub(parent: u32) -> (u16, Vec<u8>) { let mut w = Writer::new(); w.u32(parent); (CLOSE_SUB, w.into_vec()) }
fn move_sub(source: u32, destination: u32) -> (u16, Vec<u8>) {
    let mut w = Writer::new(); w.u32_le(destination); w.u32(source); (MOVE_SUB, w.into_vec())
}
fn set_text(widget: u32, text: &str) -> (u16, Vec<u8>) {
    let mut w = Writer::new(); w.string(text).unwrap(); w.u32_inverse_middle(widget); (SET_TEXT, w.into_vec())
}
fn run_script(id: u16) -> (u16, Vec<u8>) {
    let mut w = Writer::new(); w.string("").unwrap(); w.u32(u32::from(id)); (RUN_CLIENT_SCRIPT, w.into_vec())
}
fn tick() -> (u16, Vec<u8>) { (SERVER_TICK, Vec::new()) }
const MESSAGE_GAME: u16 = 74;
/// MESSAGE_GAME payload: unsigned-smart kind 0, no sender, text.
fn game_message(text: &str) -> (u16, Vec<u8>) {
    let mut w = Writer::new(); w.unsigned_smart(0).unwrap(); w.u8(0); w.string(text).unwrap(); (MESSAGE_GAME, w.into_vec())
}
/// Text of the first game packet sent with the login reply (see `logged_in`).
const FIRST_MESSAGE: &str = "fixture ready";
fn world_hop() -> (u16, Vec<u8>) {
    let mut w = Writer::new(); w.string("127.0.0.1").unwrap(); w.u16(1); w.u32(0); (WORLD_HOP, w.into_vec())
}

const LOGIN_SEEDS: [u32; 4] = [1, 2, 3, 4];
/// A logged-in production Connection over a real loopback Session.
fn logged_in(mut client: ClientState) -> (ClientState, Connection, Peer) {
    let (listener, address) = fixture_listener("logged_in");
    // The listener is dropped after one accept, so any reconnect is refused.
    let accept = thread::spawn(move || {
        let mut socket = accept_within(&listener);
        let mut init = [0];
        socket.read_exact(&mut init).unwrap();
        assert_eq!(init, [14]);
        socket.write_all(&[0, 1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        let mut header = [0; 3];
        socket.read_exact(&mut header).unwrap();
        assert_eq!(header[0], 16);
        let mut request = vec![0; u16::from_be_bytes([header[1], header[2]]) as usize];
        socket.read_exact(&mut request).unwrap();
        socket
    });
    let mut session = Session::connect(address, SETUP).unwrap();
    let key = RsaPublicKey::from_hex(&"f".repeat(256), 65537).unwrap();
    let platform = Platform::default();
    let crcs = [0; 23];
    let request = Request {
        username: "fixture",
        credentials: Credentials::Login { authentication: Authentication::Password, secret: "test", token: false },
        client_build: 2147483647, client_type: 0, platform_type: 0, resizable: false, low_detail: false,
        width: 765, height: 503, uuid: [0; 24], site_settings: "", affiliate: 0, deep_links: &[],
        platform: &platform, archive_crcs: &crcs,
    };
    let mut seeded = false;
    for _ in 0..5000 {
        if session.poll().unwrap().iter().any(|e| matches!(e, Event::Login(crate::login::Event::ServerSeed(_)))) {
            seeded = true;
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }
    assert!(seeded, "no server seed");
    session.submit_with_seed(&request, &key, LOGIN_SEEDS).unwrap();
    // submit_with_seed only queues the request; Session::poll flushes it. Keep
    // polling (bounded) until the peer has read it. The peer sends nothing more
    // until after the join, so no login event can be consumed here.
    let deadline = std::time::Instant::now() + SETUP;
    while !accept.is_finished() {
        assert!(std::time::Instant::now() < deadline, "login request never reached the peer within {SETUP:?}");
        assert!(session.poll().unwrap().is_empty(), "no login reply is sent before the request is read");
        thread::sleep(Duration::from_millis(1));
    }
    let mut socket = accept.join().unwrap();
    // Login success, byte-identical to tests/session.rs (no tokens): response 2,
    // declared size 37, then 34 account bytes (token flag + 4 token bytes, staff
    // level 2, moderator 1, player index 42, members 1, three u64 hashes).
    // Revision 239's login decoder (login.rs State::Success) counts the first
    // game packet's opcode/length in that 37 and emits Success only once 38
    // bytes follow the response code, so, as a real server does, the reply is
    // written together with a real first game packet: MESSAGE_GAME (74,
    // var-byte) "fixture ready". Its opcode consumes the first server ISAAC
    // word; after Success the login decoder drains 35 bytes and hands the rest
    // to the game decoder. Its only effect is one chat/game message: no tick,
    // varp, interface or script-queue change (pinned below as the baseline).
    let mut cipher = Some(Isaac::new(&LOGIN_SEEDS.map(|s| s + 50)));
    let mut w = Writer::new();
    w.bytes(&[2, 37, 0, 0, 0, 0, 0, 2, 1]);
    w.u16(42); w.u8(1); w.u64(1); w.u64(2); w.u64(3);
    w.bytes(&encode(&mut cipher, &[game_message(FIRST_MESSAGE)]));
    socket.write_all(w.as_slice()).unwrap();
    let mut connection = Connection::for_packet_tests(&mut client, session, address, LOGIN_SEEDS).unwrap();
    for _ in 0..1000 {
        connection.poll_elapsed(&mut client, 1).unwrap();
        if connection.local.is_some() { break; }
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(connection.local, Some(42), "production Success branch ran");
    assert!(connection.title.is_none());
    // Fixture baseline after login: the first packet was applied, nothing else.
    for _ in 0..1000 {
        if !client.game_messages.is_empty() { break; }
        connection.poll_elapsed(&mut client, 1).unwrap();
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(client.game_messages.iter().map(|m| m.text.as_str()).collect::<Vec<_>>(), [FIRST_MESSAGE],
        "first game packet applied exactly once");
    assert_eq!(client.server_tick, 0, "baseline: no tick before a scenario sends one");
    assert!(connection.deferred_events.is_empty() && client.interfaces.scripts.is_empty());
    // The peer continues the same server ISAAC stream (first word already used).
    let peer = Peer { socket, cipher };
    (client, connection, peer)
}
fn local_client() -> (Fixture, ClientState) {
    let fixture = write_disk(&[]);
    let client = ClientState::load(DiskCache::open(&fixture.0, 1 << 20).unwrap()).unwrap();
    (fixture, client)
}

/// One observation after a poll.
#[derive(Clone, Debug, PartialEq)]
struct Seen {
    tick: i32,
    root_onload: i32,
    onload: i32,
    read: i32,
    chain: i32,
    life: i32,
    mounted_at: Option<u32>,
    text: String,
    deferred: usize,
    queued: usize,
}
fn seen(client: &ClientState, connection: &Connection) -> Seen {
    let varp = |id: u16| client.scripts.variables.player(id);
    Seen {
        tick: client.server_tick,
        root_onload: varp(ROOT_V),
        onload: varp(ONLOAD_V),
        read: varp(READ_V),
        chain: varp(CHAIN_COUNT),
        life: varp(LIFE_COUNT),
        mounted_at: client.interfaces.children.values().find(|s| s.group == SUB_GROUP).map(|s| s.parent),
        text: client.interfaces.widgets.get(&G_TEXT).map_or(String::new(), |w| w.text.clone()),
        deferred: connection.deferred_events.len(),
        queued: client.interfaces.scripts.len(),
    }
}
/// Polls until `done` (bounded); returns every observation, one per poll.
fn poll_until(client: &mut ClientState, connection: &mut Connection, limit: usize,
    done: impl Fn(&Seen) -> bool) -> Vec<Seen> {
    let mut trace = Vec::new();
    for _ in 0..limit {
        connection.poll_elapsed(client, 1).unwrap();
        let now = seen(client, connection);
        trace.push(now.clone());
        if done(&now) { return trace; }
        // Pacing for loopback delivery only; never part of a pass condition.
        thread::sleep(Duration::from_millis(2));
    }
    panic!("condition not reached in {limit} polls: {:?}", trace.last());
}
fn first(trace: &[Seen], what: &str, f: impl Fn(&Seen) -> bool) -> usize {
    trace.iter().position(f).unwrap_or_else(|| panic!("{what} never observed; per-poll trace: {trace:?}"))
}
/// Polls until the production poll path has joined the owned connect worker
/// (reconnect or hop) and cleared `reconnect_job`; bounded by SETUP.
fn reap_connect_worker(client: &mut ClientState, connection: &mut Connection) {
    let limit = (SETUP.as_millis() / 2) as usize;
    for _ in 0..limit {
        if connection.reconnect_job.is_none() { return; }
        connection.poll_elapsed(client, 1).unwrap();
        thread::sleep(Duration::from_millis(2));
    }
    panic!("connect worker still running after {SETUP:?}");
}
/// Mounts SUB_GROUP under ROOT_W; returns once its onLoad ran.
fn mounted(client: &mut ClientState, connection: &mut Connection, peer: &mut Peer) {
    peer.send(&[open_top(ROOT_GROUP), open_sub(ROOT_W, SUB_GROUP, 1)]);
    poll_until(client, connection, 200, |s| s.onload == 7 && s.queued == 0);
}

// ---- scenarios ---------------------------------------------------------------

#[test]
fn open_top_open_sub_text_script_tick_apply_in_reference_dependency_order() {
    let (_fixture, client) = local_client();
    let (mut client, mut connection, mut peer) = logged_in(client);
    peer.send(&[open_top(ROOT_GROUP), open_sub(ROOT_W, SUB_GROUP, 1), set_text(G_TEXT, "hello"), run_script(READ), tick()]);
    let trace = poll_until(&mut client, &mut connection, 200, |s| s.tick == 1);
    let root = first(&trace, "root onLoad", |s| s.root_onload == 1);
    let mount = first(&trace, "sub-interface mount", |s| s.mounted_at == Some(ROOT_W));
    let text = first(&trace, "SETTEXT hello", |s| s.text == "hello");
    let ticked = first(&trace, "SERVER_TICK", |s| s.tick == 1);
    // Root onLoad (queued by OPEN_TOP) holds OPEN_SUB until a later poll.
    assert!(mount > root, "CURRENT: mount waits for the root onLoad generation {trace:?}");
    // The sub-interface onLoad and its 1424 lifecycle run in the mount poll's drain.
    assert_eq!((trace[mount].onload, trace[mount].life), (7, 1), "{trace:?}");
    // Dependent packets wait for that drain: SETTEXT lands in a later poll.
    assert!(text > mount, "CURRENT: SETTEXT held while onLoad was queued {trace:?}");
    assert_eq!(trace[mount].text, "", "update never precedes its mount");
    // RUNCLIENTSCRIPT runs inline and observes the onLoad result; the tick follows it.
    assert!(ticked >= text);
    assert_eq!(trace[ticked].read, 7, "RUNCLIENTSCRIPT saw onLoad's varp");
}

#[test]
fn missing_onload_script_suspends_following_packets_then_resumes_in_order() {
    let (_fixture, client, release, worker) = remote_cache((12, ONLOAD));
    let (mut client, mut connection, mut peer) = logged_in(client);
    client.scripts.source.cache.wait_for_recovery(false);
    peer.send(&[open_top(ROOT_GROUP), open_sub(ROOT_W, SUB_GROUP, 1), set_text(G_TEXT, "hello"), tick()]);
    let trace = poll_until(&mut client, &mut connection, 200, |s| s.mounted_at == Some(ROOT_W) && s.deferred == 2);
    // Held: onLoad is a PendingAsset at the queue front; SETTEXT and the tick stay deferred.
    for _ in 0..5 {
        connection.poll_elapsed(&mut client, 1).unwrap();
        let now = seen(&client, &connection);
        assert_eq!((now.onload, now.tick, now.text.as_str(), now.deferred), (0, 0, "", 2), "{trace:?} {now:?}");
        assert!(connection.pending_script.is_none(), "no spin: the event is requeued, not running");
    }
    release.send(()).unwrap();
    let after = poll_until(&mut client, &mut connection, 2000, |s| s.tick == 1);
    let ran = first(&after, "recovered onLoad", |s| s.onload == 7);
    let text = first(&after, "SETTEXT after recovery", |s| s.text == "hello");
    assert!(text > ran, "CURRENT: SETTEXT applies only after the recovered onLoad ran {after:?}");
    drop(connection);
    drop(client);
    drop(release);
    assert!(join_peer(worker), "the released held JS5 reply was written");
}

fn chunk<'a>(r: &mut crate::buffer::Reader<'a>) -> &'a [u8] {
    let n = r.u32().unwrap() as usize;
    r.bytes(n).unwrap()
}

/// Render-only fixture difference, logout scenario only. After LOGOUT the
/// production poll runs `title_input`, whose `ensure_title_assets` loads the
/// title screen from the cache; the synthetic cache has none. This installs
/// the committed decoded fixtures (tests/fixtures/title-assets.bin and
/// title-form-assets.bin, as tests/title.rs uses them) through the public
/// `title::Assets` APIs instead, so `input_dimensions` has its form assets.
/// It must run after login: the production Success branch resets
/// `title_assets`. Normal runtime loading is unchanged.
/// Also scoped to this test: title music is disabled via the device setting,
/// so the production `music.title` call only stops music (no cache lookup,
/// no audio). Fixture input stays empty, so no click, key, clipboard read,
/// world-list load or legal link is reached.
fn preload_title(client: &mut ClientState, connection: &mut Connection) {
    let mut a = crate::buffer::Reader::new(include_bytes!("../tests/fixtures/title-assets.bin"));
    let image = chunk(&mut a);
    let sprites = [chunk(&mut a), chunk(&mut a), chunk(&mut a)];
    let font = [chunk(&mut a), chunk(&mut a)];
    let mut assets = crate::title::Assets::decode(image, sprites, font).unwrap();
    assert_eq!(a.remaining(), 0);
    let mut a = crate::buffer::Reader::new(include_bytes!("../tests/fixtures/title-form-assets.bin"));
    let font = [chunk(&mut a), chunk(&mut a)];
    let radios = [chunk(&mut a), chunk(&mut a), chunk(&mut a), chunk(&mut a)];
    assets.install_form(font, radios).unwrap();
    assets.install_regular([chunk(&mut a), chunk(&mut a)]).unwrap();
    assert_eq!(a.remaining(), 0);
    assert!(connection.title_assets.is_none() && connection.world_selector.is_none());
    connection.title_assets = Some(assets);
    client.preferences.device.title_music_disabled = true;
}

#[test]
fn logout_ends_the_batch() {
    let (_fixture, client) = local_client();
    let (mut client, mut connection, mut peer) = logged_in(client);
    preload_title(&mut client, &mut connection);
    peer.send(&[tick(), (LOGOUT, Vec::new()), tick()]);
    poll_until(&mut client, &mut connection, 200, |_| true);
    for _ in 0..200 {
        if connection.logged_out.is_some() { break; }
        connection.poll_elapsed(&mut client, 1).unwrap();
        thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(connection.logged_out, Some(0));
    for _ in 0..5 { connection.poll_elapsed(&mut client, 1).unwrap(); }
    assert_eq!(client.server_tick, 1, "the tick after LOGOUT is never applied");
}

#[test]
fn world_hop_ends_the_batch() {
    let (_fixture, client) = local_client();
    let (mut client, mut connection, mut peer) = logged_in(client);
    peer.send(&[tick(), world_hop(), tick()]);
    for _ in 0..200 {
        if connection.hopping { break; }
        connection.poll_elapsed(&mut client, 1).unwrap();
        thread::sleep(Duration::from_millis(2));
    }
    assert!(connection.hopping && connection.local.is_none());
    assert_eq!(client.server_tick, 1, "the tick after the hop is not applied in that poll");
    // The hop's connect worker (same loopback port, listener dropped) is joined by the production poll.
    reap_connect_worker(&mut client, &mut connection);
    assert_eq!(client.server_tick, 1, "the tick after the hop is never applied");
}

#[test]
fn active_editor_callback_chain_does_not_hold_unrelated_ticks() {
    // SLICE A (A-T1, F1): a ct-class editor chain no longer gates received packets.
    let (_fixture, client) = local_client();
    let (mut client, mut connection, mut peer) = logged_in(client);
    mounted(&mut client, &mut connection, &mut peer);
    let mut packets = vec![run_script(START)];
    packets.extend((0..10).map(|_| tick()));
    peer.send(&packets);
    let mut trace = poll_until(&mut client, &mut connection, (CHAIN_LIMIT as usize) * 4 + 50, |s| s.tick == 10);
    let ticked = trace.len() - 1;
    assert!(trace[ticked].chain < CHAIN_LIMIT,
        "F1: all ten ticks applied only after the editor chain ended (chain {} of {CHAIN_LIMIT}); per-poll trace: {trace:?}",
        trace[ticked].chain);
    assert!(trace[ticked].chain > 0, "the chain started before the ticks were applied {trace:?}");
    trace.extend(poll_until(&mut client, &mut connection, (CHAIN_LIMIT as usize) * 4 + 50, |s| s.chain == CHAIN_LIMIT));
    // Bounded generations are retained: at most one chain link per poll.
    for pair in trace.windows(2) {
        assert!(pair[1].chain - pair[0].chain <= 1, "one link per drain {trace:?}");
    }
    assert_eq!(client.server_tick, 10);
}

#[test]
fn a_never_ending_editor_chain_cannot_starve_received_ticks() {
    // SLICE A (A-T2): a 1436 listener that always requeues itself; poll-bounded.
    let (_fixture, client) = local_client();
    let (mut client, mut connection, mut peer) = logged_in(client);
    mounted(&mut client, &mut connection, &mut peer);
    let mut packets = vec![run_script(START_FOREVER)];
    packets.extend((0..20).map(|_| tick()));
    peer.send(&packets);
    let forever = |client: &ClientState| client.scripts.variables.player(FOREVER_COUNT);
    let mut counts = vec![forever(&client)];
    let mut trace = Vec::new();
    for _ in 0..60 {
        connection.poll_elapsed(&mut client, 1).unwrap();
        trace.push(seen(&client, &connection));
        counts.push(forever(&client));
        if client.server_tick == 20 { break; }
        thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(client.server_tick, 20,
        "F1: received ticks starved by a never-ending editor chain within 60 polls (forever count {}); per-poll trace: {trace:?}",
        forever(&client));
    // The chain is still alive and still bounded to one link per poll.
    for _ in 0..5 {
        connection.poll_elapsed(&mut client, 1).unwrap();
        counts.push(forever(&client));
    }
    assert!(counts.windows(2).all(|pair| pair[1] - pair[0] <= 1), "one link per poll: {counts:?}");
    assert!(counts.last().unwrap() > &counts[counts.len() - 6], "the chain keeps running: {counts:?}");
    assert!(!client.interfaces.scripts.is_empty(), "its next link is queued");
}

#[test]
fn packet_synchronous_work_still_gates_dependent_packets_during_an_editor_chain() {
    // SLICE A (A-T3): only end-of-cycle entries are exempt from the gate. While
    // an editor chain is live, a root onLoad still runs before a following
    // RUNCLIENTSCRIPT reads it, and a MOVE_SUB relocation still precedes a
    // following SETTEXT; ticks keep applying, so the chain link re-pushed by
    // move_sub/resume kept its end-of-cycle class.
    let (_fixture, client) = local_client();
    let (mut client, mut connection, mut peer) = logged_in(client);
    mounted(&mut client, &mut connection, &mut peer);
    peer.send(&[run_script(START)]);
    let mut trace = poll_until(&mut client, &mut connection, 200, |s| s.chain >= 2);
    // Setup only: clear the root onLoad's marker so its re-run is observable.
    client.scripts.variables.set_player(ROOT_V, 0);
    peer.send(&[open_top(ROOT_GROUP), run_script(READ_ROOT), move_sub(ROOT_W, ROOT_ALT), set_text(G_TEXT, "moved"), tick(), tick()]);
    let base = trace.len();
    trace.extend(poll_until(&mut client, &mut connection, (CHAIN_LIMIT as usize) * 4 + 50, |s| s.tick == 2));
    assert!(trace.last().unwrap().chain < CHAIN_LIMIT,
        "the ticks applied while the editor chain was still live {trace:?}");
    assert_eq!(client.scripts.variables.player(READ_ROOT_V), 1, "RUNCLIENTSCRIPT ran after the root onLoad it depends on");
    let moved = base + first(&trace[base..], "relocation", |s| s.mounted_at == Some(ROOT_ALT));
    let text = base + first(&trace[base..], "SETTEXT moved", |s| s.text == "moved");
    assert!(text > moved, "SETTEXT waited for the relocation drain {trace:?}");
    assert_ne!(trace[moved].text, "moved", "update never precedes its relocation");
    for pair in trace.windows(2) {
        assert!(pair[1].chain - pair[0].chain <= 1, "one link per drain {trace:?}");
    }
}

#[test]
fn an_editor_callback_awaiting_its_script_keeps_its_class_and_does_not_hold_ticks() {
    // SLICE A (A-T4): a PendingAsset on an end-of-cycle callback's script is
    // requeued at the front as Deferred (not as a packet-synchronous Script),
    // so later ticks still apply; on recovery it runs exactly once.
    let (_fixture, client, release, worker) = remote_cache((12, HELD_CB));
    let (mut client, mut connection, mut peer) = logged_in(client);
    client.scripts.source.cache.wait_for_recovery(false);
    mounted(&mut client, &mut connection, &mut peer);
    peer.send(&[run_script(START_HELD)]);
    let held = |client: &ClientState| matches!(client.interfaces.scripts.front(),
        Some(crate::interface_state::PendingEvent::Deferred { event, .. }) if event.widget == G_EDIT);
    let mut ready = false;
    for _ in 0..200 {
        connection.poll_elapsed(&mut client, 1).unwrap();
        if held(&client) && connection.pending_script.is_none() { ready = true; break; }
        thread::sleep(Duration::from_millis(2));
    }
    assert!(ready, "the callback was not requeued as an end-of-cycle entry: {:?}", client.interfaces.scripts.front().map(|e| format!("{e:?}")));
    peer.send(&(0..5).map(|_| tick()).collect::<Vec<_>>());
    let trace = poll_until(&mut client, &mut connection, 200, |s| s.tick == 5);
    assert_eq!(client.scripts.variables.player(HELD_V), 0, "the callback waited for its script {trace:?}");
    assert!(held(&client), "still queued with its class");
    release.send(()).unwrap();
    let mut ran = false;
    for _ in 0..2000 {
        connection.poll_elapsed(&mut client, 1).unwrap();
        if client.scripts.variables.player(HELD_V) == 1 { ran = true; break; }
        thread::sleep(Duration::from_millis(2));
    }
    assert!(ran, "the recovered callback ran");
    for _ in 0..5 { connection.poll_elapsed(&mut client, 1).unwrap(); }
    assert_eq!(client.scripts.variables.player(HELD_V), 1, "exactly once");
    drop(connection);
    drop(client);
    drop(release);
    assert!(join_peer(worker), "the released held JS5 reply was written");
}

#[test]
fn a_queued_editor_callback_never_runs_after_its_static_widget_unloads() {
    // SLICE A (A-T5): a chain link for static G_EDIT is queued when CLOSE_SUB
    // unloads the group and OPEN_SUB mounts a replacement with the same id.
    // The link must be dropped, not run against the unloaded id (whose by-id
    // op would reload the group) or the replacement.
    let (_fixture, client) = local_client();
    let (mut client, mut connection, mut peer) = logged_in(client);
    mounted(&mut client, &mut connection, &mut peer);
    let original = client.interfaces.runtime[&G_EDIT].instance;
    peer.send(&[run_script(START)]);
    let mut trace = poll_until(&mut client, &mut connection, 200, |s| s.chain >= 3);
    peer.send(&[close_sub(ROOT_W), open_sub(ROOT_W, SUB_GROUP, 1), tick()]);
    let after = poll_until(&mut client, &mut connection, (CHAIN_LIMIT as usize) * 4 + 50,
        |s| s.tick == 1 && s.mounted_at == Some(ROOT_W));
    let base = trace.len();
    trace.extend(after);
    let closed = base + first(&trace[base..], "unmount", |s| s.mounted_at.is_none());
    let before = trace[closed - 1].chain;
    assert!(before < CHAIN_LIMIT,
        "F1: CLOSE_SUB was held until the editor chain ended, so no link was queued at the unload; trace: {trace:?}");
    for _ in 0..10 {
        connection.poll_elapsed(&mut client, 1).unwrap();
        trace.push(seen(&client, &connection));
    }
    for s in &trace[closed..] {
        assert_eq!(s.chain, before, "a queued link ran after its widget unloaded (replacement mutation) {trace:?}");
    }
    let replacement = client.interfaces.runtime.get(&G_EDIT).map(|r| r.instance);
    assert!(replacement.is_some_and(|instance| instance != original), "a new G_EDIT instance was mounted");
    assert_ne!(client.interfaces.focused_editor(), Some(G_EDIT), "the stale link did not focus the replacement");
    assert!(connection.reported.iter().any(|m| m.starts_with(STALE_REPORT)),
        "the dropped link is reported: {:?}", connection.reported);
    assert_eq!(client.server_tick, 1);
}

/// Applies a dynamic-editor RUNCLIENTSCRIPT and a tick through the real poll,
/// then polls five more times so any queued callback has been reached.
fn dynamic_stale_case(script: u16) -> (Fixture, ClientState, Connection) {
    let (fixture, client) = local_client();
    let (mut client, mut connection, mut peer) = logged_in(client);
    mounted(&mut client, &mut connection, &mut peer);
    peer.send(&[run_script(script), tick()]);
    poll_until(&mut client, &mut connection, 200, |s| s.tick == 1);
    for _ in 0..5 {
        connection.poll_elapsed(&mut client, 1).unwrap();
        thread::sleep(Duration::from_millis(2));
    }
    (fixture, client, connection)
}

#[test]
fn a_queued_editor_callback_never_runs_after_its_dynamic_slot_is_replaced() {
    // SLICE A (A-T6): the slot holds a new instance before the callback is reached.
    let (_fixture, client, connection) = dynamic_stale_case(MAKE_REPLACE);
    assert!(client.interfaces.dynamic.contains_key(&(G_CONTAINER, 0)), "the replacement occupies the slot");
    assert_eq!(client.scripts.variables.player(STALE_V), 0, "the stale dynamic callback ran");
    assert!(connection.reported.iter().any(|m| m.starts_with(STALE_REPORT)),
        "the dropped callback is reported: {:?}", connection.reported);
}

#[test]
fn a_nested_dynamic_editor_callback_runs_while_its_slot_is_current() {
    // SLICE A (F1 positive): the slot check must use create_child_inner's key,
    // not the editor's parent or inherited logical id.
    let (_fixture, client, connection) = dynamic_stale_case(NEST_MAKE);
    let layer = client.interfaces.dynamic[&(G_CONTAINER, 0)];
    let nested = client.interfaces.dynamic[&(G_CONTAINER, 1)];
    assert_eq!(client.interfaces.widgets[&nested].parent, Some(layer), "the editor's parent is the dynamic layer");
    assert!(client.scripts.variables.player(NEST_V) >= 1, "the current nested callback ran");
    assert!(!connection.reported.iter().any(|m| m.starts_with(STALE_REPORT)),
        "a current callback is never reported stale: {:?}", connection.reported);
}

#[test]
fn a_nested_dynamic_editor_callback_never_runs_after_detach() {
    // SLICE A (F1 negative, detach).
    let (_fixture, client, connection) = dynamic_stale_case(NEST_DETACH);
    assert!(!client.interfaces.dynamic.contains_key(&(G_CONTAINER, 1)), "the nested editor was detached");
    assert_eq!(client.scripts.variables.player(NEST_V), 0, "the detached nested callback ran");
    assert!(connection.reported.iter().any(|m| m.starts_with(STALE_REPORT)), "{:?}", connection.reported);
}

#[test]
fn a_nested_dynamic_editor_callback_never_runs_after_its_layer_is_replaced() {
    // SLICE A (F1 negative, replacement).
    let (_fixture, client, connection) = dynamic_stale_case(NEST_REPLACE);
    assert!(client.interfaces.dynamic.contains_key(&(G_CONTAINER, 0)), "a replacement layer holds slot 0");
    assert!(!client.interfaces.dynamic.contains_key(&(G_CONTAINER, 1)), "the nested editor went with the old layer");
    assert_eq!(client.scripts.variables.player(NEST_V), 0, "the replaced nested callback ran");
    assert!(connection.reported.iter().any(|m| m.starts_with(STALE_REPORT)), "{:?}", connection.reported);
}

#[test]
fn a_queued_editor_callback_never_runs_after_its_dynamic_slot_is_detached() {
    // SLICE A (A-T6 detach): the ps.ac dynamic-slot check, with no replacement.
    let (_fixture, client, connection) = dynamic_stale_case(MAKE_DETACH);
    assert!(!client.interfaces.dynamic.contains_key(&(G_CONTAINER, 0)), "the slot was detached");
    assert_eq!(client.scripts.variables.player(STALE_V), 0, "the detached dynamic callback ran");
    assert!(connection.reported.iter().any(|m| m.starts_with(STALE_REPORT)),
        "the dropped callback is reported: {:?}", connection.reported);
}

#[test]
fn ordinary_packets_respect_the_100_per_poll_budget_then_eof_reconnects() {
    let (_fixture, client) = local_client();
    let (mut client, mut connection, mut peer) = logged_in(client);
    peer.send(&(0..150).map(|_| tick()).collect::<Vec<_>>());
    peer.socket.shutdown(Shutdown::Write).unwrap();
    // Receipt diagnostic on failure: poll_until reports the last observed state.
    let trace = poll_until(&mut client, &mut connection, 200, |s| s.tick == 150);
    let mut previous = 0;
    let mut applying = 0;
    for s in &trace {
        assert!(s.tick - previous <= 100, "budget exceeded {trace:?}");
        if s.tick > previous { applying += 1; }
        previous = s.tick;
    }
    assert!(applying >= 2, "150 packets need at least two polls");
    // EOF after the batch: every packet applied first, then the reconnect path.
    for _ in 0..200 {
        if connection.is_reconnecting() { break; }
        connection.poll_elapsed(&mut client, 1).unwrap();
        thread::sleep(Duration::from_millis(2));
    }
    assert!(connection.is_reconnecting());
    assert_eq!(client.server_tick, 150);
    // The reconnect worker targets the dropped loopback listener (refused);
    // the production poll joins it, so it cannot outlive the test unnoticed.
    reap_connect_worker(&mut client, &mut connection);
    assert_eq!(client.server_tick, 150, "nothing applied after EOF");
}

#[test]
fn move_and_close_run_lifecycle_before_the_following_update() {
    let (_fixture, client) = local_client();
    let (mut client, mut connection, mut peer) = logged_in(client);
    mounted(&mut client, &mut connection, &mut peer);
    let base = client.scripts.variables.player(LIFE_COUNT);
    peer.send(&[move_sub(ROOT_W, ROOT_ALT), set_text(G_TEXT, "moved")]);
    let trace = poll_until(&mut client, &mut connection, 200, |s| s.text == "moved");
    let moved = first(&trace, "relocation", |s| s.mounted_at == Some(ROOT_ALT));
    assert!(trace[moved].life > base, "relocation lifecycle ran in the move poll {trace:?}");
    assert_ne!(trace[moved].text, "moved");
    assert!(first(&trace, "SETTEXT moved", |s| s.text == "moved") > moved, "CURRENT: update waits for the lifecycle drain");

    let base = client.scripts.variables.player(LIFE_COUNT);
    peer.send(&[close_sub(ROOT_ALT), set_text(G_TEXT, "closed")]);
    let trace = poll_until(&mut client, &mut connection, 200, |s| s.text == "closed");
    let closed = first(&trace, "unmount", |s| s.mounted_at.is_none());
    assert!(trace[closed].life > base, "close lifecycle ran in the close poll {trace:?}");
    assert!(first(&trace, "SETTEXT closed", |s| s.text == "closed") > closed);
}

// ---- #175 draw custody (private test adapter) ----------------------------------
// The real Connection, packets and paint; the adapter reads and publishes through
// its public poll/frame entry points, exactly as viewport.rs calls them.

/// A test-owned directory under the run's TMP root, created exclusively (an
/// existing directory is never adopted). Drop removes only the enumerated
/// files the test can create, then the directory itself if it is now empty
/// (never recursively); anything left is reported for a cleanup request.
#[cfg(feature = "private-test-adapter")]
struct CustodyDir { path: std::path::PathBuf, files: Vec<String> }
#[cfg(feature = "private-test-adapter")]
impl Drop for CustodyDir {
    fn drop(&mut self) {
        for name in &self.files { let _ = std::fs::remove_file(self.path.join(name)); }
        if let Err(error) = std::fs::remove_dir(&self.path) {
            eprintln!("draw-custody test directory left for cleanup: {} ({error})", self.path.display());
        }
    }
}
#[cfg(feature = "private-test-adapter")]
fn custody_dir(name: &str, captures: u64) -> CustodyDir {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("draw-custody-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    // Every name the adapter or this test can write in these scenarios.
    let mut files: Vec<String> = ["command.txt", "command.tmp", "receipt.txt", "receipt.tmp", "state.json", "state.tmp"]
        .map(String::from).to_vec();
    for k in 1..=captures {
        for suffix in [".png", "-draw.json", "-camera.json", "-child-worlds.json", "-minimap.json", "-minimap.png"] {
            files.push(format!("frame-{k}{suffix}"));
            files.push(format!("frame-{k}{suffix}.tmp"));
        }
    }
    CustodyDir { path, files }
}
#[cfg(feature = "private-test-adapter")]
fn custody_command(dir: &std::path::Path, line: &str) {
    std::fs::write(dir.join("command.tmp"), line).unwrap();
    std::fs::rename(dir.join("command.tmp"), dir.join("command.txt")).unwrap();
}

#[cfg(feature = "private-test-adapter")]
#[test]
fn draw_custody_world_hop_is_a_reset_boundary() {
    use crate::draw_custody::Boundary;
    let (_fixture, client) = local_client();
    let (mut client, mut connection, mut peer) = logged_in(client);
    let before = connection.draw_custody.scene_epoch;
    peer.send(&[tick(), world_hop(), tick()]);
    for _ in 0..200 {
        if connection.hopping { break; }
        connection.poll_elapsed(&mut client, 1).unwrap();
        thread::sleep(Duration::from_millis(2));
    }
    assert!(connection.hopping);
    let custody = &connection.draw_custody;
    assert!(custody.scene_epoch > before, "the hop's clear_world advanced the epoch");
    assert_eq!((custody.epoch_reason, custody.camera_generation, custody.world_loaded), (Some(Boundary::Reset), None, false));
    reap_connect_worker(&mut client, &mut connection);
}

#[cfg(feature = "private-test-adapter")]
#[test]
fn draw_custody_logout_is_a_reset_boundary() {
    let (_fixture, client) = local_client();
    let (mut client, mut connection, mut peer) = logged_in(client);
    preload_title(&mut client, &mut connection);
    let before = connection.draw_custody.scene_epoch;
    peer.send(&[tick(), (LOGOUT, Vec::new())]);
    for _ in 0..200 {
        if connection.logged_out.is_some() { break; }
        connection.poll_elapsed(&mut client, 1).unwrap();
        thread::sleep(Duration::from_millis(2));
    }
    assert!(connection.logged_out.is_some());
    assert!(connection.draw_custody.scene_epoch > before);
    assert_eq!(connection.draw_custody.epoch_reason, Some(crate::draw_custody::Boundary::Reset));
}

#[cfg(feature = "private-test-adapter")]
#[test]
fn adapter_capture_links_only_its_own_frame_paint_and_publishes_custody_before_the_png() {
    let (_fixture, client) = local_client();
    let (mut client, mut connection, _peer) = logged_in(client);
    let dir = custody_dir("capture", 4);
    let mut adapter = crate::private_adapter::Adapter::new(dir.path.clone()).unwrap();
    let read = |name: &str| std::fs::read_to_string(dir.path.join(name)).unwrap();

    // Frame 1: a real paint with no loaded world, then the capture of that raster.
    custody_command(&dir.path, "1 capture\n");
    adapter.poll(&mut client, &mut connection, false).unwrap();
    assert!(read("receipt.txt").contains("status=accepted"));
    let state = read("state.json");
    assert!(state.contains("\"draw_custody\":{\"schema\":\"draw-custody/1\",\"scope\":\"connection counters when this receipt was written; not a captured raster\""), "{state}");
    assert!(state.contains("\"message_events\":["), "legacy keys stay");
    let raster = connection.paint(&mut client, 64, 48).unwrap();
    assert_eq!(connection.draw_custody.paint_serial, 1);
    adapter.frame(&raster, &client, Some(&connection)).unwrap();
    let draw = read("frame-1-draw.json");
    assert!(draw.contains("\"link\":\"painted\"") && draw.contains("\"outcome\":\"no_world\""), "{draw}");
    assert!(draw.contains("\"raster_draw_generation\":null") && draw.contains("\"world_raster_current\":false"));
    assert!(dir.path.join("frame-1.png").exists() && dir.path.join("frame-1-camera.json").exists());

    // Frame 2: no paint ran for this raster, so it cannot claim frame 1's paint.
    custody_command(&dir.path, "2 capture\n");
    adapter.poll(&mut client, &mut connection, false).unwrap();
    adapter.frame(&raster, &client, Some(&connection)).unwrap();
    let draw = read("frame-2-draw.json");
    assert!(draw.contains("\"link\":\"unpainted\"") && draw.contains("\"paint_serial\":null"), "{draw}");

    // Frame 3: no live connection; frame 4: still no new paint.
    custody_command(&dir.path, "3 capture\n");
    adapter.poll(&mut client, &mut connection, false).unwrap();
    adapter.frame(&raster, &client, None).unwrap();
    assert!(read("frame-3-draw.json").contains("\"link\":\"not_live\""));
    custody_command(&dir.path, "4 capture\n");
    adapter.poll(&mut client, &mut connection, false).unwrap();
    adapter.frame(&raster, &client, Some(&connection)).unwrap();
    assert!(read("frame-4-draw.json").contains("\"link\":\"unpainted\""));
    assert!(std::fs::read_dir(&dir.path).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().ends_with(".tmp")));
}

#[cfg(feature = "private-test-adapter")]
#[test]
fn adapter_frames_without_a_capture_still_advance_custody_bookkeeping() {
    let (_fixture, client) = local_client();
    let (mut client, mut connection, _peer) = logged_in(client);
    let dir = custody_dir("bookkeeping", 1);
    let mut adapter = crate::private_adapter::Adapter::new(dir.path.clone()).unwrap();
    // A painted frame with no capture pending consumes its paint.
    let raster = connection.paint(&mut client, 64, 48).unwrap();
    adapter.frame(&raster, &client, Some(&connection)).unwrap();
    custody_command(&dir.path, "1 capture\n");
    adapter.poll(&mut client, &mut connection, false).unwrap();
    adapter.frame(&raster, &client, Some(&connection)).unwrap();
    let draw = std::fs::read_to_string(dir.path.join("frame-1-draw.json")).unwrap();
    assert!(draw.contains("\"link\":\"unpainted\"") && draw.contains("\"adapter_frame\":2"), "{draw}");
}

// ---- #181 client journal wiring (private test adapter) ---------------------------

#[cfg(feature = "private-test-adapter")]
mod journal {
    //! #181 W1: the client journal on the real outbound seam (`poll_elapsed` drain) and its
    //! single export boundary. S* tests drive `Connection` directly; T* tests drive the
    //! adapter's attach/export on the same production poll. Every fixture here is already
    //! logged in (`logged_in` polls to Success BEFORE any attach), so these tests do not
    //! show what the viewport records before login. The viewport places attach at the first
    //! main-loop poll; connect/login/EULA/title polls there are recorded calls. The viewport
    //! placement is a static review item, not exercised here.
    //! Off-vs-on comparisons (S1/S3/S9) assert an identical login history: `logged_in`'s
    //! login poll count is timing dependent, so a slow login is a loud failure, never a pass.
    use super::*;
    use crate::client_journal::{
        check_framing, worst_case_bytes, CallCounts, DequeueEvent, Fact, FramingError, Invalidation, Limits,
        Phase, Recorder, StageOutcome, Status, MAX_RECORDS,
    };
    use crate::live_runtime::JournalSlot;
    use crate::private_adapter::Adapter;
    use crate::protocol::Packet;

    const WINDOW_STATUS: u16 = 10;
    const KEEPALIVE: u16 = 89;
    /// Not a client packet: `send_event` refuses it before any cipher or buffer change.
    const UNKNOWN_OPCODE: u16 = 1000;
    /// Shared with the Python reader suite (tools/harness-client/tests/test_client_journal.py).
    const W1_EXPORT_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/client-journal-w1-export.jsonl");

    fn window_status() -> Packet { Packet { opcode: WINDOW_STATUS, payload: vec![0, 2, 253, 1, 247] } }
    fn keepalive() -> Packet { Packet { opcode: KEEPALIVE, payload: Vec::new() } }
    fn unknown() -> Packet { Packet { opcode: UNKNOWN_OPCODE, payload: Vec::new() } }
    fn recorder(max_records: u32) -> Recorder {
        let limits = Limits::new(max_records, 0, worst_case_bytes(max_records, 0)).unwrap();
        Recorder::new(limits, Phase::new(0, "session").unwrap()).unwrap()
    }
    fn facts(connection: &Connection) -> Vec<Fact> {
        match &connection.journal {
            JournalSlot::Recording(recorder) => recorder.records().iter().map(|r| *r.fact()).collect(),
            _ => panic!("journal is not recording"),
        }
    }
    fn dequeues(facts: &[Fact]) -> Vec<(DequeueEvent, StageOutcome)> {
        facts.iter().filter_map(|f| match *f { Fact::Dequeue { event, outcome } => Some((event, outcome)), _ => None }).collect()
    }
    fn calls(facts: &[Fact]) -> usize { facts.iter().filter(|f| matches!(f, Fact::Call { .. })).count() }
    /// Everything the client has written to the peer so far (bounded by a short read timeout).
    fn client_bytes(peer: &mut Peer) -> Vec<u8> {
        peer.socket.set_read_timeout(Some(Duration::from_millis(300))).unwrap();
        let mut out = Vec::new();
        let mut buffer = [0; 4096];
        loop {
            match peer.socket.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => out.extend_from_slice(&buffer[..n]),
                Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => break,
                Err(e) => panic!("peer read failed: {e}"),
            }
        }
        peer.socket.set_read_timeout(Some(SETUP)).unwrap();
        out
    }

    struct Run { before: Vec<u8>, after: Vec<u8>, result: String, left: usize, connection: Connection }
    /// One scripted outbound sequence (None = cipher discard) over a fresh login, with or
    /// without a recorder. `before` is the login history both runs must share.
    fn scripted(journal: Option<Recorder>, events: Vec<Option<Packet>>) -> Run {
        let (_fixture, client) = local_client();
        let (mut client, mut connection, mut peer) = logged_in(client);
        let before = client_bytes(&mut peer);
        if let Some(recorder) = journal { connection.attach_journal(recorder).unwrap(); }
        for event in events {
            match event { Some(packet) => client.outbound.push_back(packet), None => client.outbound.discard_cipher() }
        }
        // Zero-tick polls: the drain runs; keepalive idle time does not advance.
        let result = format!("{:?}", connection.poll_elapsed(&mut client, 0));
        if result.starts_with("Ok") { connection.poll_elapsed(&mut client, 0).unwrap(); }
        let after = client_bytes(&mut peer);
        Run { before, after, result, left: client.outbound.len(), connection }
    }

    #[test]
    fn journal_recording_is_passive_on_the_outbound_wire() {
        // S1: identical bytes, order and cipher position with the journal on and off.
        let events = || vec![Some(window_status()), None, Some(keepalive())];
        let off = scripted(None, events());
        let on = scripted(Some(recorder(MAX_RECORDS)), events());
        assert_eq!(off.before, on.before, "both runs start from the same deterministic login history");
        assert!(!off.after.is_empty(), "the scripted packets reached the peer");
        assert_eq!((&off.after, &off.result, off.left), (&on.after, &on.result, on.left));
        let facts = facts(&on.connection);
        assert_eq!(dequeues(&facts).len(), 3);
        assert_eq!(calls(&facts), 2, "one call record per poll");
    }

    #[test]
    fn one_journal_record_per_outbound_pop_and_none_for_direct_sends() {
        // S2: N packets plus one cipher discard give N+1 dequeue records in pop order; a
        // direct session send (not an outbound pop) gives none.
        let (_fixture, client) = local_client();
        let (mut client, mut connection, _peer) = logged_in(client);
        connection.attach_journal(recorder(MAX_RECORDS)).unwrap();
        connection.session.send(KEEPALIVE, &[]).unwrap();
        client.outbound.push_back(window_status());
        client.outbound.discard_cipher();
        client.outbound.push_back(keepalive());
        client.outbound.push_back(window_status());
        connection.poll_elapsed(&mut client, 0).unwrap();
        let facts = facts(&connection);
        assert_eq!(dequeues(&facts), vec![
            (DequeueEvent::Packet { opcode: WINDOW_STATUS, payload_len: 5 }, StageOutcome::Staged),
            (DequeueEvent::DiscardCipher, StageOutcome::Staged),
            (DequeueEvent::Packet { opcode: KEEPALIVE, payload_len: 0 }, StageOutcome::Staged),
            (DequeueEvent::Packet { opcode: WINDOW_STATUS, payload_len: 5 }, StageOutcome::Staged),
        ]);
        assert_eq!(calls(&facts), 1);
        assert_eq!(client.outbound.len(), 0);
    }

    #[test]
    fn a_refused_staging_is_recorded_once_and_its_error_returned_unchanged() {
        // S3 (unit-level forced refusal): recorded once as refused_other; the same error is
        // returned; the following event is neither popped nor recorded.
        let events = || vec![Some(window_status()), Some(unknown()), Some(keepalive())];
        let off = scripted(None, events());
        let on = scripted(Some(recorder(MAX_RECORDS)), events());
        assert!(on.result.starts_with("Err"), "{}", on.result);
        assert_eq!(off.before, on.before);
        assert_eq!((&off.after, &off.result, off.left), (&on.after, &on.result, on.left));
        assert_eq!(on.left, 1, "the event after the refusal stays queued");
        let facts = facts(&on.connection);
        assert_eq!(dequeues(&facts), vec![
            (DequeueEvent::Packet { opcode: WINDOW_STATUS, payload_len: 5 }, StageOutcome::Staged),
            (DequeueEvent::Packet { opcode: UNKNOWN_OPCODE, payload_len: 0 }, StageOutcome::RefusedOther),
        ]);
        assert_eq!(calls(&facts), 1, "the failed call still ends with its call record");
    }

    #[test]
    fn an_empty_zero_tick_poll_still_records_one_call_with_zero_counts() {
        // S8.
        let (_fixture, client) = local_client();
        let (mut client, mut connection, _peer) = logged_in(client);
        connection.attach_journal(recorder(MAX_RECORDS)).unwrap();
        connection.poll_elapsed(&mut client, 0).unwrap();
        let facts = facts(&connection);
        assert_eq!(facts.len(), 2, "{facts:?}");
        match facts[1] {
            Fact::Call { call, counts, .. } => assert_eq!((call, counts), (0, CallCounts::default())),
            other => panic!("expected a call record, got {other:?}"),
        }
    }

    #[test]
    fn journal_overflow_invalidates_and_drops_without_changing_the_game() {
        // S9: tiny limits invalidate at the first refused ordinal and drop afterwards; the
        // game's bytes, result and queue equal the journal-off run.
        let events = || vec![Some(window_status()), Some(keepalive())];
        let off = scripted(None, events());
        let on = scripted(Some(recorder(3)), events());
        assert_eq!(off.before, on.before);
        assert_eq!((&off.after, &off.result, off.left), (&on.after, &on.result, on.left));
        match &on.connection.journal {
            JournalSlot::Recording(recorder) => {
                assert_eq!(recorder.status(), Status::Invalid { reason: Invalidation::RecordOverflow, at_ordinal: 2 });
                assert_eq!(recorder.records().len(), 2, "Initial and the first dequeue; the Final slot stays reserved");
            }
            _ => panic!("journal is not recording"),
        }
    }

    /// Every name the adapter can write in these flows (`stop` command, receipt, state,
    /// journal export). Inspect/action/capture outputs are not reachable here.
    const DIR_NAMES: [&str; 8] = ["command.txt", "command.tmp", "receipt.txt", "receipt.tmp",
        "state.json", "state.tmp", "journal.jsonl", "journal-export.json"];
    /// A test-owned command directory under TMP, created exclusively. `close` is the checked
    /// success-path teardown and asserts the directory is gone; Drop is best-effort only
    /// (unwinding), never recursive, and never panics, so a test panic is not masked.
    struct JournalDir { path: PathBuf, closed: bool }
    impl JournalDir {
        fn new(name: &str) -> Self {
            let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("client-journal-{name}-{}-{nanos}", std::process::id()));
            std::fs::create_dir(&path).unwrap();
            Self { path, closed: false }
        }
        fn file(&self, name: &str) -> PathBuf { self.path.join(name) }
        fn command(&self, line: &str) {
            std::fs::write(self.file("command.tmp"), line).unwrap();
            std::fs::rename(self.file("command.tmp"), self.file("command.txt")).unwrap();
        }
        fn remove_known(&self, checked: bool) {
            for name in DIR_NAMES {
                let path = self.path.join(name);
                let result = match std::fs::symlink_metadata(&path) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(error) => Err(error),
                    Ok(meta) if meta.is_dir() => std::fs::remove_dir(&path),
                    Ok(_) => std::fs::remove_file(&path),
                };
                if checked { result.unwrap_or_else(|error| panic!("teardown {}: {error}", path.display())); }
            }
        }
        fn close(mut self) {
            self.remove_known(true);
            std::fs::remove_dir(&self.path)
                .unwrap_or_else(|error| panic!("test directory not empty at teardown {}: {error}", self.path.display()));
            assert!(!self.path.exists(), "test directory removed");
            self.closed = true;
        }
    }
    impl Drop for JournalDir {
        fn drop(&mut self) {
            if !self.closed {
                self.remove_known(false);
                let _ = std::fs::remove_dir(&self.path);
            }
        }
    }
    fn adapter_session(name: &str) -> (Fixture, ClientState, Connection, Peer, JournalDir, Adapter) {
        let (fixture, client) = local_client();
        let (client, connection, peer) = logged_in(client);
        let dir = JournalDir::new(name);
        let adapter = Adapter::new(dir.path.clone()).unwrap();
        (fixture, client, connection, peer, dir, adapter)
    }
    fn exported(dir: &JournalDir) -> (Vec<u8>, String) {
        (std::fs::read(dir.file("journal.jsonl")).unwrap(), std::fs::read_to_string(dir.file("journal-export.json")).unwrap())
    }
    /// Independent oracle for the commit marker the adapter writes last.
    fn marker(stop: u64, bytes: &[u8], records: u32, status: &str) -> String {
        use sha2::{Digest, Sha256};
        let sha = format!("{:x}", Sha256::digest(bytes));
        format!(
            "{{\"format\":\"client-journal-export/1\",\"stop_sequence\":{stop},\"journal_bytes\":{},\"journal_sha256\":\"{sha}\",\"records\":{records},\"final_status\":\"{status}\"}}\n",
            bytes.len())
    }

    #[test]
    fn deterministic_export_matches_the_shared_cross_language_fixture() {
        // U1: the actual adapter export of a deterministic flow is byte-identical to the
        // fixture the Python reader suite also reads. Baseline clocks are pinned from source:
        // ClientState::load sets actor_cycle 1 and server_tick 0, and nothing in poll_elapsed
        // advances actor_cycle (only tick_actor_sequences, from the viewport).
        let (_fixture, mut client, mut connection, _peer, dir, mut adapter) = adapter_session("u1");
        assert_eq!((client.actor_cycle, client.server_tick), (1, 0), "pinned baseline clocks");
        assert_eq!(client.outbound.len(), 0, "nothing queued before attach");
        adapter.attach_journal(&mut connection).unwrap();
        connection.poll_elapsed(&mut client, 0).unwrap();
        connection.poll_elapsed(&mut client, 0).unwrap();
        dir.command("1 stop\n");
        adapter.poll(&mut client, &mut connection, false).unwrap();
        assert!(adapter.stop_ready());
        adapter.export_journal(&mut connection).unwrap();
        let (bytes, commit) = exported(&dir);
        assert_eq!(bytes, W1_EXPORT_FIXTURE, "exported journal == shared fixture");
        assert_eq!(commit, marker(1, &bytes, 4, "valid"));
        dir.close();
    }

    #[test]
    fn stop_with_a_pending_receipt_keeps_recording_until_export() {
        // T1: while the stop receipt is pending (unit-level transient failure: receipt.tmp is a
        // directory, so the write is refused and retried), frames keep polling and recording.
        let (_fixture, mut client, mut connection, _peer, dir, mut adapter) = adapter_session("t1");
        adapter.attach_journal(&mut connection).unwrap();
        std::fs::create_dir(dir.file("receipt.tmp")).unwrap();
        dir.command("1 stop\n");
        let mut polls = 0u32;
        for _ in 0..4 {
            connection.poll_elapsed(&mut client, 1).unwrap();
            polls += 1;
            adapter.poll(&mut client, &mut connection, false).unwrap();
            assert!(!adapter.stop_ready(), "stop waits for its pending receipt");
        }
        let pending = facts(&connection);
        assert_eq!(calls(&pending), polls as usize, "every frame while the receipt is pending is recorded");
        assert!(!pending.iter().any(|f| matches!(f, Fact::Final { .. })), "no Final before the boundary");
        std::fs::remove_dir(dir.file("receipt.tmp")).unwrap();
        connection.poll_elapsed(&mut client, 1).unwrap();
        polls += 1;
        adapter.poll(&mut client, &mut connection, false).unwrap();
        assert!(adapter.stop_ready());
        adapter.export_journal(&mut connection).unwrap();
        let (bytes, commit) = exported(&dir);
        let framing = check_framing(&bytes, MAX_RECORDS).unwrap();
        assert_eq!((framing.final_claims.calls, framing.final_claims.status), (polls, Status::Valid));
        assert_eq!(commit, marker(1, &bytes, framing.lines, "valid"));
        assert!(std::fs::read_to_string(dir.file("receipt.txt")).unwrap().contains("status=accepted"));
        dir.close();
    }

    #[test]
    fn a_poll_after_export_is_refused_and_records_nothing() {
        // T2: after the boundary every poll is refused before any work.
        let (_fixture, mut client, mut connection, _peer, dir, mut adapter) = adapter_session("t2");
        adapter.attach_journal(&mut connection).unwrap();
        connection.poll_elapsed(&mut client, 1).unwrap();
        adapter.export_journal(&mut connection).unwrap();
        let before = exported(&dir);
        client.outbound.push_back(window_status());
        let refused = connection.poll_elapsed(&mut client, 1);
        assert_eq!(format!("{refused:?}"), format!("{:?}", Err::<(), _>(crate::Error::Invalid("client journal: recorded seam after final"))));
        assert_eq!(client.outbound.len(), 1, "the refused call did no work");
        assert!(matches!(connection.journal, JournalSlot::Exported));
        assert_eq!(exported(&dir), before, "the committed export is unchanged");
        dir.close();
    }

    #[test]
    fn attach_refuses_prior_outputs_and_export_is_one_shot() {
        // T3.
        let (_fixture, mut client, mut connection, _peer, dir, mut adapter) = adapter_session("t3");
        for name in ["journal.jsonl", "journal-export.json"] {
            std::fs::write(dir.file(name), b"").unwrap();
            assert!(adapter.attach_journal(&mut connection).is_err(), "prior {name} refuses the attach");
            assert!(matches!(connection.journal, JournalSlot::Off));
            std::fs::remove_file(dir.file(name)).unwrap();
        }
        adapter.attach_journal(&mut connection).unwrap();
        assert!(adapter.attach_journal(&mut connection).is_err(), "a second attach is refused");
        connection.poll_elapsed(&mut client, 1).unwrap();
        adapter.export_journal(&mut connection).unwrap();
        let first = exported(&dir);
        assert!(adapter.export_journal(&mut connection).is_err(), "a second export is refused");
        assert!(matches!(connection.journal, JournalSlot::Exported));
        assert_eq!(exported(&dir), first);
        dir.close();
    }

    #[test]
    fn a_terminal_error_before_the_boundary_exports_nothing() {
        // T4, unit level only: a recorded error and the recorder's drop write nothing; only
        // export_journal writes. That an error exit in the viewport never reaches export is a
        // static property (the `?` exits at the poll and adapter calls precede the boundary).
        let (_fixture, mut client, mut connection, _peer, dir, mut adapter) = adapter_session("t4");
        adapter.attach_journal(&mut connection).unwrap();
        client.outbound.push_back(unknown());
        assert!(connection.poll_elapsed(&mut client, 1).is_err());
        drop(adapter);
        drop(connection);
        assert!(!dir.file("journal.jsonl").exists() && !dir.file("journal-export.json").exists());
        dir.close();
    }

    #[test]
    fn an_export_without_its_commit_marker_is_uncommitted() {
        // T5: the commit marker is written last; if it cannot be created the export fails and
        // the journal bytes alone are not a committed export.
        let (_fixture, mut client, mut connection, _peer, dir, mut adapter) = adapter_session("t5");
        adapter.attach_journal(&mut connection).unwrap();
        connection.poll_elapsed(&mut client, 1).unwrap();
        std::fs::create_dir(dir.file("journal-export.json")).unwrap();
        assert!(adapter.export_journal(&mut connection).is_err());
        assert!(dir.file("journal.jsonl").is_file());
        assert!(!dir.file("journal-export.json").is_file(), "no commit marker file");
        assert!(connection.poll_elapsed(&mut client, 1).is_err(), "the boundary was still crossed");
        dir.close();
    }

    #[test]
    fn exported_bytes_match_the_commit_marker_and_every_truncation_fails() {
        // T6: the marker binds the exact bytes; every strict prefix fails the Rust framing
        // check. The cross-language reading of the same bytes is U1 + the Python suite.
        let (_fixture, mut client, mut connection, _peer, dir, mut adapter) = adapter_session("t6");
        adapter.attach_journal(&mut connection).unwrap();
        client.outbound.push_back(window_status());
        connection.poll_elapsed(&mut client, 1).unwrap();
        dir.command("1 stop\n");
        adapter.poll(&mut client, &mut connection, false).unwrap();
        assert!(adapter.stop_ready());
        connection.poll_elapsed(&mut client, 1).unwrap();
        adapter.export_journal(&mut connection).unwrap();
        let (bytes, commit) = exported(&dir);
        let framing = check_framing(&bytes, MAX_RECORDS).unwrap();
        assert_eq!(commit, marker(1, &bytes, framing.lines, "valid"));
        for end in 0..bytes.len() {
            assert!(check_framing(&bytes[..end], MAX_RECORDS).is_err(), "prefix of {end} bytes accepted");
        }
        assert!(matches!(check_framing(&bytes[..bytes.len() - 1], MAX_RECORDS), Err(FramingError::PartialEof { .. })));
        dir.close();
    }

    #[test]
    fn attach_on_an_already_logged_in_fixture_records_initial_then_calls_only() {
        // T7, fixture level only: the fixture logs in before attach, so its login polls are
        // not in this journal. In the viewport, attach precedes login and those polls ARE
        // recorded calls; that placement is a static review item.
        let (_fixture, mut client, mut connection, _peer, dir, mut adapter) = adapter_session("t7");
        adapter.attach_journal(&mut connection).unwrap();
        for _ in 0..3 { connection.poll_elapsed(&mut client, 1).unwrap(); }
        let facts = facts(&connection);
        match facts[0] {
            Fact::Initial { phase, .. } => assert_eq!((phase.ordinal(), phase.label()), (0, "session")),
            other => panic!("expected Initial, got {other:?}"),
        }
        assert!(facts[1..].iter().all(|f| matches!(f, Fact::Call { .. })), "{facts:?}");
        assert_eq!(calls(&facts), 3);
        dir.close();
    }

    #[test]
    fn eof_reconnect_keeps_one_journal_phase_and_no_final_before_export() {
        // T8: transport EOF and the reconnect join stay in the one session phase. Events the
        // client discards with its queue on reconnect are never popped, so never recorded.
        let (_fixture, client) = local_client();
        let (mut client, mut connection, peer) = logged_in(client);
        connection.attach_journal(recorder(MAX_RECORDS)).unwrap();
        peer.socket.shutdown(Shutdown::Write).unwrap();
        let mut polls = 0;
        for _ in 0..200 {
            if connection.is_reconnecting() { break; }
            connection.poll_elapsed(&mut client, 1).unwrap();
            polls += 1;
            thread::sleep(Duration::from_millis(2));
        }
        assert!(connection.is_reconnecting());
        reap_connect_worker(&mut client, &mut connection);
        let facts = facts(&connection);
        assert_eq!(facts.iter().filter(|f| matches!(f, Fact::Initial { .. })).count(), 1);
        assert!(!facts.iter().any(|f| matches!(f, Fact::Final { .. })));
        let numbers: Vec<u32> = facts.iter().filter_map(|f| match *f { Fact::Call { call, .. } => Some(call), _ => None }).collect();
        assert!(numbers.len() > polls, "polls across EOF and the reconnect join are all recorded");
        assert_eq!(numbers, (0..numbers.len() as u32).collect::<Vec<_>>());
    }

    #[test]
    fn overflow_then_export_is_committed_invalid_not_missing() {
        // T9.
        let (_fixture, mut client, mut connection, _peer, dir, mut adapter) = adapter_session("t9");
        connection.attach_journal(recorder(3)).unwrap();
        for _ in 0..3 { connection.poll_elapsed(&mut client, 1).unwrap(); }
        dir.command("1 stop\n");
        adapter.poll(&mut client, &mut connection, false).unwrap();
        assert!(adapter.stop_ready());
        adapter.export_journal(&mut connection).unwrap();
        let (bytes, commit) = exported(&dir);
        let framing = check_framing(&bytes, MAX_RECORDS).unwrap();
        assert_eq!(framing.final_claims.status, Status::Invalid { reason: Invalidation::RecordOverflow, at_ordinal: 2 });
        assert_eq!(commit, marker(1, &bytes, framing.lines, "invalid"));
        dir.close();
    }
}
