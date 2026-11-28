//! Live interface-event drain (#11) on a real ClientState, real cache, real VM
//! scripts and the real interface queue. No timing assertions.
use super::{deferred_open, drain_interface_events, Drain};
use crate::{
    client_state::{ClientState, EventExecution},
    crypto::crc32,
    disk_cache::DiskCache,
    interface_packet::SubInterface,
    interface_state::{PendingEvent, ScriptEvent},
    vm::Value,
    widget::{Argument, Widget},
};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

/// 601 instructions (seven 100-instruction steps), ending with varp 0 = 11.
const LONG: u16 = 1;
/// Copies varp 0 into varp 1, then sets varp 0 = 22.
const SHORT: u16 = 2;
/// Jumps to itself until the VM instruction limit.
const SPIN: u16 = 3;
/// Sets varp 0 = 33.
const AFTER: u16 = 4;
/// Sets varp 0 = 44; listed in the reference table but served only by recovery.
const HELD: u16 = 5;
/// Editor 1437 listener on EDITOR_A that changes EDITOR_A's own text (self-requeue).
const SELF_CHAIN: u16 = 6;
/// Editor listener on EDITOR_A that changes EDITOR_B's text.
const A_TO_B: u16 = 7;
/// Editor listener on EDITOR_B that changes EDITOR_A's text.
const B_TO_A: u16 = 8;
/// Requests a modal close (opcode 3103), as a script button would.
const CLOSER: u16 = 9;
/// Lifecycle listener: varp 3 = varp 0, then varp 0 = 66.
const LIFE: u16 = 10;
/// onLoad listener: varp 2 = varp 0, then varp 0 = 77.
const ONLOAD: u16 = 11;
const EDITOR_A: u32 = 1 << 16;
const EDITOR_B: u32 = (1 << 16) | 1;
/// Chains stop after this many links, so a missing boundary fails an assertion instead of hanging.
const CHAIN_LIMIT: i32 = 40;

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
    /// Opcode 0..=99 with an int operand (21 takes a byte).
    I(u16, i32),
    /// Opcode 3: push string.
    S(&'static str),
    /// Opcode >= 100 with its byte operand.
    X(u16),
}
fn encode(ops: &[Op]) -> Vec<u8> {
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
fn script(ops: &[(u16, i32)]) -> Vec<u8> {
    encode(&ops.iter().map(|&(op, arg)| Op::I(op, arg)).collect::<Vec<_>>())
}
/// Counts runs in `counter`; until CHAIN_LIMIT, alternates `target`'s editor
/// text between "a" and "b" (state in `toggle`) with if_settext 2112, so the
/// target's 1437 listener is queued after every run.
fn chain(target: u32, counter: i32, toggle: i32) -> Vec<u8> {
    use Op::*;
    let target = target as i32;
    encode(&[
        I(1, counter), I(0, 1), X(4000), I(2, counter),
        I(1, counter), I(0, CHAIN_LIMIT), I(32, 15),
        I(1, toggle), I(0, 0), I(8, 6),
        I(0, 0), I(2, toggle), S("a"), I(0, target), X(2112), I(21, 0),
        I(0, 1), I(2, toggle), S("b"), I(0, target), X(2112), I(21, 0),
        I(21, 0),
    ])
}
fn scripts() -> BTreeMap<u16, Vec<u8>> {
    let mut long = Vec::new();
    for _ in 0..300 { long.extend([(0, 11), (2, 0)]); }
    long.push((21, 0));
    BTreeMap::from([
        (LONG, script(&long)),
        (SHORT, script(&[(1, 0), (2, 1), (0, 22), (2, 0), (21, 0)])),
        (SPIN, script(&[(6, -1)])),
        (AFTER, script(&[(0, 33), (2, 0), (21, 0)])),
        (HELD, script(&[(0, 44), (2, 0), (21, 0)])),
        (SELF_CHAIN, chain(EDITOR_A, 5, 6)),
        (A_TO_B, chain(EDITOR_B, 8, 9)),
        (B_TO_A, chain(EDITOR_A, 10, 11)),
        (CLOSER, encode(&[Op::X(3103), Op::I(21, 0)])),
        (LIFE, script(&[(1, 0), (2, 3), (0, 66), (2, 0), (21, 0)])),
        (ONLOAD, script(&[(1, 0), (2, 2), (0, 77), (2, 0), (21, 0)])),
    ])
}
/// Group containers by archive: everything ClientState::load needs plus scripts.
fn archives() -> BTreeMap<u8, BTreeMap<u16, Vec<u8>>> {
    let mut archives = BTreeMap::<u8, BTreeMap<u16, Vec<u8>>>::new();
    for group in [9, 10, 11, 13, 16, 19, 32, 33] {
        archives.entry(2).or_default().insert(group, container(&[0]));
    }
    for (id, bytes) in scripts() {
        archives.entry(12).or_default().insert(id, container(&bytes));
    }
    let mut defaults = vec![2];
    for index in 0..11 { defaults.extend((if index == 7 { 0u16 } else { 32767 }).to_be_bytes()); }
    defaults.push(0);
    archives.entry(17).or_default().insert(3, container(&defaults));
    let mut sprites = Vec::new();
    for color in 1..=8 { sprites.extend([0, color, 0]); }
    for color in 1..=8 { sprites.extend([0, 0, color]); }
    sprites.extend(16u16.to_be_bytes()); sprites.extend(16u16.to_be_bytes()); sprites.push(8);
    for value in [2u16, 3, 2, 1] { for _ in 0..8 { sprites.extend(value.to_be_bytes()); } }
    sprites.extend(8u16.to_be_bytes());
    archives.entry(8).or_default().insert(0, container(&sprites));
    archives
}
fn references(archives: &BTreeMap<u8, BTreeMap<u16, Vec<u8>>>) -> BTreeMap<u8, Vec<u8>> {
    archives.iter().map(|(&archive, groups)| {
        let mut reference = vec![6]; reference.extend(1u32.to_be_bytes()); reference.push(0);
        reference.extend((groups.len() as u16).to_be_bytes());
        let mut last = 0;
        for &id in groups.keys() { reference.extend((id - last).to_be_bytes()); last = id; }
        for bytes in groups.values() { reference.extend(crc32(bytes).to_be_bytes()); }
        for _ in groups { reference.extend(1u32.to_be_bytes()); }
        for _ in groups { reference.extend(1u16.to_be_bytes()); }
        for _ in groups { reference.extend(0u16.to_be_bytes()); }
        (archive, container(&reference))
    }).collect()
}
/// Writes the disk cache; `withheld` groups stay listed in their reference
/// table but are absent from disk.
fn write_disk(withheld: &[(u8, u16)]) -> Fixture {
    // Tests run concurrently in one process and the clock can repeat at its
    // granularity, so a per-process counter keeps every fixture directory unique.
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("live-drain-{}-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
    std::fs::create_dir(&root).unwrap();
    let all = archives();
    let mut disk: BTreeMap<u8, BTreeMap<u16, Vec<u8>>> = all.iter().map(|(&archive, groups)| {
        (archive, groups.iter().filter(|(&id, _)| !withheld.contains(&(archive, id)))
            .map(|(&id, bytes)| (id, bytes.clone())).collect())
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
fn local() -> (Fixture, ClientState) {
    let fixture = write_disk(&[]);
    let client = ClientState::load(DiskCache::open(&fixture.0, 1 << 20).unwrap()).unwrap();
    (fixture, client)
}
fn wire(archive: u8, group: u16, container: &[u8]) -> Vec<u8> {
    let mut bytes = vec![archive, (group >> 8) as u8, group as u8];
    for &byte in container {
        if bytes.len() % 512 == 0 { bytes.push(255); }
        bytes.push(byte);
    }
    bytes
}
/// Disk cache plus a real JS5 recovery source that serves the master index and
/// reference tables at once, but holds the HELD script until released.
fn remote() -> (Fixture, ClientState, mpsc::Sender<()>, thread::JoinHandle<()>) {
    let fixture = write_disk(&[(12, HELD)]);
    let all = archives();
    let references = references(&all);
    let mut master = vec![0; (*all.keys().last().unwrap() as usize + 1) * 8];
    for (&archive, reference) in &references {
        let offset = archive as usize * 8;
        master[offset..offset + 4].copy_from_slice(&crc32(reference).to_be_bytes());
        master[offset + 4..offset + 8].copy_from_slice(&1u32.to_be_bytes());
    }
    let held = all[&12][&HELD].clone();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (release, released) = mpsc::channel();
    let worker = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
        let mut handshake = [0; 21];
        stream.read_exact(&mut handshake).unwrap();
        stream.write_all(&[0]).unwrap();
        // Other replies keep flowing while HELD waits for its release.
        let writer = std::sync::Arc::new(std::sync::Mutex::new(stream.try_clone().unwrap()));
        let held_writer = std::sync::Arc::clone(&writer);
        let held_reply = thread::spawn(move || {
            if released.recv_timeout(Duration::from_secs(20)).is_ok() {
                let _ = held_writer.lock().unwrap().write_all(&wire(12, HELD, &held));
            }
        });
        let mut request = [0; 4];
        while stream.read_exact(&mut request).is_ok() {
            let (archive, group) = (request[1], u16::from_be_bytes([request[2], request[3]]));
            let data = match (archive, group) {
                (255, 255) => container(&master),
                (255, archive) => references[&(archive as u8)].clone(),
                (12, HELD) => continue,
                other => panic!("unexpected JS5 request {other:?}"),
            };
            if writer.lock().unwrap().write_all(&wire(archive, group, &data)).is_err() { break; }
        }
        let _ = held_reply.join();
    });
    let mut cache = DiskCache::open_remote(&fixture.0, 1 << 20, address).unwrap();
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache).unwrap();
    // The live loop polls without waiting for recovery (viewport.rs).
    client.scripts.source.cache.wait_for_recovery(false);
    (fixture, client, release, worker)
}
fn no_boundary(client: &ClientState) {
    assert!(!client.interfaces.scripts.iter().any(|e| matches!(e, PendingEvent::Boundary)),
        "a generation boundary must never outlive its drain");
}
fn drain(client: &mut ClientState, failures: &mut Vec<String>) -> Drain {
    let mut pending: Option<EventExecution> = None;
    let drained = drain_interface_events(&mut pending, client, failures).unwrap();
    assert!(pending.is_none());
    no_boundary(client);
    drained
}
/// Two ready editors in group 1. `listeners` gives each one's 1437 script.
fn editors(client: &mut ClientState, listeners: [Option<u16>; 2]) {
    let widgets = [EDITOR_A, EDITOR_B].map(|id| { let mut w = Widget::empty(id, 0); w.kind = 12; w });
    client.interfaces.install_group(1, widgets.into()).unwrap();
    for (id, listener) in [EDITOR_A, EDITOR_B].into_iter().zip(listeners) {
        let runtime = client.interfaces.runtime.get_mut(&id).unwrap();
        // A ready editor carries font metrics, as the production font load provides.
        let editor = runtime.editor.as_mut().unwrap();
        editor.initialize(0, crate::font::Metrics { advances: [6; 256], ascent: 12, kerning: None }).unwrap();
        editor.text.take_events();
        if let Some(script) = listener {
            runtime.listeners.insert(1437, (vec![Value::Int(i32::from(script))], None));
        }
    }
}
fn listener_event(widget: u32, script: u16) -> PendingEvent {
    PendingEvent::Script(ScriptEvent {
        widget,
        context: crate::script_event::Context { widget: Some((widget, -1)), ..Default::default() },
        arguments: vec![Value::Int(i32::from(script))],
    })
}
fn queue(client: &mut ClientState, ids: &[u16]) {
    for &id in ids {
        client.interfaces.scripts.push_back(PendingEvent::Script(ScriptEvent {
            widget: 0,
            context: Default::default(),
            arguments: vec![Value::Int(i32::from(id))],
        }));
    }
}

#[test]
fn one_drain_completes_multi_step_events_in_queue_order_and_opens_deferred_packets() {
    let (_fixture, mut client) = local();
    let mut pending: Option<EventExecution> = None;
    queue(&mut client, &[LONG, SHORT]);
    assert!(!deferred_open(&pending, &client), "queued events hold deferred packets");
    let mut failures = Vec::new();
    let Drain { steps, deferred } = drain_interface_events(&mut pending, &mut client, &mut failures).unwrap();
    assert!(failures.is_empty(), "{failures:?}");
    assert!(steps >= 8, "LONG needs several 100-instruction steps, got {steps}");
    assert_eq!(deferred, 0);
    no_boundary(&client);
    assert!(pending.is_none() && client.interfaces.scripts.is_empty());
    assert_eq!(client.scripts.variables.player(1), 11, "SHORT ran after LONG finished");
    assert_eq!(client.scripts.variables.player(0), 22);
    // The deferred loop runs before the drain in poll_elapsed, so the next poll applies held packets.
    assert!(deferred_open(&pending, &client));
}

#[test]
fn instruction_limit_failure_is_reported_and_later_events_still_run() {
    let (_fixture, mut client) = local();
    let mut pending = None;
    queue(&mut client, &[SPIN, AFTER]);
    let mut failures = Vec::new();
    drain_interface_events(&mut pending, &mut client, &mut failures).unwrap();
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(failures[0].contains("script instruction limit"), "{failures:?}");
    assert_eq!(client.scripts.variables.player(0), 33);
    assert!(pending.is_none() && deferred_open(&pending, &client));
    no_boundary(&client);
}

#[test]
fn event_failures_before_a_queue_error_are_kept_for_reporting() {
    let (_fixture, mut client) = local();
    let mut pending = None;
    queue(&mut client, &[999]);
    // A real queue error: an onLoad array slot that was never loaded.
    let unloaded = std::rc::Rc::new(std::cell::RefCell::new(vec![None]));
    client.interfaces.scripts.push_back(PendingEvent::OnLoadArray(unloaded, 0));
    let mut failures = Vec::new();
    let drained = drain_interface_events(&mut pending, &mut client, &mut failures);
    assert!(drained.is_err(), "queue error propagates");
    assert_eq!(failures.len(), 1, "the missing script's failure is still available: {failures:?}");
    no_boundary(&client);
}

#[test]
fn pending_script_asset_yields_without_running_later_events_then_resumes_in_order() {
    let (_fixture, mut client, release, worker) = remote();
    let mut pending = None;
    queue(&mut client, &[HELD, SHORT]);
    let mut failures = Vec::new();
    for _ in 0..3 {
        let drained = drain_interface_events(&mut pending, &mut client, &mut failures).unwrap();
        assert_eq!(drained, Drain { steps: 0, deferred: 2 }, "a PendingAsset event yields instead of spinning");
        assert!(pending.is_none());
        no_boundary(&client);
        assert_eq!(client.interfaces.scripts.len(), 2, "HELD is requeued at the front; SHORT waits");
        assert!(matches!(&client.interfaces.scripts[0], PendingEvent::Script(e) if matches!(e.arguments[0], Value::Int(id) if id == i32::from(HELD))));
        assert!(!deferred_open(&pending, &client));
    }
    assert_eq!(client.scripts.variables.player(0), 0);
    release.send(()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !client.interfaces.scripts.is_empty() {
        assert!(Instant::now() < deadline, "recovered script never became available");
        drain_interface_events(&mut pending, &mut client, &mut failures).unwrap();
        no_boundary(&client);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(failures.is_empty(), "{failures:?}");
    assert_eq!(client.scripts.variables.player(1), 44, "HELD ran before SHORT");
    assert_eq!(client.scripts.variables.player(0), 22);
    assert!(deferred_open(&pending, &client));
    drop(client);
    drop(release);
    worker.join().unwrap();
}

#[test]
fn a_self_requeuing_editor_listener_runs_once_per_drain() {
    let (_fixture, mut client) = local();
    editors(&mut client, [Some(SELF_CHAIN), None]);
    client.interfaces.scripts.push_back(listener_event(EDITOR_A, SELF_CHAIN));
    let mut failures = Vec::new();
    for poll in 1..CHAIN_LIMIT {
        let drained = drain(&mut client, &mut failures);
        assert!(failures.is_empty(), "{failures:?}");
        assert_eq!(client.scripts.variables.player(5), poll, "exactly one link per drain");
        assert_eq!(drained.deferred, 1, "the requeued listener waits for the next poll");
        assert!(matches!(&client.interfaces.scripts[0], PendingEvent::Deferred { event, .. } if event.widget == EDITOR_A),
            "the requeued link is an end-of-cycle editor callback");
        // #88 Slice A: an end-of-cycle editor chain no longer holds deferred packets.
        assert!(deferred_open(&None, &client), "packets stay open during an active editor chain");
    }
    let drained = drain(&mut client, &mut failures);
    assert_eq!(client.scripts.variables.player(5), CHAIN_LIMIT);
    assert_eq!(drained.deferred, 0);
    assert!(deferred_open(&None, &client), "packets open once the chain is exhausted");
}

#[test]
fn mutually_requeuing_editor_listeners_advance_one_generation_per_drain() {
    let (_fixture, mut client) = local();
    editors(&mut client, [Some(A_TO_B), Some(B_TO_A)]);
    client.interfaces.scripts.push_back(listener_event(EDITOR_A, A_TO_B));
    let mut failures = Vec::new();
    for poll in 1..=6 {
        drain(&mut client, &mut failures);
        assert!(failures.is_empty(), "{failures:?}");
        let (a, b) = (client.scripts.variables.player(8), client.scripts.variables.player(10));
        assert_eq!((a, b), ((poll + 1) / 2, poll / 2), "A and B alternate, one link per drain");
        assert_eq!(client.interfaces.scripts.len(), 1);
    }
}

/// Root group 0 (widget 0, lifecycle listener LIFE), optionally with a modal
/// sub (group 2) mounted under it.
fn root_with_lifecycle(client: &mut ClientState, modal: bool) {
    client.interfaces.install_group(0, vec![Widget::empty(0, 0)]).unwrap();
    client.interfaces.runtime.get_mut(&0).unwrap().listeners.insert(1424, (vec![Value::Int(i32::from(LIFE))], None));
    client.interfaces.root = Some(0);
    if modal {
        client.interfaces.install_group(2, vec![Widget::empty(2 << 16, 4)]).unwrap();
        client.interfaces.children.insert(0, SubInterface { parent: 0, group: 2, kind: 0 });
    }
}

#[test]
fn mutual_chain_seeded_with_both_listeners_runs_each_once_per_drain() {
    let (_fixture, mut client) = local();
    editors(&mut client, [Some(A_TO_B), Some(B_TO_A)]);
    client.interfaces.scripts.push_back(listener_event(EDITOR_A, A_TO_B));
    client.interfaces.scripts.push_back(listener_event(EDITOR_B, B_TO_A));
    let mut failures = Vec::new();
    for poll in 1..=5 {
        let drained = drain(&mut client, &mut failures);
        assert!(failures.is_empty(), "{failures:?}");
        assert_eq!((client.scripts.variables.player(8), client.scripts.variables.player(10)), (poll, poll),
            "both initial entries run, their appends wait");
        assert_eq!(drained.deferred, 2);
    }
}

#[test]
fn an_already_running_execution_with_an_empty_queue_is_bounded_too() {
    let (_fixture, mut client) = local();
    editors(&mut client, [Some(SELF_CHAIN), None]);
    let PendingEvent::Script(event) = listener_event(EDITOR_A, SELF_CHAIN) else { unreachable!() };
    let mut pending = Some(client.begin_event(event).unwrap());
    assert!(client.interfaces.scripts.is_empty());
    let mut failures = Vec::new();
    let drained = drain_interface_events(&mut pending, &mut client, &mut failures).unwrap();
    assert!(failures.is_empty(), "{failures:?}");
    assert!(pending.is_none());
    no_boundary(&client);
    assert_eq!(client.scripts.variables.player(5), 1, "the running execution finished; its append waits");
    assert_eq!(drained.deferred, 1);
}

#[test]
fn a_completed_generation_prunes_released_lifecycle_registries() {
    let (_fixture, mut client) = local();
    root_with_lifecycle(&mut client, false);
    client.interfaces.queue_lifecycle(1424).unwrap();
    queue(&mut client, &[SHORT]);
    let mut failures = Vec::new();
    drain(&mut client, &mut failures);
    assert!(failures.is_empty(), "{failures:?}");
    assert_eq!(client.scripts.variables.player(1), 66, "lifecycle ran, then SHORT");
    assert!(client.interfaces.scripts.is_empty());
    assert_eq!(client.interfaces.lifecycle_registry_sizes(), [0, 0, 0], "traversal handles released and pruned");
}

#[test]
fn script_requested_close_runs_its_callbacks_before_the_queued_tail_in_one_drain() {
    let (_fixture, mut client) = local();
    root_with_lifecycle(&mut client, true);
    queue(&mut client, &[CLOSER, SHORT]);
    let mut failures = Vec::new();
    let drained = drain(&mut client, &mut failures);
    assert!(failures.is_empty(), "{failures:?}");
    assert_eq!(drained.deferred, 0, "close_modal's take/extend keeps its callbacks in this generation");
    assert!(!client.interfaces.children.contains_key(&0), "modal closed");
    assert!(!client.interfaces.closing_modal);
    assert_eq!(client.scripts.variables.player(3), 0, "the close lifecycle callback ran first");
    assert_eq!(client.scripts.variables.player(1), 66, "SHORT ran after the close callback");
    assert_eq!(client.scripts.variables.player(0), 22);
    assert!(client.interfaces.scripts.is_empty());
}

#[test]
fn queued_mount_runs_onload_and_lifecycle_before_the_queued_tail_in_one_drain() {
    let (_fixture, mut client) = local();
    root_with_lifecycle(&mut client, false);
    let mut onload = Widget::empty(3 << 16, 4);
    onload.listeners[0] = Some(vec![Argument::Int(i32::from(ONLOAD))]);
    client.interfaces.install_group(3, vec![onload]).unwrap();
    client.interfaces.scripts.push_back(PendingEvent::OpenSub(SubInterface { parent: 0, group: 3, kind: 1 }));
    queue(&mut client, &[SHORT]);
    let mut failures = Vec::new();
    let drained = drain(&mut client, &mut failures);
    assert!(failures.is_empty(), "{failures:?}");
    assert_eq!(drained.deferred, 0, "resume_mutation's tail extraction keeps mount work in this generation");
    assert_eq!(client.scripts.variables.player(2), 0, "onLoad ran first");
    assert_eq!(client.scripts.variables.player(3), 77, "lifecycle ran after onLoad");
    assert_eq!(client.scripts.variables.player(1), 66, "SHORT ran after both");
    assert_eq!(client.scripts.variables.player(0), 22);
    assert!(client.interfaces.children.contains_key(&0));
    assert!(client.interfaces.scripts.is_empty());
}

/// An end-of-cycle callback for `widget`, as queue_editor_listener produces it.
fn deferred_entry(client: &ClientState, widget: u32) -> PendingEvent {
    PendingEvent::Deferred {
        event: ScriptEvent {
            widget,
            context: crate::script_event::Context { widget: Some((widget, -1)), ..Default::default() },
            arguments: vec![Value::Int(i32::from(SHORT))],
        },
        instance: client.interfaces.runtime[&widget].instance,
    }
}
fn front_is_deferred(client: &ClientState) -> bool {
    matches!(client.interfaces.scripts.front(), Some(PendingEvent::Deferred { event, .. }) if event.widget == EDITOR_A)
}

// #88 Slice A per-site class preservation (F3): each production re-push site
// pops the queue front, which here is an end-of-cycle callback; it must be
// re-pushed as Deferred, not as a packet-synchronous Script.
#[test]
fn move_sub_requeues_a_front_editor_callback_as_end_of_cycle() {
    let (_fixture, mut client) = local();
    editors(&mut client, [None, None]);
    root_with_lifecycle(&mut client, true);
    let entry = deferred_entry(&client, EDITOR_A);
    client.interfaces.scripts.push_back(entry);
    client.interfaces.move_sub(0, EDITOR_B).unwrap();
    assert!(front_is_deferred(&client), "move_sub kept the class: {:?}", client.interfaces.scripts.front());
    assert!(client.interfaces.scripts.iter().any(|e| matches!(e, PendingEvent::MoveSub { .. })));
    assert!(!deferred_open(&None, &client), "the queued relocation still gates packets");
}

#[test]
fn open_top_resume_requeues_a_front_editor_callback_as_end_of_cycle() {
    let (_fixture, mut client) = local();
    editors(&mut client, [None, None]);
    root_with_lifecycle(&mut client, false);
    let entry = deferred_entry(&client, EDITOR_A);
    client.interfaces.scripts.push_back(entry);
    client.apply(crate::game_packet::Update::Interface(crate::interface_packet::Update::OpenTop(0))).unwrap();
    assert!(front_is_deferred(&client), "resume_until_callback kept the class: {:?}", client.interfaces.scripts.front());
    assert!(client.interfaces.scripts.iter().any(|e| matches!(e, PendingEvent::OnLoadArray(..))));
    assert!(!deferred_open(&None, &client), "the queued root onLoad still gates packets");
}

#[test]
fn close_modal_keeps_a_queued_editor_callback_end_of_cycle() {
    // close_modal re-pushes only the first callback of its own isolated
    // CloseModal queue (packet-synchronous by construction); the editor
    // callback travels in the taken tail and must keep its class.
    let (_fixture, mut client) = local();
    editors(&mut client, [None, None]);
    root_with_lifecycle(&mut client, true);
    let entry = deferred_entry(&client, EDITOR_A);
    client.interfaces.scripts.push_back(entry);
    client.interfaces.close_modal().unwrap();
    assert!(matches!(client.interfaces.scripts.front(), Some(PendingEvent::Script(_))), "the close lifecycle callback is packet-synchronous");
    assert!(matches!(client.interfaces.scripts.back(), Some(PendingEvent::Deferred { event, .. }) if event.widget == EDITOR_A));
}

#[test]
fn an_empty_queue_takes_no_generation_and_leaves_no_boundary() {
    let (_fixture, mut client) = local();
    let mut failures = Vec::new();
    assert_eq!(drain(&mut client, &mut failures), Drain::default());
    assert!(client.interfaces.next_event().unwrap().is_none());
}
