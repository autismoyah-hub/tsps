//! Opt-in local validation adapter. Uses the same menu actions as the native UI;
//! never injects desktop input or changes server state directly.
use crate::{client_state::ClientState, context_menu::Action, raster::Raster, Result};
use std::{io, io::Read, path::PathBuf};
fn transient_io(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::PermissionDenied | io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
    ) || (cfg!(windows) && matches!(error.raw_os_error(), Some(5 | 32 | 33)))
}
/// #181: the journal export and its commit marker, written once per command directory.
const JOURNAL: &str = "journal.jsonl";
const JOURNAL_COMMIT: &str = "journal-export.json";
fn write_new(path: &std::path::Path, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
fn quoted(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub struct Adapter {
    pub headless: bool,
    directory: PathBuf,
    sequence: u64,
    capture: Option<u64>,
    receipt_pending: Option<String>,
    inspect: bool,
    scene_reported: u64,
    /// IF3 close in flight; its receipt waits for the driver's outcome.
    close: Option<crate::adapter_action::CloseDriver>,
    /// Admission or delivery error text from the consuming tick, for the receipt.
    close_error: Option<String>,
    /// IF3 native-menu close in flight; its receipt waits for its bounded outcome.
    menu_close: Option<MenuCloseState>,
    /// `chat` in flight (issue #12); its receipt waits for the bounded outcome.
    chat: Option<native_input::ChatDriver>,
    /// `minimap` in flight (issue #12); its receipt waits for the bounded outcome.
    minimap: Option<native_input::MinimapDriver>,
    /// #175: links each frame to its own paint; advanced on every `frame` call.
    frames: crate::draw_custody::FrameTracker,
    /// #175: `draw_custody` counters read at the start of the poll that writes a receipt.
    custody_state: Option<String>,
    pub stop: bool,
}
/// A `close` resolved to an exact native-menu Close op (adapter_action menu close).
#[derive(Clone, Debug)]
enum MenuCloseState {
    /// The onop script is not local yet; re-resolved every poll up to CLOSE_ARM_POLLS.
    Waiting { sequence: u64, polls: u32, error: String },
    /// Dispatched with exactly [95]; waits up to CLOSE_RESULT_POLLS for the unload.
    Watching { sequence: u64, target: crate::adapter_action::MenuCloseTarget, polls: u32 },
    /// Ended by a title/reconnect frame before completion; `dispatched` records
    /// whether the menu op (and its [95]) had already been queued.
    Aborted { sequence: u64, dispatched: bool },
}
impl Adapter {
    pub fn new(directory: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&directory)?;
        Ok(Self { headless:false, directory, sequence:0, capture:None, receipt_pending:None,
            inspect:false, scene_reported:0, close:None, close_error:None, menu_close:None, chat:None, minimap:None,
            frames:Default::default(), custody_state:None, stop:false })
    }
    pub fn stop_ready(&self)->bool {
        self.stop && self.receipt_pending.is_none() && self.capture.is_none() && self.close.is_none()
            && self.menu_close.is_none() && self.chat.is_none() && self.minimap.is_none()
    }
    /// #181: attach a fresh client journal to the live connection before the first
    /// main-loop poll. Connect, login, EULA and title polls in the main loop ARE recorded
    /// as `call` records (no login payload or handshake record exists). One process, one
    /// command directory, one journal: earlier journal outputs in the directory refuse.
    pub fn attach_journal(&mut self, live: &mut crate::live_runtime::Connection) -> Result<()> {
        use crate::client_journal::{worst_case_bytes, Limits, Phase, Recorder, MAX_RECORDS};
        for name in [JOURNAL, JOURNAL_COMMIT] {
            match std::fs::symlink_metadata(self.directory.join(name)) {
                Ok(_) => return Err(crate::Error::Invalid("client journal output already present")),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        let limits = Limits::new(MAX_RECORDS, 0, worst_case_bytes(MAX_RECORDS, 0))
            .map_err(|_| crate::Error::Invalid("client journal limits"))?;
        let phase = Phase::new(0, "session").map_err(|_| crate::Error::Invalid("client journal phase"))?;
        let recorder = Recorder::new(limits, phase).map_err(|_| crate::Error::Invalid("client journal allocation"))?;
        live.attach_journal(recorder)
    }
    /// #181: the single finalization boundary (viewport loop exit once `stop_ready`).
    /// Takes the recorder, so every later poll is refused, then writes `journal.jsonl`
    /// (create_new) and the `journal-export.json` commit marker last. One attempt, no
    /// retry; an error ends the process with that error. A failed write or sync can leave
    /// a journal and/or a partial marker: only a complete canonical marker that matches the
    /// journal bytes, with process exit 0, is a commit (see docs/harness/client-journal.md).
    pub fn export_journal(&mut self, live: &mut crate::live_runtime::Connection) -> Result<()> {
        use crate::client_journal::{check_framing, Status, MAX_RECORDS};
        use sha2::{Digest, Sha256};
        let journal = live.take_journal()?.finish();
        let bytes = journal.encode().map_err(|_| crate::Error::Invalid("client journal encode"))?;
        let framing = check_framing(&bytes, MAX_RECORDS).map_err(|_| crate::Error::Invalid("client journal framing"))?;
        let status = match journal.status() { Status::Valid => "valid", Status::Invalid { .. } => "invalid" };
        write_new(&self.directory.join(JOURNAL), &bytes)?;
        let marker = format!(
            "{{\"format\":\"client-journal-export/1\",\"stop_sequence\":{},\"journal_bytes\":{},\"journal_sha256\":\"{:x}\",\"records\":{},\"final_status\":\"{status}\"}}\n",
            self.sequence, bytes.len(), Sha256::digest(&bytes), framing.lines);
        write_new(&self.directory.join(JOURNAL_COMMIT), marker.as_bytes())?;
        Ok(())
    }
    /// H-pre: immediately before `tick_interfaces_elapsed_with(ticks, ..)`.
    /// Errors become a bounded close refusal (owned press released), never an exit.
    pub fn before_tick(&mut self, client: &mut ClientState, tick_input: &mut crate::input::Input, ticks: u32) {
        if let Some(close) = self.close.as_mut() {
            if let Some(error) = close.before_tick(client, tick_input, ticks) { self.close_error = Some(error); }
        }
        if let Some(chat) = self.chat.as_mut() { chat.before_tick(client, tick_input, ticks); }
        if let Some(minimap) = self.minimap.as_mut() { minimap.before_tick(client, tick_input, ticks); }
    }
    /// Per internal tick, from the tick batch's own callback: observes exactly the
    /// tick that consumed a press or release.
    pub fn observe_tick(&mut self, client: &ClientState) {
        if let Some(close) = self.close.as_mut() { close.observe_tick(client); }
        if let Some(chat) = self.chat.as_mut() { chat.observe_tick(client); }
    }
    /// H-post: after the whole tick batch; releases a delivery no tick observed.
    pub fn after_frame(&mut self, tick_input: &mut crate::input::Input) {
        if let Some(close) = self.close.as_mut() { close.after_frame(tick_input); }
    }
    /// After every adapter poll, ticking frame or not: bounds a delivered press.
    pub fn release_watchdog(&mut self, tick_input: &mut crate::input::Input) {
        if let Some(close) = self.close.as_mut() { close.watchdog(tick_input); }
        if let Some(minimap) = self.minimap.as_mut() { minimap.watchdog(tick_input); }
    }
    /// H-reset: title/reconnect frames release a delivered press and end the close.
    pub fn abort_input(&mut self, tick_input: &mut crate::input::Input) {
        if let Some(close) = self.close.as_mut() { close.abort(tick_input); }
        if let Some(chat) = self.chat.as_mut() { chat.abort(tick_input); }
        if let Some(minimap) = self.minimap.as_mut() { minimap.abort(tick_input); }
        // A menu close holds no input; it only ends with its own receipt.
        match self.menu_close {
            Some(MenuCloseState::Waiting { sequence, .. }) =>
                self.menu_close = Some(MenuCloseState::Aborted { sequence, dispatched: false }),
            Some(MenuCloseState::Watching { sequence, .. }) =>
                self.menu_close = Some(MenuCloseState::Aborted { sequence, dispatched: true }),
            _ => {}
        }
    }
    /// Bounded per-widget refusal diagnostic for the open modal groups.
    /// Also records the read-only native menu at each Close op centre.
    fn write_close_diagnostics(&self, client: &mut ClientState, live: &mut crate::live_runtime::Connection, sequence: u64, menu: &crate::adapter_action::MenuClose) {
        let pointer = client.input.pointer;
        let text = match crate::adapter_action::close_diagnostics(client, pointer, menu) {
            Ok(text) => text,
            Err(error) => format!("{{\"error\":{}}}", quoted(&error.to_string())),
        };
        let native = match crate::adapter_action::close_menu_probe(&mut (&mut *client, &mut *live)) {
            Ok(text) => text,
            Err(error) => format!("{{\"error\":{}}}", quoted(&error.to_string())),
        };
        self.write_diagnostic("close-diagnostics", format!("{{\"sequence\":{sequence},\"diagnostics\":{text},\"native_menu\":{native}}}"));
    }
    fn write_close_mode(&self, sequence: u64, text: String) {
        self.write_diagnostic("close-mode", format!("{{\"sequence\":{sequence},{text}}}"));
    }
    /// Revalidates and dispatches a resolved menu close through the real native menu.
    fn start_menu_close(&mut self, client: &mut ClientState, live: &mut crate::live_runtime::Connection, ready: bool,
        sequence: u64, target: crate::adapter_action::MenuCloseTarget) -> Result<()> {
        // Every open modal group (e.g. the bank's 15 beside 12) is recorded, not assumed.
        let mut before: Vec<u16> = crate::adapter_action::open_modal_groups(client).into_iter().collect();
        before.sort_unstable();
        let status = match crate::adapter_action::dispatch_menu_close(&mut (&mut *client, &mut *live), target) {
            Ok(Ok(packets)) if packets == [95] => {
                self.write_close_mode(sequence, format!("\"mode\":\"native-menu\",\"dispatched\":true,\"widget\":{},\"instance\":{},\"operation\":{},\"modal_groups_before\":{before:?},\"packets\":[95]",
                    target.widget, target.instance, target.operation));
                self.menu_close = Some(MenuCloseState::Watching { sequence, target, polls: 0 });
                return Ok(());
            }
            Ok(Ok(packets)) => {
                // Dispatched, not accepted: record what was actually queued.
                self.write_close_mode(sequence, format!("\"mode\":\"native-menu\",\"dispatched\":true,\"accepted\":false,\"widget\":{},\"instance\":{},\"operation\":{},\"modal_groups_before\":{before:?},\"packets\":{packets:?}",
                    target.widget, target.instance, target.operation));
                format!("close menu op dispatched but queued unexpected packets {packets:?} (not accepted)")
            }
            Ok(Err(reason)) => {
                self.write_close_diagnostics(client, live, sequence, &crate::adapter_action::MenuClose::Refused(reason.clone()));
                format!("IF3 menu close refused: {reason}")
            }
            Err(error) => format!("client action failed: {error}"),
        };
        self.sequence = sequence;
        self.receipt(client, ready, &status)
    }
    /// Advances an in-flight menu close. Ok(true) while it still owns the poll.
    fn poll_menu_close(&mut self, client: &mut ClientState, live: &mut crate::live_runtime::Connection, ready: bool) -> Result<bool> {
        use crate::adapter_action::{MenuClose, CLOSE_ARM_POLLS, CLOSE_RESULT_POLLS};
        let Some(state) = self.menu_close.take() else { return Ok(false); };
        let (sequence, status) = match state {
            MenuCloseState::Aborted { sequence, dispatched: false } => (sequence, "close aborted before dispatch".to_owned()),
            MenuCloseState::Aborted { sequence, dispatched: true } =>
                (sequence, "close aborted after dispatch: CLOSE_MODAL already queued, unload not observed".to_owned()),
            MenuCloseState::Watching { sequence, target, polls } => {
                if crate::adapter_action::menu_close_done(client, target) {
                    (sequence, "accepted: CLOSE_MODAL queued client-side".to_owned())
                } else if polls + 1 >= CLOSE_RESULT_POLLS {
                    (sequence, "close menu op dispatched [95] but did not unload the modal".to_owned())
                } else {
                    self.menu_close = Some(MenuCloseState::Watching { sequence, target, polls: polls + 1 });
                    return Ok(true);
                }
            }
            // Every poll counts toward the bound; only ready frames re-resolve.
            MenuCloseState::Waiting { sequence, polls, error } if !ready => {
                if polls + 1 < CLOSE_ARM_POLLS {
                    self.menu_close = Some(MenuCloseState::Waiting { sequence, polls: polls + 1, error });
                    return Ok(true);
                }
                (sequence, format!("close script unavailable: {error}"))
            }
            MenuCloseState::Waiting { sequence, polls, .. } => match crate::adapter_action::menu_close_target(client) {
                Ok(MenuClose::Target(target)) => {
                    self.start_menu_close(client, live, ready, sequence, target)?;
                    return Ok(true);
                }
                Ok(MenuClose::Pending(error)) if polls + 1 < CLOSE_ARM_POLLS => {
                    self.menu_close = Some(MenuCloseState::Waiting { sequence, polls: polls + 1, error });
                    return Ok(true);
                }
                Ok(menu) => {
                    self.write_close_diagnostics(client, live, sequence, &menu);
                    (sequence, match menu {
                        MenuClose::Pending(error) => format!("close script unavailable: {error}"),
                        MenuClose::Refused(reason) => format!("IF3 menu close refused: {reason}"),
                        _ => "IF3 menu close target disappeared while its script loaded".to_owned(),
                    })
                }
                Err(error) => (sequence, format!("client action failed: {error}")),
            },
        };
        self.sequence = sequence;
        self.receipt(client, ready, &status)?;
        Ok(true)
    }
    /// Advances an in-flight `chat` or `minimap`; its receipt and mode file follow
    /// the bounded outcome. Deferred input telemetry is applied here (see
    /// native_input).
    fn poll_native_input(&mut self, client: &mut ClientState, live: &mut crate::live_runtime::Connection, ready: bool) -> Result<()> {
        if let Some(chat) = self.chat.as_mut() {
            for (key, down) in chat.take_telemetry() { live.keyboard_key(key, down); }
            let Some(status) = chat.poll(client) else { return Ok(()); };
            for (key, down) in chat.take_telemetry() { live.keyboard_key(key, down); }
            let chat = self.chat.take().unwrap();
            let gate = chat.gate.as_ref().map_or("null".to_owned(), |g| g.json());
            self.write_diagnostic("chat-mode", format!("{{\"sequence\":{},\"mode\":\"chatdefault_onkey\",\"text\":{},\"typed\":{},\"gate\":{gate},\"packets\":{:?},\"status\":{}}}",
                chat.sequence, quoted(chat.text()), chat.typed(), chat.packets, quoted(&status)));
            self.sequence = chat.sequence;
            return self.receipt(client, ready, &status);
        }
        if let Some(minimap) = self.minimap.as_mut() {
            let pressed = minimap.press_frame();
            let status = minimap.poll(client, Some(&mut *live));
            // Mouse telemetry for a press that passed the live check, in this frame.
            if pressed && minimap.telemetry_sent() { live.mouse_pointer(minimap.target.point); live.mouse_click(1, minimap.target.point); }
            let Some(status) = status else { return Ok(()); };
            let minimap = self.minimap.take().unwrap();
            let packets = minimap.packets.as_ref().map_or("null".to_owned(), |p| format!("{p:?}"));
            self.write_diagnostic("minimap-mode", format!("{{\"sequence\":{},\"mode\":\"minimap-click\",{},\"packets\":{packets},\"telemetry\":{},\"status\":{}}}",
                minimap.sequence, minimap.target.json(), minimap.telemetry_sent(), quoted(&status)));
            self.sequence = minimap.sequence;
            return self.receipt(client, ready, &status);
        }
        Ok(())
    }
    /// Small escaped diagnostics beside, not inside, the existing receipts.
    fn write_refusal(&self, sequence: u64, seen: &[crate::adapter_action::Seen]) {
        self.write_diagnostic("refusal", format!("{{\"sequence\":{sequence},\"probes\":[{}]}}", probes_json(seen)));
    }
    /// Written only after an admitted dispatch: the candidate index (0 is the
    /// interior pixel), its point, and the bounded refused probes before it.
    fn write_admission(&self, sequence: u64, admission: crate::adapter_action::Admission, seen: &[crate::adapter_action::Seen]) {
        self.write_diagnostic("admission", format!("{{\"sequence\":{sequence},\"index\":{},\"point\":[{},{}],\"refused_before\":[{}]}}",
            admission.index, admission.point[0], admission.point[1], probes_json(seen)));
    }
    fn write_diagnostic(&self, name: &str, text: String) {
        let path = self.directory.join(format!("{name}.tmp"));
        // Diagnostics are optional; a sharing violation must not fail the command.
        let _ = std::fs::write(&path, text)
            .and_then(|_| std::fs::rename(&path, self.directory.join(format!("{name}.json"))));
    }
    pub fn poll(&mut self, client: &mut ClientState, live: &mut crate::live_runtime::Connection, ready: bool) -> Result<()> {
        // Receipts written by this poll report the counters as they are now: before this
        // frame's paint, never the custody of a captured raster (#175).
        self.custody_state = Some(live.draw_custody.state_json());
        // A sharing violation retries only observation output, never dispatch.
        if self.receipt_pending.is_some() {
            self.flush_receipt(client, ready)?;
            if self.receipt_pending.is_some() {
                return Ok(());
            }
        }
        if self.poll_menu_close(client, live, ready)? {
            return Ok(());
        }
        if let Some(close) = self.close.as_mut() {
            if let Some(status) = close.poll(client) {
                self.sequence = close.sequence;
                self.close = None;
                let status = match self.close_error.take() {
                    Some(error) => format!("{status}: {error}"),
                    None => status.to_owned(),
                };
                return self.receipt(client, ready, &status);
            }
            return Ok(());
        }
        if self.chat.is_some() || self.minimap.is_some() {
            return self.poll_native_input(client, live, ready);
        }
        // A controller atomically replaces command.txt with: SEQUENCE VERB ARGS.
        // Only one outstanding command is allowed; receipt.txt acknowledges it.
        let text = match std::fs::File::open(self.directory.join("command.txt")).and_then(read_command) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) if transient_io(&error) => return Ok(()),
            Err(error) => return Err(error.into()),
        };
        let words: Vec<_> = text.split_whitespace().collect();
        let Some(sequence) = words.first().and_then(|s| s.parse::<u64>().ok()) else {
            return Ok(());
        };
        if !sequence_available(sequence,self.sequence,self.receipt_pending.is_some()) {
            return Ok(());
        }
        // Resolve dynamic inventory widgets only after the same UI/script
        // barrier that blocks normal native input has cleared.
        if !ready
            && !matches!(
                words.get(1),
                Some(&"observe" | &"capture" | &"inspect" | &"stop")
            )
        {
            return Ok(());
        }
        self.inspect = words.get(1) == Some(&"inspect");
        if words.get(1) == Some(&"chat") {
            // The raw tail keeps the text exactly; `split_whitespace` would not.
            let tail = text.trim_end_matches(['\r', '\n']).strip_prefix(&format!("{} chat ", words[0]));
            let refusal = match tail.map(|tail| native_input::ChatDriver::new(sequence, tail)) {
                Some(Ok(driver)) => { self.chat = Some(driver); return Ok(()); }
                Some(Err(reason)) => format!("invalid arguments: {reason}"),
                None => "invalid arguments".to_owned(),
            };
            self.sequence = sequence;
            return self.receipt(client, ready, &refusal);
        }
        if words.get(1) == Some(&"minimap") {
            let Some((x, z)) = words.get(2).and_then(|s| s.parse::<u16>().ok()).zip(words.get(3).and_then(|s| s.parse::<u16>().ok())).filter(|_| words.len() == 4) else {
                self.sequence = sequence;
                return self.receipt(client, ready, "invalid arguments");
            };
            let refusal = match native_input::minimap_target(client, live, [x, z]) {
                Ok(Ok(target)) => {
                    self.minimap = Some(native_input::MinimapDriver::arm(sequence, target));
                    return Ok(());
                }
                Ok(Err(reason)) => format!("minimap refused: {reason}"),
                Err(error) => format!("client action failed: {error}"),
            };
            self.sequence = sequence;
            return self.receipt(client, ready, &refusal);
        }
        let number = |i: usize| words.get(i).and_then(|s| s.parse::<u16>().ok());
        let item_widget = |slot: u16| {
            client
                .interfaces
                .runtime
                .iter()
                .filter(|(_, runtime)| {
                    runtime.logical_id >> 16 == 149
                        && runtime.child_index == slot as i32
                        && runtime.item >= 0
                })
                .map(|(&id, _)| id)
                .min()
        };
        if words.get(1) == Some(&"camera") && words.len() == 4 {
            if let Some((pitch, yaw)) = number(2)
                .zip(number(3))
                .filter(|(pitch, yaw)| (1024..=3064).contains(pitch) && *yaw < 16384)
            {
                client.camera.pitch = pitch as i32;
                client.camera.yaw = yaw as i32;
                self.sequence = sequence;
                return self.receipt(client, ready, "camera updated");
            }
        }
        #[cfg(feature="custom-assets")]
        if words.get(1)==Some(&"environment-effects") && words.len()==3 {
            self.sequence=sequence;
            let Ok(mode)=words[2].parse::<crate::environment_motion::Mode>() else {
                return self.receipt(client,ready,"invalid command");
            };
            let admitted=live.world.as_mut().is_some_and(|(_,mesh)|mesh.set_environment_motion_mode(mode));
            return self.receipt(client,ready,if admitted {"visual effects updated"} else {"authored motion unavailable"});
        }
        let action =
            match words.get(1).copied() {
                Some("cancel") if words.len() == 2 => Some(Action::Cancel),
                Some("examine-npc") if words.len() == 3 => number(2).map(Action::ExamineNpc),
                Some("use-item") if words.len() == 3 => number(2).and_then(item_widget).map(Action::CastInterface),
                Some("item") if words.len() == 4 => number(2)
                    .and_then(item_widget)
                    .zip(number(3))
                    .filter(|(_, op)| (1..=32).contains(op))
                    .map(|(id, op)| Action::Interface {
                        id,
                        operation: op as u8,
                        sub: None,
                    }),
                Some("select-item") if words.len() == 3 => {
                    number(2).and_then(item_widget).map(Action::SelectSpell)
                }
                Some("use-object") if words.len() == 5 => number(2)
                    .zip(number(3))
                    .zip(number(4))
                    .map(|((id, x), y)| Action::CastWorld {
                        kind: 0,
                        id,
                        tile: [x, y],
                    }),
                Some("ground") if words.len() == 6 => number(2)
                    .zip(number(3))
                    .zip(number(4))
                    .zip(number(5))
                    .filter(|(_, op)| (1..=5).contains(op))
                    .map(|(((id, x), y), op)| Action::Ground {
                        id,
                        tile: [x, y],
                        operation: op as u8,
                        sub: 0,
                    }),
                Some("walk") if words.len() == 4 => {
                    number(2).zip(number(3)).map(|(x, y)| Action::Walk([x, y]))
                }
                Some("close") if words.len() == 2 => Some(Action::Close),
                Some("interface") if words.len() == 4 => words[2]
                    .parse::<u32>()
                    .ok()
                    .zip(number(3))
                    .filter(|(_, op)| (1..=32).contains(op))
                    .map(|(id, op)| Action::Interface {
                        id,
                        operation: op as u8,
                        sub: None,
                    }),
                Some("continue") if words.len() == 3 => {
                    words[2].parse::<u32>().ok().map(Action::Continue)
                }
                Some("npc") if words.len() == 4 => number(2)
                    .zip(number(3))
                    .filter(|(_, op)| (1..=5).contains(op))
                    .map(|(index, op)| Action::Npc {
                        index,
                        operation: op as u8,
                        sub: 0,
                    }),
                Some("object") if words.len() == 6 => number(2)
                    .zip(number(3))
                    .zip(number(4))
                    .zip(number(5))
                    .filter(|(_, op)| (1..=5).contains(op))
                    .map(|(((id, x), y), op)| Action::Object {
                        id,
                        tile: [x, y],
                        operation: op as u8,
                        sub: 0,
                    }),
                Some("observe" | "capture" | "inspect" | "stop") if words.len() == 2 => None,
                _ => {
                    self.sequence = sequence;
                    return self.receipt(client, ready, "invalid command");
                }
            };
        if action.is_none()
            && !matches!(
                words.get(1),
                Some(&"observe" | &"capture" | &"inspect" | &"stop")
            )
        {
            self.sequence = sequence;
            return self.receipt(client, ready, "invalid arguments or target not available");
        }
        if action.is_some() && !ready {
            return Ok(());
        }
        if words.get(1) == Some(&"item") {
            if let Some(Action::Interface { id, operation, .. }) = action {
                let visible = client
                    .interfaces
                    .widgets
                    .get(&id)
                    .and_then(|widget| widget.actions.get(operation as usize - 1))
                    .is_some_and(|label| !label.trim().is_empty());
                if !visible {
                    self.sequence = sequence;
                    return self.receipt(
                        client,
                        ready,
                        "inventory operation has no displayed action",
                    );
                }
            }
        }
        if let Some(action) = action {
            use crate::adapter_action::Dispatch;
            let status=match crate::adapter_action::dispatch_visible(client,live,action) {
                Ok(Dispatch::Admitted(admission,seen))=>{
                    self.write_admission(sequence,admission,&seen);
                    None
                }
                Ok(Dispatch::Refused(seen))=>{
                    // IF1 close buttons are menu-admitted above. Steelborder IF3 close
                    // controls have no menu op: arm a consume-time delivery instead.
                    // An exact native-menu Close op (steelborder, onop 29) is chosen
                    // through the real menu first; op-less 1400 controls use the driver.
                    if action==Action::Close {
                        use crate::adapter_action::MenuClose;
                        match crate::adapter_action::menu_close_target(client) {
                            Ok(MenuClose::Target(target))=>return self.start_menu_close(client,live,ready,sequence,target),
                            Ok(MenuClose::Pending(error))=>{
                                self.menu_close=Some(MenuCloseState::Waiting{sequence,polls:0,error});
                                return Ok(());
                            }
                            Ok(menu@MenuClose::Refused(_))=>{
                                self.write_close_diagnostics(client,live,sequence,&menu);
                                let MenuClose::Refused(reason)=menu else {unreachable!()};
                                self.sequence=sequence;
                                return self.receipt(client,ready,&format!("IF3 menu close refused: {reason}"));
                            }
                            Ok(MenuClose::Absent)=>{}
                            Err(error)=>{
                                self.sequence=sequence;
                                return self.receipt(client,ready,&format!("client action failed: {error}"));
                            }
                        }
                        let pointer=client.input.pointer;
                        match crate::adapter_action::close_target(client,pointer) {
                            Ok(Some(target))=>{
                                self.write_close_mode(sequence,format!("\"mode\":\"press\",\"widget\":{},\"instance\":{}",target.widget,target.instance));
                                self.close=Some(crate::adapter_action::CloseDriver::arm(sequence,target));
                                self.close_error=None;
                                return Ok(());
                            }
                            Ok(None)=>{
                                self.write_close_diagnostics(client,live,sequence,&MenuClose::Absent);
                                self.write_refusal(sequence,&seen);
                                self.sequence=sequence;
                                return self.receipt(client,ready,"no IF3 close control admitted (see close-diagnostics.json)");
                            }
                            Err(error)=>{
                                self.sequence=sequence;
                                return self.receipt(client,ready,&format!("client action failed: {error}"));
                            }
                        }
                    }
                    self.write_refusal(sequence,&seen);
                    Some("action is not in the visible native menu".to_owned())
                }
                Err(error)=>Some(format!("client action failed: {error}")),
            };
            if let Some(status)=status {
                self.sequence=sequence;
                return self.receipt(client,ready,&status);
            }
        }
        self.sequence = sequence;
        if words[1] == "capture" {
            self.capture = Some(sequence);
        }
        self.stop = words[1] == "stop";
        self.receipt(client, ready, "accepted")
    }
    fn receipt(&mut self, client: &ClientState, ready: bool, status: &str) -> Result<()> {
        self.receipt_pending = Some(status.into());
        self.flush_receipt(client, ready)
    }
    fn flush_receipt(&mut self, client: &ClientState, ready: bool) -> Result<()> {
        let Some(status) = self.receipt_pending.as_deref() else {
            return Ok(());
        };
        match self.write_receipt(client, ready, status) {
            Ok(()) => {
                self.receipt_pending = None;
                Ok(())
            }
            Err(error) if transient_io(&error) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
    fn write_receipt(&self, client: &ClientState, ready: bool, status: &str) -> io::Result<()> {
        let npcs: Vec<_> = client
            .npc_state
            .npcs
            .iter()
            .map(|(&index, n)| (index, n.id, n.position))
            .collect();
        let messages: Vec<_> = client
            .game_messages
            .iter()
            .rev()
            .take(12)
            .map(|m| &m.text)
            .collect();
        let widgets: Vec<_> = if self.inspect {
            client
                .interfaces
                .widgets
                .iter()
                .filter(|(_, w)| !w.actions.is_empty() || !w.button_text.is_empty())
                .map(|(&id, w)| (id, &w.text, &w.actions, &w.button_text))
                .collect()
        } else {
            Vec::new()
        };
        let text = format!("sequence={}\nstatus={status}\nready={ready}\nposition={:?}\nserver_tick={}\nqueued_packets={}\nmap_flag={:?}\ninterfaces={:?}\nnpcs={npcs:?}\nmessages={messages:?}\n", self.sequence, client.data.position, client.server_tick, client.outbound.len(), client.map_flag, client.interfaces.children);
        std::fs::write(self.directory.join("receipt.tmp"), text)?;
        std::fs::rename(
            self.directory.join("receipt.tmp"),
            self.directory.join("receipt.txt"),
        )?;
        if self.inspect {
            // Read-only diagnostics for actual visible bank/inventory controls.
            // Commands still use the ordinary interface/menu policy unchanged.
            let layout = client.interfaces.layout(client.canvas)
                .map_err(|error| io::Error::other(error.to_string()))?;
            let mut ui_items = Vec::new();
            let mut geometry = Vec::new();
            // Open modal groups too, so unload checks have a real baseline.
            let modal = crate::adapter_action::open_modal_groups(client);
            for (&id, runtime) in &client.interfaces.runtime {
                if !matches!(runtime.logical_id >> 16, 149 | 548 | 161 | 164)
                    && !modal.contains(&((runtime.logical_id >> 16) as u16)) { continue; }
                let Some(widget) = client.interfaces.widgets.get(&id) else { continue; };
                geometry.push((id, runtime.logical_id, runtime.child_index, widget.parent,
                    widget.kind, widget.hidden, widget.raw_position, widget.raw_size,
                    runtime.aligned.get(), runtime.item, runtime.count, layout.widgets.get(&id)));
            }
            geometry.sort_by_key(|row|row.0);
            std::fs::write(self.directory.join("ui-geometry.txt"), format!("{geometry:#?}"))?;
            for (&id, runtime) in &client.interfaces.runtime {
                if runtime.item < 0 || runtime.count <= 0 || runtime.child_index < 0 {
                    continue;
                }
                let Some(placement) = layout.widgets.get(&id) else { continue; };
                if placement.hidden || placement.clip[0] >= placement.clip[2]
                    || placement.clip[1] >= placement.clip[3] { continue; }
                let Some(widget) = client.interfaces.widgets.get(&id) else { continue; };
                let actions = widget.actions.iter().map(|s| quoted(s)).collect::<Vec<_>>().join(",");
                ui_items.push(format!("{{\"id\":{id},\"logical_id\":{},\"slot\":{},\"item\":{},\"count\":{},\"actions\":[{actions}]}}",
                    runtime.logical_id, runtime.child_index, runtime.item, runtime.count));
            }
            std::fs::write(self.directory.join("ui-items.ndjson"), ui_items.join("\n"))?;
            let mut containers = Vec::new();
            for (&inventory, items) in &client.inventories.containers {
                for (slot, item) in items.iter().enumerate() {
                    if item.id >= 0 && item.count > 0 {
                        containers.push(format!("{{\"inventory\":{inventory},\"slot\":{slot},\"item\":{},\"count\":{}}}", item.id, item.count));
                    }
                }
            }
            std::fs::write(self.directory.join("containers.ndjson"), containers.join("\n"))?;
            std::fs::write(self.directory.join("widgets.txt"), format!("{widgets:#?}"))?;
            std::fs::write(
                self.directory.join("inventory.txt"),
                format!("{:?}", client.inventories.containers),
            )?;
            std::fs::write(
                self.directory.join("experience.txt"),
                format!("{:?}", client.data.experience),
            )?;
            let graphics: Vec<_> = client
                .world_spots
                .iter()
                .map(|effect| (effect.position, effect.animation.id, effect.animation.start))
                .collect();
            std::fs::write(self.directory.join("graphics.txt"), format!("{graphics:?}"))?;
            std::fs::write(
                self.directory.join("levels.txt"),
                format!(
                    "base={:?}\ncurrent={:?}",
                    client.data.base, client.data.boosted
                ),
            )?;
        }
        let items=client.inventories.containers.get(&93).into_iter().flatten().enumerate().map(|(slot,item)| {
            let widget=client.interfaces.runtime.iter().filter(|(_,r)|r.logical_id>>16==149 && r.child_index==slot as i32 && r.item==item.id)
                .map(|(&id,_)|id).min();
            let operations=widget.and_then(|id|client.interfaces.widgets.get(&id)).map_or(String::new(),|w|w.actions.iter().map(|s|quoted(s)).collect::<Vec<_>>().join(","));
            format!("{{\"slot\":{slot},\"id\":{},\"count\":{},\"widget\":{},\"operations\":[{operations}]}}",item.id,item.count,widget.map_or("null".into(),|id|id.to_string()))
        }).collect::<Vec<_>>().join(",");
        let npcs = client
            .npc_state
            .npcs
            .iter()
            .map(|(&index, n)| {
                format!(
                    "{{\"index\":{index},\"id\":{},\"x\":{},\"z\":{}}}",
                    n.id, n.position[0], n.position[1]
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let graphics = client
            .world_spots
            .iter()
            .map(|g| {
                format!(
                    "{{\"id\":{},\"x\":{},\"z\":{},\"start\":{}}}",
                    g.animation.id, g.position[1], g.position[2], g.animation.start
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        // Normal chat UIDs distinguish repeated server messages without
        // mutating gameplay or guessing acknowledgement from a text cache.
        // Public chat (1/2) is listed even when no game message shares its kind.
        let kinds: std::collections::BTreeSet<_> = client.game_messages.iter().map(|m|m.kind as i32).chain([1, 2]).collect();
        let mut events=Vec::new();
        for kind in kinds {for line in 0..32 {if let Some(message)=client.chat.line(kind,line) {events.push(message);}}}
        events.sort_unstable_by_key(|message|std::cmp::Reverse(message.uid));
        events.truncate(32);
        let message_events=events.into_iter().map(|m|format!("{{\"uid\":{},\"cycle\":{},\"kind\":{},\"name\":{},\"text\":{}}}",m.uid,m.cycle,m.kind,quoted(&m.name),quoted(&m.text))).collect::<Vec<_>>().join(",");
        let messages = messages
            .iter()
            .map(|s| quoted(s))
            .collect::<Vec<_>>()
            .join(",");
        let equipment = client
            .inventories
            .containers
            .get(&94)
            .into_iter()
            .flatten()
            .enumerate()
            .filter(|(_, item)| item.id >= 0)
            .map(|(slot, item)| {
                format!(
                    "{{\"slot\":{slot},\"id\":{},\"count\":{}}}",
                    item.id, item.count
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let ground = client
            .zone
            .piles
            .iter()
            .flat_map(|(position, pile)| {
                pile.iter().map(move |item| {
                    format!(
                        "{{\"plane\":{},\"x\":{},\"z\":{},\"id\":{},\"count\":{}}}",
                        position[0], position[1], position[2], item.id, item.quantity
                    )
                })
            })
            .collect::<Vec<_>>()
            .join(",");
        let modals = client
            .interfaces
            .children
            .iter()
            .filter(|(_, sub)| sub.kind == 0)
            .map(|(_, sub)| sub.group.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let journal = client
            .interfaces
            .widgets
            .get(&((259 << 16) | 1))
            .map_or_else(|| quoted(""), |widget| quoted(&widget.text));
        let selection_active=client.selected_spell.is_some();
        let cross_frame=client.click_cross.frame().map_or("null".into(),|frame|frame.to_string());
        let cross_position=client.click_cross.position;
        let state=format!("{{\"sequence\":{},\"status\":{},\"ready\":{ready},\"selection_active\":{selection_active},\"cross_frame\":{cross_frame},\"cross_position\":{cross_position:?},\"position\":{:?},\"tick\":{},\"cycle\":{},\"hp\":{},\"base\":{:?},\"xp\":{:?},\"items\":[{items}],\"equipment\":[{equipment}],\"ground\":[{ground}],\"modals\":[{modals}],\"journal\":{journal},\"npcs\":[{npcs}],\"graphics\":[{graphics}],\"messages\":[{messages}],\"message_events\":[{message_events}],\"draw_custody\":{}}}",self.sequence,quoted(status),client.data.position,client.server_tick,client.actor_cycle,client.data.boosted[3],client.data.base,client.data.experience,self.custody_state.as_deref().unwrap_or("null"));
        std::fs::write(self.directory.join("state.tmp"), state)?;
        std::fs::rename(
            self.directory.join("state.tmp"),
            self.directory.join("state.json"),
        )?;
        Ok(())
    }
    /// Read-only scene provenance for environmental audits; never changes scene
    /// objects, clipping, cache data or authoritative gameplay state.
    pub fn inspect_scene(&mut self, connection: &crate::live_runtime::Connection) -> Result<()> {
        if !self.inspect || self.scene_reported == self.sequence {return Ok(());}
        if let Some((scene,_))=&connection.world {
            // Null renderables legitimately retain clipping without a scene
            // entry. Export the actual normal collision map for that audit.
            let mut clipping=String::from("plane,x,z,flags\n");
            for (plane,map) in scene.collision.iter().enumerate() {
                for x in 0..scene.width {for z in 0..scene.height {
                    if let Some(flags)=map.flag_at(x as i32,z as i32) {
                        clipping.push_str(&format!("{plane},{},{},{flags}\n",scene.base_x+x as i32,scene.base_y+z as i32));
                    }
                }}
            }
            std::fs::write(self.directory.join("scene-collision.tmp"),clipping)?;
            std::fs::rename(self.directory.join("scene-collision.tmp"),self.directory.join("scene-collision.csv"))?;
            let mut text=String::from("id,plane,x,z,shape,rotation,runtime,rejected\n");
            for o in &scene.objects {
                text.push_str(&format!("{},{},{},{},{},{},{},{}\n",o.id,o.plane,scene.base_x+o.x as i32,scene.base_y+o.y as i32,o.shape,o.rotation,o.runtime,o.rejected));
            }
            let path=self.directory.join("scene-objects.tmp");
            match std::fs::write(&path,text).and_then(|_|std::fs::rename(&path,self.directory.join("scene-objects.csv"))) {
                Ok(())=>self.scene_reported=self.sequence,
                Err(error) if transient_io(&error)=>{},
                Err(error)=>return Err(error.into()),
            }
        }
        Ok(())
    }
    pub fn frame(&mut self, raster: &Raster,client:&ClientState,live:Option<&crate::live_runtime::Connection>) -> Result<()> {
        // #175: first, on every frame, so a later frame can never reuse this paint.
        let custody=live.map(|c|&c.draw_custody);
        let link=self.frames.observe(custody);
        // After pending_clicks: what the real minimap click queued this frame.
        if let Some(minimap) = self.minimap.as_mut() { minimap.observe_frame(client); }
        if let Some(sequence) = self.capture {
            let draw=crate::draw_custody::frame_json(sequence,self.frames.frames,link,custody);
            let sidecars=capture_sidecars(client,live,draw)?;
            let pixels = raster.png()?;
            match publish_capture(&self.directory,sequence,&pixels,&sidecars) {
                Ok(()) => self.capture = None,
                Err(error) if transient_io(&error) => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }
}

fn probes_json(seen: &[crate::adapter_action::Seen]) -> String {
    seen.iter().map(|s| format!("{{\"point\":[{},{}],\"options\":[{}]}}", s.point[0], s.point[1],
        s.options.iter().map(|o| quoted(o)).collect::<Vec<_>>().join(","))).collect::<Vec<_>>().join(",")
}
fn read_command(reader: impl Read) -> io::Result<String> {
    let mut text=String::new();
    reader.take(4097).read_to_string(&mut text)?;
    if text.len()>4096 {return Err(io::Error::new(io::ErrorKind::InvalidData,"adapter command exceeds4096bytes"));}
    Ok(text)
}
fn sequence_available(next:u64,current:u64,pending_receipt:bool)->bool {
    !pending_receipt && next>current
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn stop_waits_for_pending_receipt_and_capture() {
        let mut a=Adapter {headless:false,directory:PathBuf::new(),sequence:1,capture:None,
            receipt_pending:Some("accepted".into()),inspect:false,scene_reported:0,close:None,close_error:None,menu_close:None,chat:None,minimap:None,
            frames:Default::default(),custody_state:None,stop:true};
        assert!(!a.stop_ready());a.receipt_pending=None;a.capture=Some(1);
        assert!(!a.stop_ready());a.capture=None;assert!(a.stop_ready());
    }
    #[test] fn admission_and_refusal_diagnostics_are_separate_sequence_tagged_files() {
        use crate::adapter_action::{Admission, Seen};
        let directory=std::env::temp_dir().join(format!("adapter-diagnostics-{}-{}",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let a=Adapter::new(directory.clone()).unwrap();
        let seen=vec![Seen{point:[571,195],options:vec!["Cancel".into(),"Skills \"tab\"".into()]}];
        a.write_admission(41,Admission{index:1,point:[576,165]},&seen);
        let text=std::fs::read_to_string(directory.join("admission.json")).unwrap();
        assert_eq!(text,r#"{"sequence":41,"index":1,"point":[576,165],"refused_before":[{"point":[571,195],"options":["Cancel","Skills \"tab\""]}]}"#);
        assert!(!directory.join("refusal.json").exists(),"admission never writes a refusal record");
        a.write_refusal(42,&seen);
        let refusal=std::fs::read_to_string(directory.join("refusal.json")).unwrap();
        assert_eq!(refusal,r#"{"sequence":42,"probes":[{"point":[571,195],"options":["Cancel","Skills \"tab\""]}]}"#);
        // A refusal leaves the earlier, differently tagged admission record untouched.
        assert_eq!(std::fs::read_to_string(directory.join("admission.json")).unwrap(),text);
        let _=std::fs::remove_dir_all(&directory);
    }
    #[test] fn command_input_is_bounded_and_utf8() {
        assert!(read_command(&vec![b'x';4096][..]).is_ok());
        assert!(read_command(&vec![b'x';4097][..]).is_err());
        assert!(read_command(&[255][..]).is_err());
    }
    #[test] fn receipt_retry_blocks_new_and_duplicate_dispatch() {
        let mut count=0;let mut current=0;
        for (next,pending) in [(1,false),(1,true),(2,true),(1,false),(0,false),(2,false)] {
            if sequence_available(next,current,pending) {count+=1;current=next;}
        }
        assert_eq!((count,current),(2,2));
    }
}

// The logical frame is published last. A controller waiting for it cannot
// mistake a transient sidecar failure for a completed capture bundle.
fn publish_capture(directory:&std::path::Path,sequence:u64,pixels:&[u8],sidecars:&[(&str,Vec<u8>)])->io::Result<()> {
    let write=|suffix:&str,bytes:&[u8]| {
        let path=directory.join(format!("frame-{sequence}{suffix}"));
        let pending=directory.join(format!("frame-{sequence}{suffix}.tmp"));
        std::fs::write(&pending,bytes).and_then(|_|std::fs::rename(&pending,&path))
    };
    for (suffix,bytes) in sidecars {write(suffix,bytes)?;}
    write(".png",pixels)
}
fn camera_diagnostic(last:Option<(crate::viewport_steering::Ray,[i32;3])>,requested:[i32;2],zoom:[i16;2])->String {
    let last=last.map_or("null".into(),|(ray,root)|format!(
        "{{\"eye\":{:?},\"root_eye\":{root:?},\"angles\":{:?},\"focus\":{:?},\"size\":{:?},\"focal\":{},\"far\":{}}}",
        ray.eye,ray.angles,ray.focus,ray.size,ray.focal,ray.far));
    format!("{{\"scope\":\"last completed world draw; may precede this logical UI frame\",\"last_world_draw\":{last},\"requested_pitch_yaw\":{requested:?},\"requested_viewport_zoom\":{zoom:?}}}\n")
}
fn capture_sidecars(client:&ClientState,live:Option<&crate::live_runtime::Connection>,draw:String)->Result<Vec<(&'static str,Vec<u8>)>> {
    // #175: custody of exactly this raster; published with the sidecars, before the PNG.
    let mut files=vec![("-draw.json",draw.into_bytes())];
    let camera=camera_diagnostic(live.and_then(|c|c.capture_camera()),[client.camera.pitch,client.camera.yaw],client.viewport.zoom);
    files.push(("-camera.json",camera.into_bytes()));
    let children=child_worlds_diagnostic(client.world_entities.get(&0),client.actor_cycle,|id,generation| {
        live.and_then(|c|c.instance_scene(id,generation)).map(|scene|
            [scene.base_x,scene.base_y,scene.width as i32,scene.height as i32])
    });
    files.push(("-child-worlds.json",children.into_bytes()));
    let map=live.and_then(|c|c.world.as_ref().and_then(|(scene,_)|c.minimap_view(client).map(|view|([scene.base_x,scene.base_y],view))));
    if let Some((base,view))=map {
        let raster=Raster::from_pixels(view.terrain.size[0],view.terrain.size[1],view.terrain.pixels.clone())?;
        let nonblack=view.terrain.pixels.iter().filter(|p|**p&0xffffff!=0).count();
        let metadata=format!("{{\"available\":true,\"scope\":\"raw current minimap terrain; not the masked displayed minimap\",\"scene_base\":{base:?},\"local_fine\":{:?},\"source\":{:?},\"minimap_state\":{},\"disabled\":{},\"size\":{:?},\"nonblack\":{nonblack},\"npc_attack_setting\":{},\"attack_varp1306\":{},\"position\":{:?}}}\n",
            client.local_minimap_fine(),view.source,client.minimap_state,view.disabled,view.terrain.size,
            client.menu_settings.npc_attack,client.scripts.variables.player(1306),client.data.position);
        files.push(("-minimap.json",metadata.into_bytes()));files.push(("-minimap.png",raster.png()?));
    } else {files.push(("-minimap.json",b"{\"available\":false}\n".to_vec()));}
    Ok(files)
}

// Snapshot actual root-child state and generation-specific resources. Admission
// and loaded resources do not by themselves prove rendered pixels or actor draws.
fn child_worlds_diagnostic(world:Option<&crate::world_entity::State>,cycle:i32,
    mut scene:impl FnMut(u16,u64)->Option<[i32;4]>)->String {
    let visible=world.map_or(&[][..],|world|world.visible.as_slice());
    let mut entities=world.into_iter().flat_map(|world|world.entities.iter()).collect::<Vec<_>>();
    entities.sort_unstable_by_key(|(id,_)|**id);
    let children=entities.into_iter().map(|(&id,entity)| {
        let resource=scene(id,entity.spawn_generation).map_or("null".into(),|value|format!("{value:?}"));
        let npcs=entity.npcs.indices.iter().map(|index| {
            entity.npcs.npcs.get(index).map_or_else(||format!("{{\"index\":{index},\"present\":false}}"),|npc|
                format!("{{\"index\":{index},\"present\":true,\"id\":{},\"tile_position\":{:?},\"fine_position\":{:?}}}",npc.id,npc.position,npc.path.position))
        }).collect::<Vec<_>>().join(",");
        format!("{{\"id\":{id},\"spawn_generation\":{},\"rebuild_generation\":{},\"registered\":{},\"plane\":{},\"dimensions\":{:?},\"pose_xyz_angle\":[{},{},{},{}],\"scene_base_size\":{resource},\"npcs\":[{npcs}]}}",
            entity.spawn_generation,entity.rebuild_generation,entity.registered,entity.plane,entity.dimensions,
            entity.pose.x,entity.pose.y,entity.pose.z,entity.pose.angle)
    }).collect::<Vec<_>>().join(",");
    format!("{{\"scope\":\"current root-child admission and resources; not proof of rendered pixels\",\"actor_cycle\":{cycle},\"root_state_present\":{},\"visible_ids\":{visible:?},\"children\":[{children}]}}\n",world.is_some())
}

#[cfg(test)] mod capture_tests {
    use super::*;
    #[test]
    fn child_capture_distinguishes_empty_admission_and_generation_resources() {
        use crate::world_entity::{Entity,State,Pose};
        let empty=child_worlds_diagnostic(None,7,|_,_|panic!("no child resource lookup"));
        assert!(empty.contains("\"root_state_present\":false"));
        assert!(empty.contains("\"children\":[]"));
        let mut world=State::default();world.visible=vec![9,2];
        let mut child=Entity {spawn_generation:42,rebuild_generation:3,dimensions:[8,16],
            pose:Pose{x:128,y:-64,z:256,angle:512},..Entity::default()};
        child.npcs.indices.push(5); // Listed without a decoded actor must not claim one.
        world.entities.insert(9,child);
        world.entities.insert(2,Entity{spawn_generation:99,registered:false,..Entity::default()});
        let mut lookups=Vec::new();
        let text=child_worlds_diagnostic(Some(&world),8,|id,generation| {
            lookups.push((id,generation));
            if (id,generation)==(9,42) {Some([100,200,8,16])} else {None}
        });
        assert_eq!(lookups,vec![(2,99),(9,42)]);
        assert!(text.contains("\"visible_ids\":[9, 2]"));
        assert!(text.contains("\"scene_base_size\":null"));
        assert!(text.contains("\"scene_base_size\":[100, 200, 8, 16]"));
        assert!(text.contains("\"pose_xyz_angle\":[128,-64,256,512]"));
        assert!(text.contains("\"index\":5,\"present\":false"));
        assert_eq!(world.entities[&9].spawn_generation,42);
        assert_eq!(world.visible,vec![9,2]);
    }
    #[test]
    fn child_capture_keeps_decoded_tile_and_fine_positions_distinct() {
        use crate::world_entity::{Entity,State};
        let definitions=[(7,crate::npc::Definition::decode(7,&[0]).unwrap())].into_iter().collect();
        let mut child=Entity::default();
        // Small NPC spawn: index 30000, delta (2,0), definition 7, jump.
        child.npcs.decode(&[0,117,48,2,0,64,7],false,[8,8],&definitions).unwrap();
        let mut world=State::default();world.entities.insert(1,child);
        let text=child_worlds_diagnostic(Some(&world),9,|_,_|None);
        assert!(text.contains("\"index\":30000,\"present\":true,\"id\":7"));
        assert!(text.contains("\"tile_position\":[10, 8]"));
        assert!(text.contains("\"fine_position\":[1344, 1088]"));
    }
    #[test]
    fn capture_bundle_does_not_publish_frame_when_sidecar_fails() {
        let root=std::env::temp_dir().join(format!("gielinor-capture-bundle-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&root).unwrap();
        struct Cleanup(PathBuf);impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let _cleanup=Cleanup(root.clone());
        let blocked=root.join("frame-7-camera.json");std::fs::create_dir(&blocked).unwrap();
        let sidecars=vec![("-camera.json",b"camera".to_vec()),("-minimap.json",b"map".to_vec())];
        assert!(publish_capture(&root,7,b"pixels",&sidecars).is_err());
        assert!(!root.join("frame-7.png").exists(),"partial diagnostics must not acknowledge a frame");
        std::fs::remove_dir(&blocked).unwrap();publish_capture(&root,7,b"pixels",&sidecars).unwrap();
        assert_eq!(std::fs::read(&blocked).unwrap(),b"camera");
        assert_eq!(std::fs::read(root.join("frame-7-minimap.json")).unwrap(),b"map");
        assert_eq!(std::fs::read(root.join("frame-7.png")).unwrap(),b"pixels");
        assert!(std::fs::read_dir(&root).unwrap().all(|e|e.unwrap().path().extension().is_none_or(|x|x!="bmp")),"no BMP in a capture bundle");
    }
    #[test]
    fn capture_bundle_writes_sidecars_before_the_png_frame() {
        let root=std::env::temp_dir().join(format!("gielinor-capture-order-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&root).unwrap();
        struct Cleanup(PathBuf);impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let _cleanup=Cleanup(root.clone());
        // Block only the logical frame: sidecars are already published when it fails.
        let frame=root.join("frame-9.png");std::fs::create_dir(&frame).unwrap();
        let png=Raster::from_pixels(2,1,vec![0xff102030,0x00405060]).unwrap().png().unwrap();
        let sidecars=vec![("-camera.json",b"camera".to_vec()),("-minimap.png",png.clone())];
        assert!(publish_capture(&root,9,&png,&sidecars).is_err());
        assert_eq!(std::fs::read(root.join("frame-9-camera.json")).unwrap(),b"camera");
        assert_eq!(std::fs::read(root.join("frame-9-minimap.png")).unwrap(),png);
        assert!(frame.is_dir(),"the frame was never replaced by a partial write");
        std::fs::remove_dir(&frame).unwrap();
        publish_capture(&root,9,&png,&sidecars).unwrap();
        assert_eq!(std::fs::read(&frame).unwrap(),png);
        assert!(!root.join("frame-9.png.tmp").exists());
    }
    #[test]
    fn capture_camera_keeps_last_draw_separate_from_requested_pose() {
        let ray=crate::viewport_steering::Ray{size:[512,334],focal:777,far:3500,
            angles:[1600,300],eye:[-100,-300,14000],focus:[100,200,300]};
        let text=camera_diagnostic(Some((ray,[128,-300,13000])),[2048,900],[256,320]);
        assert!(text.contains("\"eye\":[-100, -300, 14000]"));
        assert!(text.contains("\"root_eye\":[128, -300, 13000]"));
        assert!(text.contains("\"angles\":[1600, 300]"));
        assert!(text.contains("\"requested_pitch_yaw\":[2048, 900]"));
        assert!(text.contains("\"focal\":777"));
        assert!(camera_diagnostic(None,[0,0],[256,256]).contains("\"last_world_draw\":null"));
    }
}

// ---------------------------------------------------------------------------
// Issue #12: public chat typed through the chatbox onkey script and a real
// minimap left click. Only the input updates the window pump would produce are
// dispatched, in H-pre (before_tick) into both input views; nothing builds a
// packet. Listeners run inside the interface tick (dispatch_interface_events),
// so the chat send is observed per tick, like the close driver. Gates come from
// the bounded cache read in reports/lead-b-20261024-2106/client-chat-minimap/p2-revised-plan/plan.md:
// [clientscript,chatdefault_onkey] (73), [proc,chat_send] (5517) and
// [proc,add_to_inputstring] (74). That read stopped at its 32-script cap with
// 14 chain nodes unread, so it is not a full script audit.
// ---------------------------------------------------------------------------
pub mod native_input {
    use crate::{client_state::ClientState, input::{Input, Update}, Result};

    /// Longer text gains a `|` from add_to_inputstring on each key-down (74:72-101).
    pub const CHAT_MAX_CHARS: usize = 15;
    /// Polls one typed character may take to appear in the chat input.
    pub const CHAT_STEP_POLLS: u32 = 250;
    /// Polls after Enter before an unsent message is refused.
    pub const CHAT_RESULT_POLLS: u32 = 250;
    const MESSAGE_PUBLIC: u16 = 69;
    /// varcstr: the chatbox input line (73:20, cleared at 73:805).
    pub const CHAT_INPUT: i32 = 335;
    /// varc: written with clientclock right after 5008 (5517:184-186) and nowhere
    /// else in the read chain, so a change is the client-side send witness.
    pub const CHAT_CLOCK: i32 = 61;
    /// varc: chat mode; anything but unset/0 changes the 5008 kind (5517:79-98).
    pub const CHAT_MODE: i32 = 945;
    /// 0: chatdefault_onkey returns before reading the key (73:10-12).
    pub const CHAT_TYPING_VARBIT: i32 = 8119;
    /// 1: chat_send shows the line locally and sends nothing (5517:0-23).
    pub const CHAT_LOCAL_VARBIT: i32 = 4394;
    /// chatdefault_onkey returns when either is hidden (73:0-9).
    pub const CHATBOX_INPUT: [u32; 2] = [162 << 16 | 55, 162 << 16 | 56];
    pub const KEY_ENTER: i32 = 84;
    /// Shift, Ctrl, Alt: never held, so typing and the minimap use no modifier.
    const MODIFIER_KEYS: [i32; 3] = [81, 82, 86];
    /// Tab (reply), Enter, Backspace and the scroll keys have their own branches.
    const RESERVED_KEYS: [i32; 5] = [80, 84, 85, 104, 105];

    fn dispatch(client: &mut ClientState, tick: &mut Input, update: Update) -> Result<()> {
        // The window pump's dispatch into both views with editor routing. The
        // clipboard is never read or written.
        crate::input::dispatch(update, [&mut client.input, tick], Some(&mut client.interfaces), |_| None, |_| false)
    }
    fn modifier_held(client: &ClientState, tick: Option<&Input>) -> bool {
        MODIFIER_KEYS.iter().any(|k| client.input.held.contains(k) || tick.is_some_and(|t| t.held.contains(k)))
    }
    fn chat_virtual_key(c: char) -> Option<usize> {
        Some(match c {
            'a'..='z' => c as usize - 32,
            '0'..='9' | ' ' => c as usize,
            '.' => 190,
            ',' => 188,
            '\'' => 222,
            '-' => 189,
            _ => return None,
        })
    }
    /// The (character, key code) of each character of a public chat text, or why
    /// it is refused. Plain [a-z0-9 .,'-] only: no Shift, no `:`/`/` commands.
    pub fn chat_keys(text: &str) -> std::result::Result<Vec<(u16, i32)>, String> {
        let count = text.chars().count();
        if count == 0 || count > CHAT_MAX_CHARS {
            return Err(format!("chat text must be 1..={CHAT_MAX_CHARS} characters"));
        }
        text.chars().map(|c| {
            chat_virtual_key(c).map(|vk| crate::windows_keys::KEYS[vk])
                .filter(|&code| code >= 0 && !RESERVED_KEYS.contains(&code))
                .map(|code| (c as u16, code))
                .ok_or_else(|| format!("character {c:?} is outside [a-z0-9 .,'-]"))
        }).collect()
    }
    /// The chat script state the gates read.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ChatGate { pub input: String, pub clock: i32, pub typing: i32, pub local_only: i32, pub mode: i32, pub hidden: [bool; 2] }
    impl ChatGate {
        pub fn json(&self) -> String {
            format!("{{\"input\":{},\"varc61\":{},\"varbit8119\":{},\"varbit4394\":{},\"varc945\":{},\"hidden_162_55_56\":[{},{}]}}",
                super::quoted(&self.input), self.clock, self.typing, self.local_only, self.mode, self.hidden[0], self.hidden[1])
        }
    }
    pub fn chat_gate(client: &mut ClientState) -> Result<ChatGate> {
        use crate::vm::{Host, Object, Value, Variable};
        fn int(client: &mut ClientState, kind: Variable, id: i32) -> Result<i32> {
            match client.scripts.read(kind, id)? {
                Value::Int(v) => Ok(v),
                _ => Err(crate::Error::Invalid("chat gate variable type")),
            }
        }
        let input = match client.scripts.read(Variable::ClientString, CHAT_INPUT)? {
            Value::Object(Object::String(text)) => text,
            _ => String::new(),
        };
        let hidden = CHATBOX_INPUT.map(|id| client.interfaces.widgets.get(&id).is_none_or(|w| w.hidden));
        Ok(ChatGate {
            input,
            clock: int(client, Variable::ClientInt, CHAT_CLOCK)?,
            typing: int(client, Variable::Bit, CHAT_TYPING_VARBIT)?,
            local_only: int(client, Variable::Bit, CHAT_LOCAL_VARBIT)?,
            mode: int(client, Variable::ClientInt, CHAT_MODE)?,
            hidden,
        })
    }
    /// Gates checked before every chat delivery (not the empty-input admission).
    pub fn chat_blocked(client: &mut ClientState, tick: &Input) -> Result<std::result::Result<ChatGate, String>> {
        if client.menu.is_some() || client.widget_drag.is_some() { return Ok(Err("a menu or widget drag is active".into())); }
        if client.interfaces.children.values().any(|s| s.kind == 0 || s.kind == 3) { return Ok(Err("a modal interface is open".into())); }
        if client.interfaces.focused_editor().is_some() { return Ok(Err("an editor has focus".into())); }
        if modifier_held(client, Some(tick)) { return Ok(Err("Shift, Ctrl or Alt is held".into())); }
        let gate = chat_gate(client)?;
        if gate.typing == 0 { return Ok(Err("varbit 8119 is 0: chatdefault_onkey ignores typing".into())); }
        if gate.local_only == 1 { return Ok(Err("varbit 4394 is 1: chat_send shows the line locally and sends nothing".into())); }
        if !matches!(gate.mode, -1 | 0) { return Ok(Err(format!("chat mode varc 945 is {}, not public", gate.mode))); }
        if gate.hidden.contains(&true) { return Ok(Err("chatbox input 162:55 or 162:56 is hidden".into())); }
        Ok(Ok(gate))
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ChatStage { Admit, Typing, Watching, Done }
    /// One `chat` command: admission once on an empty input, then one character per
    /// step, each delivered only after the input shows exactly the owned prefix,
    /// and Enter only when it shows the whole text.
    #[derive(Debug)]
    pub struct ChatDriver {
        pub sequence: u64,
        text: String,
        keys: Vec<(u16, i32)>,
        stage: ChatStage,
        typed: usize,
        polls: u32,
        clock: i32,
        /// Keys this driver pressed and has not released (only after a failed dispatch).
        held: Vec<i32>,
        telemetry: Vec<(i32, bool)>,
        outcome: Option<String>,
        /// Gate state at the last delivery or observation, for chat-mode.json.
        pub gate: Option<ChatGate>,
        /// Outbound packets already seen while watching (the queue drains every frame).
        seen: usize,
        /// Every packet queued by the ticks after Enter, and the MESSAGE_PUBLIC payloads.
        pub packets: Vec<u16>,
        publics: Vec<Vec<u8>>,
    }
    impl ChatDriver {
        pub fn new(sequence: u64, text: &str) -> std::result::Result<Self, String> {
            Ok(Self { sequence, text: text.to_owned(), keys: chat_keys(text)?, stage: ChatStage::Admit, typed: 0, polls: 0,
                clock: 0, held: Vec::new(), telemetry: Vec::new(), outcome: None, gate: None, seen: 0, packets: Vec::new(),
                publics: Vec::new() })
        }
        fn prefix(&self, count: usize) -> String { self.text.chars().take(count).collect() }
        fn finish(&mut self, status: String) {
            if self.stage != ChatStage::Done { self.stage = ChatStage::Done; self.outcome = Some(status); }
        }
        pub fn text(&self) -> &str { &self.text }
        pub fn typed(&self) -> usize { self.typed }
        /// H-pre: delivers at most one character (or Enter) on a ticking frame.
        pub fn before_tick(&mut self, client: &mut ClientState, tick: &mut Input, ticks: u32) {
            if ticks == 0 { return; }
            if self.stage == ChatStage::Watching { self.seen = client.outbound.len(); }
            if !matches!(self.stage, ChatStage::Admit | ChatStage::Typing) { return; }
            if let Err(error) = self.step(client, tick) {
                self.release_owned(Some(&mut client.input), tick);
                self.finish(format!("chat delivery failed: {error}"));
            }
        }
        fn step(&mut self, client: &mut ClientState, tick: &mut Input) -> Result<()> {
            let gate = match chat_blocked(client, tick)? {
                Ok(gate) => gate,
                Err(reason) => { self.finish(format!("chat refused: {reason}")); return Ok(()); }
            };
            self.gate = Some(gate.clone());
            if self.stage == ChatStage::Admit {
                if !gate.input.is_empty() {
                    self.finish(format!("chat refused: the chat input is not empty ({:?})", gate.input));
                    return Ok(());
                }
                self.stage = ChatStage::Typing;
                return self.type_next(client, tick);
            }
            let want = self.prefix(self.typed);
            if gate.input == want {
                if self.typed < self.keys.len() { return self.type_next(client, tick); }
                self.clock = gate.clock;
                self.stage = ChatStage::Watching;
                self.polls = 0;
                self.seen = client.outbound.len();
                return self.key(client, tick, KEY_ENTER, Some(10));
            }
            // The previous prefix only means the last character is not consumed yet.
            if gate.input != self.prefix(self.typed - 1) {
                self.finish(format!("chat input diverged: expected {want:?}, found {:?}", gate.input));
            }
            Ok(())
        }
        fn type_next(&mut self, client: &mut ClientState, tick: &mut Input) -> Result<()> {
            let (ch, code) = self.keys[self.typed];
            self.typed += 1;
            self.polls = 0;
            self.key(client, tick, code, Some(ch))
        }
        /// One key as the window pump delivers it: key down, its character, key up.
        fn key(&mut self, client: &mut ClientState, tick: &mut Input, code: i32, ch: Option<u16>) -> Result<()> {
            self.held.push(code);
            self.telemetry.push((code, true));
            dispatch(client, tick, Update::Key(code, true))?;
            if let Some(ch) = ch { dispatch(client, tick, Update::Character(ch))?; }
            dispatch(client, tick, Update::Key(code, false))?;
            self.held.retain(|&k| k != code);
            self.telemetry.push((code, false));
            Ok(())
        }
        /// After every internal tick: what the ticks after Enter queued.
        pub fn observe_tick(&mut self, client: &ClientState) {
            if self.stage != ChatStage::Watching { return; }
            for packet in client.outbound.iter().skip(self.seen) {
                self.packets.push(packet.opcode);
                if packet.opcode == MESSAGE_PUBLIC { self.publics.push(packet.payload.clone()); }
            }
            self.seen = client.outbound.len();
        }
        /// Releases only keys this driver pressed and still holds.
        fn release_owned(&mut self, client: Option<&mut Input>, tick: &mut Input) {
            let held = std::mem::take(&mut self.held);
            for &code in &held { tick.apply(Update::Key(code, false)); self.telemetry.push((code, false)); }
            if let Some(client) = client { for &code in &held { client.apply(Update::Key(code, false)); } }
        }
        fn observe(&mut self, client: &mut ClientState) -> Result<Option<String>> {
            let gate = chat_gate(client)?;
            let (cleared, clock) = (gate.input.is_empty(), gate.clock != self.clock);
            self.gate = Some(gate);
            match self.publics.len() {
                0 if self.polls >= CHAT_RESULT_POLLS => Ok(Some(format!(
                    "Enter queued no MESSAGE_PUBLIC within the bound (varc 61 changed: {clock}, input cleared: {cleared})"))),
                0 => Ok(None),
                1 => {
                    // The same table 5008 uses; the payload must be the public type-0 text.
                    let table = client.scripts.source.cache.named_file(10, "huffman", "")?;
                    let codec = crate::huffman::Huffman::new(&table)?;
                    let expected = crate::chat_packet::public_message(0, &self.text, -1, &codec)?;
                    Ok(Some(if self.publics[0] != expected.payload {
                        "MESSAGE_PUBLIC does not encode the typed text as public type 0".into()
                    } else if !cleared || !clock {
                        format!("MESSAGE_PUBLIC queued but chat_send did not finish (varc 61 changed: {clock}, input cleared: {cleared})")
                    } else {
                        "accepted: MESSAGE_PUBLIC queued client-side".into()
                    }))
                }
                n => Ok(Some(format!("Enter queued {n} MESSAGE_PUBLIC packets"))),
            }
        }
        /// H-reset: a title/reconnect frame ends the chat; nothing more is typed.
        pub fn abort(&mut self, tick: &mut Input) {
            self.release_owned(None, tick);
            self.finish("chat aborted by a title or reconnect frame".into());
        }
        /// Key events for `Connection::keyboard_key`, applied by the adapter in the
        /// same frame's poll (one frame after the pump would).
        pub fn take_telemetry(&mut self) -> Vec<(i32, bool)> { std::mem::take(&mut self.telemetry) }
        /// Once per adapter poll; the final status when decided.
        pub fn poll(&mut self, client: &mut ClientState) -> Option<String> {
            self.polls = self.polls.saturating_add(1);
            match self.stage {
                ChatStage::Admit if self.polls >= CHAT_STEP_POLLS => self.finish("no interface tick before the chat timed out".into()),
                ChatStage::Typing if self.polls >= CHAT_STEP_POLLS => {
                    let found = self.gate.as_ref().map(|g| g.input.clone()).unwrap_or_default();
                    self.finish(format!("chat input did not show {:?} within the bound (found {found:?}); a script gate such as keypress_permit refused the key", self.prefix(self.typed)));
                }
                ChatStage::Watching => match self.observe(client) {
                    Err(error) => self.finish(format!("chat observation failed: {error}")),
                    Ok(Some(status)) => self.finish(status),
                    Ok(None) => {}
                },
                _ => {}
            }
            if self.stage == ChatStage::Done { self.outcome.take() } else { None }
        }
    }

    /// Ticking polls a resolved minimap click may wait for its press frame.
    pub const MINIMAP_ARM_POLLS: u32 = 50;
    /// Polls a delivered press may stay down before it is released and refused.
    pub const MINIMAP_PRESS_POLLS: u32 = 50;
    /// Matching pixels whose native menu is read before the search gives up.
    pub const MINIMAP_MENU_PROBES: usize = 8;
    const MOVE_MINIMAP_CLICK: u16 = 67;
    /// A minimap pixel whose forward `minimap::click_target` is the requested tile,
    /// with everything `Connection::click` reads when it is resolved.
    #[derive(Clone, Debug)]
    pub struct MinimapTarget {
        pub widget: u32,
        pub instance: u64,
        pub point: [i32; 2],
        pub origin: [i32; 2],
        pub tile: [u16; 2],
        pub base: [i32; 2],
        pub yaw: u16,
        pub fine: [i32; 2],
        pub mask: crate::minimap::Mask,
    }
    impl MinimapTarget {
        fn local(&self) -> [i32; 2] { [self.tile[0] as i32 - self.base[0], self.tile[1] as i32 - self.base[1]] }
        fn scene_fine(&self) -> [i32; 2] { [self.fine[0] - self.base[0] * 128, self.fine[1] - self.base[1] * 128] }
        pub fn json(&self) -> String {
            format!("\"widget\":{},\"instance\":{},\"point\":[{},{}],\"tile\":[{},{}],\"yaw\":{}",
                self.widget, self.instance, self.point[0], self.point[1], self.tile[0], self.tile[1], self.yaw)
        }
    }
    /// Every pixel of `clip` whose forward click target is `local`, nearest first to
    /// the centroid of that set (an interior pixel of the tile's footprint).
    pub fn minimap_pixels(mask: &crate::minimap::Mask, clip: [i32; 4], origin: [i32; 2], scene_fine: [i32; 2], yaw: u16, local: [i32; 2]) -> Result<Vec<[i32; 2]>> {
        let mut hits = Vec::new();
        for y in clip[1]..clip[3] {
            for x in clip[0]..clip[2] {
                if let Some((target, _)) = crate::minimap::click_target(mask, [x - origin[0], y - origin[1]], scene_fine, yaw, 4.0)? {
                    if target == local { hits.push([x, y]); }
                }
            }
        }
        if hits.is_empty() { return Ok(hits); }
        let n = hits.len() as i64;
        let centre = [hits.iter().map(|p| p[0] as i64).sum::<i64>() * 2 / n, hits.iter().map(|p| p[1] as i64).sum::<i64>() * 2 / n];
        hits.sort_by_key(|p| { let d = [p[0] as i64 * 2 - centre[0], p[1] as i64 * 2 - centre[1]]; (d[0] * d[0] + d[1] * d[1], p[1], p[0]) });
        Ok(hits)
    }
    /// State gates shared by resolution and revalidation: plain walk policy, no
    /// modifier (keys 0, never a control-run or telejump), no menu/drag/editor.
    fn minimap_blocked(client: &ClientState, tick: Option<&Input>) -> Option<String> {
        if !matches!(client.minimap_state, 0 | 3) { return Some(format!("minimap state {} does not accept clicks", client.minimap_state)); }
        let mode = client.interaction_policy.get(-1)[0];
        if mode != 1 { return Some(format!("minimap policy mode {mode} is not plain walking")); }
        if modifier_held(client, tick) { return Some("Shift, Ctrl or Alt is held".into()); }
        if client.menu.is_some() || client.widget_drag.is_some() { return Some("a menu or widget drag is active".into()); }
        if client.interfaces.focused_editor().is_some() { return Some("an editor has focus".into()); }
        None
    }
    /// The click route's own widget choice at `point` (`Connection::click`), and
    /// no pointer listener or editor under it that the press would also reach.
    fn route_at(client: &ClientState, layout: &crate::widget_layout::Layout, point: [i32; 2]) -> std::result::Result<u32, String> {
        let hit: Vec<u32> = layout.hit_test(point).collect();
        let Some(&id) = hit.iter().find(|&&id| client.interfaces.widgets[&id].content == 1338) else {
            return Err("no minimap widget under the point".into());
        };
        for &other in &hit {
            let w = &client.interfaces.widgets[&other];
            if matches!(w.kind, 11 | 12) { return Err("an editor is under the point".into()); }
            if [1400u16, 1401, 1402, 1411].iter().any(|&op| client.interfaces.listener(other, op).is_some()) {
                return Err(format!("widget {other} under the point has a click listener"));
            }
        }
        Ok(id)
    }
    /// Resolves `tile` against the live minimap: forward search, the click route's
    /// widget and an empty native menu at the chosen pixel.
    pub fn minimap_target(client: &mut ClientState, live: &mut crate::live_runtime::Connection, tile: [u16; 2]) -> Result<std::result::Result<MinimapTarget, String>> {
        if let Some(reason) = minimap_blocked(client, None) { return Ok(Err(reason)); }
        let Some((scene, _)) = live.world.as_ref() else { return Ok(Err("no loaded scene".into())); };
        let base = [scene.base_x, scene.base_y];
        let size = [scene.width as i32, scene.height as i32];
        let local = [tile[0] as i32 - base[0], tile[1] as i32 - base[1]];
        if local[0] < 0 || local[1] < 0 || local[0] >= size[0] || local[1] >= size[1] { return Ok(Err("tile is outside the loaded scene".into())); }
        if live.minimap_view(client).is_none() { return Ok(Err("minimap view unavailable".into())); }
        let layout = client.interfaces.layout(client.canvas)?;
        let Some(&widget) = layout.order.iter().find(|&&id| !layout.widgets[&id].hidden && client.interfaces.widgets[&id].content == 1338) else {
            return Ok(Err("no visible minimap widget".into()));
        };
        let Some(mask) = crate::minimap::widget_mask(&mut client.scripts.source.cache, &client.interfaces.widgets[&widget], true)? else {
            return Ok(Err("minimap mask unavailable".into()));
        };
        let placed = &layout.widgets[&widget];
        let (origin, clip) = (placed.absolute, placed.clip);
        let fine = client.local_fine();
        let yaw = client.camera.yaw as u16;
        let scene_fine = [fine[0] - base[0] * 128, fine[1] - base[1] * 128];
        let pixels = minimap_pixels(&mask, clip, origin, scene_fine, yaw, local)?;
        if pixels.is_empty() { return Ok(Err("no minimap pixel targets that tile".into())); }
        let instance = client.interfaces.runtime[&widget].instance;
        let (pointer, press) = (client.input.pointer, client.input.press_position);
        let mut probes = 0;
        let mut last = String::from("no matching pixel is routed to the minimap");
        for point in pixels {
            if route_at(client, &layout, point) != Ok(widget) { continue; }
            if probes == MINIMAP_MENU_PROBES { break; }
            probes += 1;
            // The native menu is built at the pointer; restored before returning.
            client.input.apply(Update::Pointer(point));
            let entries = live.context_menu(client, point);
            client.input.apply(Update::Pointer(pointer));
            client.input.press_position = press;
            let entries = entries?;
            if entries.is_empty() {
                return Ok(Ok(MinimapTarget { widget, instance, point, origin, tile, base, yaw, fine, mask }));
            }
            last = format!("the native menu at {point:?} offers {:?}, so a left click would not walk", entries[0].option);
        }
        Ok(Err(last))
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum MinimapStage { Armed, Pressed, Observed, Done }
    /// One `minimap` command: a client-only revalidation and press in H-pre, the live
    /// revalidation in the same frame's poll (before `pending_clicks`), observation of
    /// what `Connection::click` queued in that frame's `Adapter::frame`, and the
    /// release on the next ticking frame.
    #[derive(Debug)]
    pub struct MinimapDriver {
        pub sequence: u64,
        pub target: MinimapTarget,
        stage: MinimapStage,
        polls: u32,
        mark: usize,
        /// The live check of the press frame has run (Some(true) when it passed).
        checked: Option<bool>,
        /// The left press this driver delivered and has not released, per view.
        holds_tick: bool,
        holds_client: bool,
        telemetry: bool,
        pub packets: Option<Vec<u16>>,
        outcome: Option<String>,
    }
    impl MinimapDriver {
        pub fn arm(sequence: u64, target: MinimapTarget) -> Self {
            Self { sequence, target, stage: MinimapStage::Armed, polls: 0, mark: 0, checked: None, holds_tick: false,
                holds_client: false, telemetry: false, packets: None, outcome: None }
        }
        fn finish(&mut self, status: String) {
            if self.stage != MinimapStage::Done { self.stage = MinimapStage::Done; self.outcome = Some(status); }
        }
        /// Client-only forward revalidation at consume time (H-pre).
        pub fn revalidate(&self, client: &mut ClientState, tick: &Input) -> Result<std::result::Result<(), String>> {
            self.current(client, Some(tick))
        }
        /// The whole route `Connection::click` will read, from current state: gates,
        /// widget instance, visibility and origin, the click route at the point, the
        /// camera and player, and the forward target through the widget's mask as
        /// loaded now (a tick may change the sprite or effects behind the cached one).
        fn current(&self, client: &mut ClientState, tick: Option<&Input>) -> Result<std::result::Result<(), String>> {
            let t = &self.target;
            if let Some(reason) = minimap_blocked(client, tick) { return Ok(Err(reason)); }
            if client.interfaces.runtime.get(&t.widget).is_none_or(|r| r.instance != t.instance) { return Ok(Err("minimap widget was replaced".into())); }
            if client.camera.yaw as u16 != t.yaw { return Ok(Err("camera yaw changed since resolution".into())); }
            if client.local_fine() != t.fine { return Ok(Err("player moved since resolution".into())); }
            let layout = client.interfaces.layout(client.canvas)?;
            if layout.widgets.get(&t.widget).is_none_or(|p| p.hidden || p.absolute != t.origin) { return Ok(Err("minimap widget moved or is hidden".into())); }
            match route_at(client, &layout, t.point) {
                Err(reason) => return Ok(Err(reason)),
                Ok(id) if id != t.widget => return Ok(Err("the click route no longer reaches the minimap at the point".into())),
                Ok(_) => {}
            }
            let Some(mask) = crate::minimap::widget_mask(&mut client.scripts.source.cache, &client.interfaces.widgets[&t.widget], true)? else {
                return Ok(Err("minimap mask unavailable".into()));
            };
            if (mask.width, &mask.starts, &mask.widths) != (t.mask.width, &t.mask.starts, &t.mask.widths) {
                return Ok(Err("minimap mask changed since resolution".into()));
            }
            match crate::minimap::click_target(&mask, [t.point[0] - t.origin[0], t.point[1] - t.origin[1]], t.scene_fine(), t.yaw, 4.0)? {
                Some((target, _)) if target == t.local() => Ok(Ok(())),
                _ => Ok(Err("the point no longer targets the tile".into())),
            }
        }
        /// H-pre: press on the first ticking frame after resolution; release on the
        /// next ticking frame, after the press frame's live check has run.
        pub fn before_tick(&mut self, client: &mut ClientState, tick: &mut Input, ticks: u32) {
            if ticks == 0 { return; }
            if self.stage == MinimapStage::Armed {
                if let Err(error) = self.press(client, tick) {
                    self.release(Some(&mut client.input), tick);
                    self.finish(format!("minimap delivery failed: {error}"));
                }
            } else if self.checked.is_some() && (self.holds_tick || self.holds_client) {
                self.release(Some(&mut client.input), tick);
                if self.stage == MinimapStage::Pressed { self.finish("the minimap press frame was not observed".into()); }
            }
        }
        fn press(&mut self, client: &mut ClientState, tick: &mut Input) -> Result<()> {
            if let Err(reason) = self.revalidate(client, tick)? { self.finish(format!("minimap refused: {reason}")); return Ok(()); }
            let point = self.target.point;
            self.mark = client.outbound.len();
            self.stage = MinimapStage::Pressed;
            self.polls = 0;
            self.holds_tick = true;
            self.holds_client = true;
            dispatch(client, tick, Update::Pointer(point))?;
            dispatch(client, tick, Update::ButtonAt(0, true, point))
        }
        /// Releases only the left press this driver delivered.
        fn release(&mut self, client: Option<&mut Input>, tick: &mut Input) {
            let point = self.target.point;
            if self.holds_tick {
                if tick.buttons & 1 != 0 { tick.apply(Update::ButtonAt(0, false, point)); }
                tick.pressed_buttons &= !1;
                self.holds_tick = false;
            }
            if let Some(client) = client {
                if self.holds_client {
                    if client.buttons & 1 != 0 { client.apply(Update::ButtonAt(0, false, point)); }
                    client.pressed_buttons &= !1;
                    self.holds_client = false;
                }
            }
        }
        /// Once per adapter poll. On the press frame this runs after the tick and
        /// immediately before `pending_clicks`: the whole current route (as in H-pre,
        /// with a fresh mask) and the live state are checked again, and on failure the
        /// owned press is taken back from the frame view so no click is dispatched.
        /// `live` is None only where no connection exists; the press-frame check then
        /// refuses.
        pub fn poll(&mut self, client: &mut ClientState, live: Option<&mut crate::live_runtime::Connection>) -> Option<String> {
            self.polls = self.polls.saturating_add(1);
            // An ended click (abort, refusal, timeout) gives back the frame-view press
            // here: title/reconnect frames never reach before_tick to release it.
            if self.stage == MinimapStage::Done && self.holds_client {
                client.input.pressed_buttons &= !1;
                if client.input.buttons & 1 != 0 { client.input.apply(Update::ButtonAt(0, false, self.target.point)); }
                self.holds_client = false;
            }
            self.settle();
            match self.stage {
                MinimapStage::Armed if self.polls >= MINIMAP_ARM_POLLS => self.finish("no interface tick before the minimap click timed out".into()),
                MinimapStage::Pressed if self.checked.is_none() => {
                    // A live-check error is a refusal, never an adapter exit.
                    // Errors are refusals, never an adapter exit.
                    let reason = match self.current(client, None) {
                        Err(error) => Some(format!("press-frame check failed: {error}")),
                        Ok(Err(reason)) => Some(format!("changed during the press tick: {reason}")),
                        Ok(Ok(())) => match live {
                            Some(live) => self.live_check(client, live).unwrap_or_else(|error| Some(format!("live check failed: {error}"))),
                            None => Some("no live connection for the press-frame check".into()),
                        },
                    };
                    self.checked = Some(reason.is_none());
                    match reason {
                        None => self.telemetry = true,
                        Some(reason) => {
                            if self.holds_client {
                                client.input.pressed_buttons &= !1;
                                client.input.apply(Update::ButtonAt(0, false, self.target.point));
                                self.holds_client = false;
                            }
                            self.finish(format!("minimap refused at the press frame: {reason}"));
                        }
                    }
                }
                MinimapStage::Pressed | MinimapStage::Observed if self.polls >= MINIMAP_PRESS_POLLS => {
                    if self.holds_client { client.input.apply(Update::ButtonAt(0, false, self.target.point)); self.holds_client = false; }
                    if self.stage == MinimapStage::Pressed { self.finish("minimap press timed out before its release frame".into()); }
                }
                _ => {}
            }
            if self.stage == MinimapStage::Done && !self.holds_tick && !self.holds_client { return self.outcome.take(); }
            None
        }
        fn live_check(&self, client: &mut ClientState, live: &mut crate::live_runtime::Connection) -> Result<Option<String>> {
            let t = &self.target;
            let Some((scene, _)) = live.world.as_ref() else { return Ok(Some("no loaded scene".into())); };
            if [scene.base_x, scene.base_y] != t.base { return Ok(Some("the scene base changed".into())); }
            match live.minimap_view(client) {
                Some(view) if view.angle == t.yaw => {}
                _ => return Ok(Some("the minimap view changed".into())),
            }
            let entries = live.context_menu(client, t.point)?;
            if !entries.is_empty() { return Ok(Some(format!("the native menu offers {:?}", entries[0].option))); }
            Ok(None)
        }
        /// From `Adapter::frame`, after `pending_clicks` ran `Connection::click`.
        pub fn observe_frame(&mut self, client: &ClientState) {
            if self.stage != MinimapStage::Pressed || self.checked != Some(true) { return; }
            let added: Vec<&crate::protocol::Packet> = client.outbound.iter().skip(self.mark).collect();
            self.packets = Some(added.iter().map(|p| p.opcode).collect());
            self.stage = MinimapStage::Observed;
            let walks: Vec<_> = added.iter().filter(|p| p.opcode == MOVE_MINIMAP_CLICK).collect();
            let t = &self.target;
            let expected = crate::world_action::minimap_walk([t.tile[0] as i32, t.tile[1] as i32], 0, [0, 0], 0, [0, 0]).ok();
            let flag = ((t.tile[0] as u32 & 16383) << 14) | (t.tile[1] as u32 & 16383);
            let status = if walks.len() != 1 {
                format!("the click queued {} MOVE_MINIMAPCLICK packets", walks.len())
            } else if expected.is_none_or(|e| walks[0].payload.get(..5) != e.payload.get(..5)) {
                "MOVE_MINIMAPCLICK does not carry the tile with no modifier keys".into()
            } else if client.map_flag != Some(flag) {
                "the map flag does not mark the tile".into()
            } else {
                "accepted: minimap walk queued client-side".into()
            };
            self.outcome = Some(status);
        }
        /// After the release frame: finishes with the observed status.
        pub fn settle(&mut self) {
            if self.stage == MinimapStage::Observed && !self.holds_tick && !self.holds_client {
                let status = self.outcome.take().unwrap_or_default();
                self.finish(status);
            }
        }
        pub fn telemetry_sent(&self) -> bool { self.telemetry }
        /// The press is down and its live check has not run yet (this frame's poll).
        pub fn press_frame(&self) -> bool { self.stage == MinimapStage::Pressed && self.checked.is_none() }
        /// Releases a tick-view press that waited too long (every frame).
        pub fn watchdog(&mut self, tick: &mut Input) {
            if self.holds_tick && self.polls >= MINIMAP_PRESS_POLLS { self.release(None, tick); }
        }
        /// H-reset: releases the tick-view press and ends the click; the frame-view
        /// press is released by the next poll.
        pub fn abort(&mut self, tick: &mut Input) {
            self.release(None, tick);
            self.finish("minimap click aborted by a title or reconnect frame".into());
        }
    }
}
