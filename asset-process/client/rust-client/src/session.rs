//! Nonblocking native game connection. Poll from the window loop; credentials
//! are borrowed only during submission and are never retained by the session.
use crate::{
    crypto::Isaac,
    login, protocol,
    rsa::RsaPublicKey,
    Error, Result,
};
use std::{
    io::{ErrorKind, Read, Write},
    net::{Shutdown, SocketAddr, TcpStream},
    time::{Duration, Instant},
};

const QUEUE_LIMIT: usize = 1 << 20;
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Login(login::Event),
    Packet(protocol::Packet),
    ProofCompleted(u64),
    Disconnected,
}

pub struct Session {
    stream: Option<TcpStream>,
    login: Option<login::Decoder>,
    game: Option<protocol::Decoder>,
    outgoing_cipher: Option<Isaac>,
    pending: Vec<u8>,
    game_outgoing: crate::outgoing_batch::BatchQueue,
    sent: usize,
    keepalive: crate::keepalive::Keepalive,
    receive_timeout: crate::receive_timeout::ReceiveTimeout,
    server_seed: Option<u64>,
    proof: Option<(login::ProofOfWork, u64)>,
    last_received: Instant,
    timeout: Duration,
    closed: bool,
}
impl Session {
    /// Closed transport placeholder while connection establishment runs off the UI thread.
    pub fn disconnected() -> Self {
        Self {
            stream: None,
            login: None,
            game: None,
            outgoing_cipher: None,
            pending: Vec::new(),
            game_outgoing: Default::default(),
            sent: 0,
            keepalive: Default::default(),
            receive_timeout: Default::default(),
            server_seed: None,
            proof: None,
            last_received: Instant::now(),
            timeout: Duration::ZERO,
            closed: true,
        }
    }
    /// Connection establishment is bounded; subsequent IO is nonblocking.
    pub fn connect(address: SocketAddr, timeout: Duration) -> Result<Self> {
        if address.port() == 43594 {
            return Err(Error::Invalid("reserved live world port"));
        }
        Self::from_connected(TcpStream::connect_timeout(&address, timeout)?, timeout)
    }
    pub fn from_connected(stream: TcpStream, timeout: Duration) -> Result<Self> {
        if timeout.is_zero() {
            return Err(Error::Invalid("zero connection timeout"));
        }
        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;
        Ok(Self {
            stream: Some(stream),
            login: Some(login::Decoder::new()),
            game: None,
            outgoing_cipher: None,
            pending: vec![14],
            game_outgoing: Default::default(),
            sent: 0,
            keepalive: Default::default(),
            receive_timeout: Default::default(),
            server_seed: None,
            proof: None,
            last_received: Instant::now(),
            timeout,
            closed: false,
        })
    }
    pub fn is_established(&self) -> bool {
        self.game.is_some() && !self.closed
    }
    pub fn is_closed(&self) -> bool {
        self.closed
    }
    pub fn close(&mut self) {
        self.closed = true;
        self.proof = None;
        self.pending.clear();
        self.game_outgoing.clear();
        self.sent = 0;
        if let Some(stream) = self.stream.take() {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }
    /// Return the generated seed so the caller can retain it for reconnection.
    pub fn submit(&mut self, request: &login::Request<'_>, key: &RsaPublicKey) -> Result<[u32; 4]> {
        let seed = login::random_seed()?;
        self.submit_with_seed(request, key, seed)?;
        Ok(seed)
    }
    /// Deterministic entry point for protocol fixtures and reconnect bookkeeping.
    pub fn submit_with_seed(
        &mut self,
        request: &login::Request<'_>,
        key: &RsaPublicKey,
        seed: [u32; 4],
    ) -> Result<()> {
        if self.closed {
            return Err(Error::Invalid("closed connection"));
        }
        let server_seed = self
            .server_seed
            .ok_or(Error::Invalid("server seed unavailable"))?;
        let bytes = request.encode(key, seed, server_seed)?;
        self.ensure_capacity(bytes.len())?;
        self.login
            .as_mut()
            .ok_or(Error::Invalid("already logged in"))?
            .submitted(seed)?;
        self.outgoing_cipher = Some(Isaac::new(&seed));
        self.queue(&bytes);
        Ok(())
    }
    /// Apply packet construction reservations in the same order as completed nodes.
    pub fn send_event(&mut self, event: crate::outbound::Event) -> Result<()> {
        if !self.is_established() {
            return Err(Error::Invalid("game connection not established"));
        }
        if let crate::outbound::Event::Packet(packet) = &event {
            self.ensure_capacity(packet.payload.len().checked_add(4)
                .ok_or(Error::Invalid("packet size overflow"))?)?;
        }
        if let Some(bytes) = crate::outbound::encode(event, &mut self.outgoing_cipher)? {
            self.game_outgoing.push(bytes);
        }
        Ok(())
    }
    pub fn send(&mut self, opcode: u16, payload: &[u8]) -> Result<()> {
        if !self.is_established() {
            return Err(Error::Invalid("game connection not established"));
        }
        // Check before committing ISAAC state, including queue overflow.
        self.ensure_capacity(
            payload
                .len()
                .checked_add(4)
                .ok_or(Error::Invalid("packet size overflow"))?,
        )?;
        let bytes = protocol::encode_client(opcode, payload, &mut self.outgoing_cipher)?;
        self.game_outgoing.push(bytes);
        Ok(())
    }
    /// The active-game deadline is 751 cycles without original packet-reader progress.
    pub fn advance_receive_timeout(&mut self, ticks: u32, gameplay: bool) -> Result<()> {
        if self.is_established() && gameplay && self.receive_timeout.advance(ticks) {
            self.close();
            return Err(Error::Io("server response timed out".into()));
        }
        Ok(())
    }
    /// Advance only elapsed game cycles. Zero-tick transport polls do not age idle time.
    pub fn advance_keepalive(&mut self, ticks: u32, gameplay: bool) -> Result<()> {
        if !self.is_established() { return Ok(()); }
        for _ in 0..ticks {
            let due = if gameplay { self.keepalive.gameplay_tick() }
                else { self.keepalive.loading_tick(false) };
            if due { self.send(89, &[])?; }
            if gameplay || due { self.flush()?; }
        }
        Ok(())
    }
    fn ensure_capacity(&self, count: usize) -> Result<()> {
        if count > QUEUE_LIMIT.saturating_sub(self.pending.len() - self.sent + self.game_outgoing.pending_bytes()) {
            return Err(Error::Invalid("outgoing queue full"));
        }
        Ok(())
    }
    fn queue(&mut self, bytes: &[u8]) {
        if self.sent != 0 {
            self.pending.drain(..self.sent);
            self.sent = 0;
        }
        self.pending.extend_from_slice(bytes);
    }
    fn flush(&mut self) -> Result<()> {
        if self.pending.is_empty() {
            if let Some(batch) = self.game_outgoing.take_batch() {
                // Original oversized nodes remain at the head. An empty staging
                // write has no wire effect and must not mean a closed TCP stream.
                if batch.is_empty() {
                    self.keepalive.written();
                    return Ok(());
                }
                self.pending = batch;
            }
        }
        // Login/raw writes retain their bound; game batches contain whole nodes
        // within dw.af's 5000-byte staging limit. Partial socket writes resume.
        let end = self.pending.len().min(self.sent.saturating_add(65536));
        if self.sent < end {
            match self
                .stream
                .as_mut()
                .ok_or(Error::Invalid("disconnected transport"))?
                .write(&self.pending[self.sent..end])
            {
                Ok(0) => return Err(Error::Io("connection write closed".into())),
                Ok(n) => self.sent += n,
                Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::Interrupted) => {}
                Err(e) => return Err(e.into()),
            }
        }
        if self.sent != 0 && self.sent == self.pending.len() {
            self.keepalive.written();
            self.pending.clear();
            self.sent = 0;
        }
        Ok(())
    }
    fn decode(&mut self, events: &mut Vec<Event>, budget: &mut usize) -> Result<()> {
        // agq processes100 ordinary packets, but must finish an available group.
        while *budget > 0 || self.game.as_ref().is_some_and(|decoder| decoder.group_remaining() > 0) {
            if let Some(decoder) = self.login.as_mut() {
                let Some(event) = decoder.next_event()? else {
                    break;
                };
                match &event {
                    login::Event::ServerSeed(seed) => self.server_seed = Some(*seed),
                    login::Event::ProofOfWork(bytes) => {
                        self.proof = Some((login::ProofOfWork::decode(bytes)?, 0))
                    }
                    login::Event::Success(_) | login::Event::Reconnected(_) => {
                        self.game = Some(self.login.take().unwrap().into_game()?);
                        self.proof = None;
                    }
                    login::Event::Rejected(_)
                    | login::Event::Message(_)
                    | login::Event::HopDelay { .. } => self.close(),
                    _ => {}
                }
                *budget = budget.saturating_sub(1);
                events.push(Event::Login(event));
                if self.closed {
                    break;
                }
            } else if let Some(decoder) = self.game.as_mut() {
                let packet = decoder.next_server_packet()?;
                self.receive_timeout.observe(decoder.receive_progress());
                let Some(packet) = packet else {
                    break;
                };
                *budget = budget.saturating_sub(1);
                // Group markers affect transport availability, not gameplay state.
                if packet.opcode != 93 { events.push(Event::Packet(packet)); }
            } else {
                break;
            }
        }
        Ok(())
    }
    pub fn poll(&mut self) -> Result<Vec<Event>> {
        if self.closed {
            return Ok(Vec::new());
        }
        let result = self.poll_inner();
        if result.is_err() {
            self.close();
        }
        result
    }
    fn poll_inner(&mut self) -> Result<Vec<Event>> {
        let mut events = Vec::new();
        let mut budget = 100;
        // Keep outbound traffic moving even when decoded packets fill the poll budget.
        self.flush()?;
        self.decode(&mut events, &mut budget)?;
        if self.closed {
            return Ok(events);
        }
        if budget == 0 {
            return Ok(events);
        }
        self.flush()?;
        let mut bytes = [0; 16384];
        match self
            .stream
            .as_mut()
            .ok_or(Error::Invalid("disconnected transport"))?
            .read(&mut bytes)
        {
            Ok(0) => {
                self.close();
                events.push(Event::Disconnected);
                return Ok(events);
            }
            Ok(n) => {
                self.last_received = Instant::now();
                if let Some(decoder) = self.login.as_mut() {
                    decoder.feed(&bytes[..n])?;
                } else if let Some(decoder) = self.game.as_mut() {
                    decoder.feed(&bytes[..n])?;
                }
                self.decode(&mut events, &mut budget)?;
            }
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::Interrupted) => {}
            Err(e) => return Err(e.into()),
        }
        if self.closed {
            return Ok(events);
        }
        if self.game.is_none() && self.last_received.elapsed() >= self.timeout {
            return Err(Error::Io("server response timed out".into()));
        }
        if let Some((proof, first)) = self.proof.as_mut() {
            if let Some(nonce) = proof.search(*first, 2048) {
                let reply = login::ProofOfWork::reply(nonce);
                self.ensure_capacity(reply.len())?;
                self.queue(&reply);
                self.proof = None;
                events.push(Event::ProofCompleted(nonce));
            } else {
                *first = first
                    .checked_add(2048)
                    .ok_or(Error::Invalid("proof nonce exhausted"))?;
            }
        }
        self.flush()?;
        Ok(events)
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
#[path = "session_batch_tests.rs"]
mod batch_tests;
