//! Revision 239.1 login encoding and fragmented server-response state machine.
use crate::{
    buffer::Reader,
    crypto::{self, Isaac},
    protocol,
    rsa::RsaPublicKey,
    writer::Writer,
    Error, Result,
};
use sha2::{Digest, Sha256};

pub fn random_seed() -> Result<[u32; 4]> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).map_err(|e| Error::Io(format!("OS random generator: {e}")))?;
    Ok(std::array::from_fn(|i| {
        u32::from_be_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap())
    }))
}

pub struct ProofOfWork {
    prefix: String,
    difficulty: u32,
}
impl ProofOfWork {
    /// Payload includes algorithm byte 0, then aq's challenge header and string.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        if r.u8()? != 0 {
            return Err(Error::Unsupported("login proof-of-work algorithm"));
        }
        let tag = r.u8()?;
        let difficulty = r.u8()? as u32;
        let text = r.string()?;
        if r.remaining() != 0 {
            return Err(Error::Invalid("proof-of-work trailing bytes"));
        }
        Ok(Self {
            prefix: format!("{tag:x}{difficulty:x}{text}"),
            difficulty,
        })
    }
    /// Bounded batches allow the game loop to keep processing input/cancellation.
    pub fn search(&self, first: u64, attempts: u64) -> Option<u64> {
        for offset in 0..attempts {
            let nonce = first.checked_add(offset)?;
            let input = format!("{}{nonce:x}", self.prefix);
            let hash = Sha256::digest(input.as_bytes());
            let zeros = hash
                .iter()
                .map(|&byte| byte.leading_zeros())
                .scan(false, |done, bits| {
                    if *done {
                        None
                    } else {
                        *done = bits != 8;
                        Some(bits)
                    }
                })
                .sum::<u32>();
            if zeros >= self.difficulty {
                return Some(nonce);
            }
        }
        None
    }
    pub fn reply(nonce: u64) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(19);
        w.u16(8);
        w.u64(nonce);
        w.into_vec()
    }
}

#[derive(Clone, Default)]
pub struct Platform {
    pub os: u8,
    pub is_64_bit: bool,
    pub os_version: u16,
    pub java_vendor: u8,
    pub java_version: [u8; 3],
    pub applet: bool,
    pub memory_mb: u16,
    pub processors: u8,
    pub system_memory: u32,
    pub system_speed: u16,
    pub gpu_dx: String,
    pub gpu_gl: String,
    pub gpu_dx_version: String,
    pub gpu_gl_version: String,
    pub driver_month: u8,
    pub driver_year: u16,
    pub cpu_manufacturer: String,
    pub cpu_brand: String,
    pub cpu_counts: [u8; 2],
    pub cpu_features: [u32; 3],
    pub cpu_signature: u32,
    pub client_name: String,
    pub device_name: String,
}
impl Platform {
    pub fn encode(&self, w: &mut Writer) -> Result<()> {
        w.u8(9);
        w.u8(self.os);
        w.u8(self.is_64_bit as u8);
        w.u16(self.os_version);
        w.u8(self.java_vendor);
        w.bytes(&self.java_version);
        w.u8(self.applet as u8);
        w.u16(self.memory_mb);
        w.u8(self.processors);
        w.u24(self.system_memory)?;
        w.u16(self.system_speed);
        for s in [
            &self.gpu_dx,
            &self.gpu_gl,
            &self.gpu_dx_version,
            &self.gpu_gl_version,
        ] {
            w.string_circumfixed(s)?;
        }
        w.u8(self.driver_month);
        w.u16(self.driver_year);
        w.string_circumfixed(&self.cpu_manufacturer)?;
        w.string_circumfixed(&self.cpu_brand)?;
        w.bytes(&self.cpu_counts);
        for word in self.cpu_features {
            w.u32(word);
        }
        w.u32(self.cpu_signature);
        w.string_circumfixed(&self.client_name)?;
        w.string_circumfixed(&self.device_name)?;
        Ok(())
    }
}

/// These wire discriminants come from ea, not the obfuscated enum ordinal.
pub enum Authentication {
    Password,
    Remembered(u32),
    Authenticator { code: u32, remember: bool },
    DateOfBirth(u32),
}
pub enum Credentials<'a> {
    Login {
        authentication: Authentication,
        secret: &'a str,
        token: bool,
    },
    Reconnect {
        previous_seed: [u32; 4],
    },
}
pub struct Request<'a> {
    pub username: &'a str,
    pub credentials: Credentials<'a>,
    pub client_build: u32,
    pub client_type: u8,
    pub platform_type: u8,
    pub resizable: bool,
    pub low_detail: bool,
    pub width: u16,
    pub height: u16,
    pub uuid: [u8; 24],
    pub site_settings: &'a str,
    pub affiliate: u32,
    pub deep_links: &'a [u32],
    pub platform: &'a Platform,
    pub archive_crcs: &'a [u32; 23],
}
impl Request<'_> {
    pub fn encode(&self, key: &RsaPublicKey, seed: [u32; 4], server_seed: u64) -> Result<Vec<u8>> {
        let mut rsa = Writer::new();
        rsa.u8(1);
        for word in seed {
            rsa.u32(word);
        }
        rsa.u64(server_seed);
        match &self.credentials {
            Credentials::Reconnect { previous_seed } => {
                for &word in previous_seed {
                    rsa.u32(word);
                }
            }
            Credentials::Login {
                authentication,
                secret,
                token,
            } => {
                match authentication {
                    Authentication::Password => {
                        rsa.u8(2);
                        rsa.u32(0);
                    }
                    Authentication::Remembered(value) => {
                        rsa.u8(0);
                        rsa.u32(*value);
                    }
                    Authentication::Authenticator { code, remember } => {
                        if *code > 999999 {
                            return Err(Error::Invalid("authenticator code range"));
                        }
                        rsa.u8(if *remember { 1 } else { 3 });
                        rsa.u24(*code)?;
                        rsa.u8(0);
                    }
                    Authentication::DateOfBirth(value) => {
                        rsa.u8(4);
                        rsa.u32(*value);
                    }
                }
                rsa.u8(if *token { 2 } else { 0 });
                rsa.string(secret)?;
            }
        }
        let mut private = Writer::new();
        private.string(self.username)?;
        private.u8((self.resizable as u8) << 1 | self.low_detail as u8);
        private.u16(self.width);
        private.u16(self.height);
        private.bytes(&self.uuid);
        private.string(self.site_settings)?;
        private.u32(self.affiliate);
        private.u8(u8::try_from(self.deep_links.len())
            .map_err(|_| Error::Invalid("login deep link count"))?);
        for &link in self.deep_links {
            private.u32(link);
        }
        self.platform.encode(&mut private)?;
        private.u8(0);
        private.u32(0);
        encode_archive_crcs(self.archive_crcs, &mut private);
        let mut private = private.into_vec();
        crypto::xtea(&mut private, seed, false);
        let mut out = Writer::new();
        out.u8(
            if matches!(self.credentials, Credentials::Reconnect { .. }) {
                18
            } else {
                16
            },
        );
        out.u16(0);
        out.u32(239);
        out.u32(1);
        out.u32(self.client_build);
        out.u8(self.client_type);
        out.u8(self.platform_type);
        out.u8(0);
        out.bytes(&key.encode_block(rsa.as_slice())?);
        out.bytes(&private);
        out.patch_u16(1, out.len() - 3)?;
        Ok(out.into_vec())
    }
}

/// cq.am archive order and per-word byte transforms. Slot 16 is always zero.
pub fn encode_archive_crcs(crcs: &[u32; 23], w: &mut Writer) {
    for (index, transform) in [
        (18, 3),
        (19, 3),
        (22, 0),
        (12, 1),
        (9, 1),
        (1, 3),
        (17, 3),
        (16, 2),
        (7, 2),
        (13, 1),
        (3, 0),
        (5, 3),
        (10, 2),
        (11, 0),
        (2, 3),
        (8, 1),
        (0, 2),
        (21, 0),
        (4, 2),
        (20, 3),
        (6, 1),
        (14, 2),
        (15, 1),
    ] {
        let value = if index == 16 { 0 } else { crcs[index] };
        match transform {
            0 => w.u32(value),
            1 => w.u32_le(value),
            2 => w.u32_middle(value),
            _ => w.u32_inverse_middle(value),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Account {
    pub remembered_token: Option<u32>,
    pub staff_level: u8,
    pub player_moderator: bool,
    pub player_index: u16,
    pub members: bool,
    pub account_hash: u64,
    pub user_id: u64,
    pub user_hash: u64,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    ServerSeed(u64),
    Success(Account),
    Reconnected(Vec<u8>),
    Rejected(u8),
    DateOfBirth { in_client: bool },
    HopDelay { cycles: u32 },
    Tokens(Vec<u8>),
    ProofOfWork(Vec<u8>),
    Message([String; 3]),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Ack,
    Seed,
    NeedRequest,
    Reply,
    Success,
    Reconnect,
    Tokens,
    Pow,
    Message,
    Hop,
    Game,
    Closed,
}

pub struct Decoder {
    state: State,
    bytes: Vec<u8>,
    cipher: Option<Isaac>,
    failed: bool,
}
impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}
impl Decoder {
    pub fn new() -> Self {
        Self {
            state: State::Ack,
            bytes: Vec::new(),
            cipher: None,
            failed: false,
        }
    }
    pub fn submitted(&mut self, seed: [u32; 4]) -> Result<()> {
        if self.state != State::NeedRequest {
            return Err(Error::Invalid("login request out of sequence"));
        }
        self.cipher = Some(Isaac::new(&seed.map(|x| x.wrapping_add(50))));
        self.state = State::Reply;
        Ok(())
    }
    pub fn feed(&mut self, bytes: &[u8]) -> Result<()> {
        if self.failed || self.state == State::Closed {
            return Err(Error::Invalid("closed login decoder"));
        }
        if bytes.len() > (1usize << 20).saturating_sub(self.bytes.len()) {
            return Err(Error::Invalid("login buffer limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    pub fn next_event(&mut self) -> Result<Option<Event>> {
        if self.failed {
            return Err(Error::Invalid("failed login decoder"));
        }
        let result = self.next_inner();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn next_inner(&mut self) -> Result<Option<Event>> {
        loop {
            match self.state {
                State::Ack => {
                    if self.bytes.is_empty() {
                        return Ok(None);
                    }
                    let code = self.bytes.remove(0);
                    if code != 0 {
                        self.state = State::Closed;
                        return Ok(Some(Event::Rejected(code)));
                    }
                    self.state = State::Seed;
                }
                State::Seed => {
                    if self.bytes.len() < 8 {
                        return Ok(None);
                    }
                    let seed = u64::from_be_bytes(self.bytes[..8].try_into().unwrap());
                    self.bytes.drain(..8);
                    self.state = State::NeedRequest;
                    return Ok(Some(Event::ServerSeed(seed)));
                }
                State::Reply => {
                    if self.bytes.is_empty() {
                        return Ok(None);
                    }
                    let code = self.bytes.remove(0);
                    self.state = match code {
                        2 => State::Success,
                        15 => State::Reconnect,
                        21 => State::Hop,
                        29 => State::Message,
                        61 => {
                            // GameClient.iz inspects the bytes available now; this flag is optional.
                            let in_client = self.bytes.len() == 1 && self.bytes.remove(0) == 1;
                            self.state = State::NeedRequest;
                            return Ok(Some(Event::DateOfBirth { in_client }));
                        }
                        64 => State::Tokens,
                        69 => State::Pow,
                        _ => {
                            self.state = State::Closed;
                            return Ok(Some(Event::Rejected(code)));
                        }
                    };
                }
                State::Hop => {
                    if self.bytes.is_empty() {
                        return Ok(None);
                    }
                    let value = self.bytes.remove(0);
                    self.state = State::Closed;
                    return Ok(Some(Event::HopDelay {
                        cycles: (value as u32 + 3) * 60,
                    }));
                }
                State::Success => {
                    if self.bytes.is_empty() {
                        return Ok(None);
                    }
                    let size = self.bytes[0] as usize;
                    // 34 account bytes + first game opcode/length (37 total in 239).
                    if size != 37 {
                        return Err(Error::Invalid("login success length for revision 239"));
                    }
                    if self.bytes.len() < size + 1 {
                        return Ok(None);
                    }
                    let mut r = Reader::new(&self.bytes[1..35]);
                    let present = r.u8()? == 1;
                    let mut token = 0u32;
                    let mut cipher = self
                        .cipher
                        .clone()
                        .ok_or(Error::Invalid("login ISAAC missing"))?;
                    for _ in 0..4 {
                        let byte = r.u8()?;
                        if present {
                            token =
                                (token << 8) | byte.wrapping_sub(cipher.next_word() as u8) as u32;
                        }
                    }
                    let account = Account {
                        remembered_token: present.then_some(token),
                        staff_level: r.u8()?,
                        player_moderator: r.u8()? == 1,
                        player_index: r.u16()?,
                        members: r.u8()? == 1,
                        account_hash: r.u64()?,
                        user_id: r.u64()?,
                        user_hash: r.u64()?,
                    };
                    self.cipher = Some(cipher);
                    self.bytes.drain(..35);
                    self.state = State::Game;
                    return Ok(Some(Event::Success(account)));
                }
                State::Reconnect | State::Tokens | State::Pow | State::Message => {
                    let prefix = if self.state == State::Tokens { 1 } else { 2 };
                    if self.bytes.len() < prefix {
                        return Ok(None);
                    }
                    let size = if prefix == 1 {
                        self.bytes[0] as usize
                    } else {
                        u16::from_be_bytes([self.bytes[0], self.bytes[1]]) as usize
                    };
                    if self.bytes.len() < prefix + size {
                        return Ok(None);
                    }
                    let data = self.bytes[prefix..prefix + size].to_vec();
                    let event = match self.state {
                        State::Reconnect => Event::Reconnected(data),
                        // GameClient's cd.ak branch skips these bytes without
                        // reading the ISAAC stream (the proxy differs here).
                        State::Tokens => Event::Tokens(data),
                        State::Pow => Event::ProofOfWork(data),
                        State::Message => {
                            let mut r = Reader::new(&data);
                            let messages = [r.string()?, r.string()?, r.string()?];
                            if r.remaining() != 0 {
                                return Err(Error::Invalid("login message trailing bytes"));
                            }
                            Event::Message(messages)
                        }
                        _ => unreachable!(),
                    };
                    self.bytes.drain(..prefix + size);
                    self.state = match self.state {
                        State::Reconnect => State::Game,
                        State::Message => State::Closed,
                        _ => State::Reply,
                    };
                    return Ok(Some(event));
                }
                State::NeedRequest | State::Game | State::Closed => return Ok(None),
            }
        }
    }
    pub fn into_game(self) -> Result<protocol::Decoder> {
        if self.failed || self.state != State::Game {
            return Err(Error::Invalid("login not established"));
        }
        let mut decoder = protocol::Decoder::new(self.cipher);
        decoder.feed(&self.bytes)?;
        Ok(decoder)
    }
}

/// Original login-state cd.ac countdown. Its display divides cycles by 60,
/// even though the native game loop runs at 50 cycles per second.
#[derive(Debug, Clone, Copy)]
pub struct TransferDelay {
    pub remaining: i32,
}
impl TransferDelay {
    fn lines(&self) -> [String; 3] {
        [
            "You have only just left another world.".into(),
            "Your profile will be transferred in:".into(),
            format!("{} seconds.", self.remaining / 60),
        ]
    }
    pub fn advance(&mut self) -> ([String; 3], bool) {
        let lines = self.lines();
        self.remaining = self.remaining.wrapping_sub(1);
        (lines, self.remaining <= 0)
    }
    /// Polling/rendering may occur without a simulation tick. Preserve the
    /// original pre-decrement text and stop at the first retry transition.
    pub fn advance_ticks(&mut self, ticks: u32) -> ([String; 3], bool) {
        let mut result = (self.lines(), false);
        for _ in 0..ticks {
            result = self.advance();
            if result.1 { break; }
        }
        result
    }
}

#[cfg(test)]
mod date_of_birth_reply_tests {
    use super::*;
    #[test]
    fn original_response61_available_byte_snapshot() {
        let mut r=Reader::new(include_bytes!("../tests/fixtures/login-reply61-jvm.bin"));
        for case in 0..r.u32().unwrap() {
            let count=r.u32().unwrap() as usize;
            let mut wire=vec![61];wire.extend_from_slice(r.bytes(count).unwrap());
            let remaining=r.u32().unwrap() as usize;let flag=r.u8().unwrap()!=0;
            let game=r.u32().unwrap();let index=r.u32().unwrap();let can_submit=r.u8().unwrap()!=0;let writes=r.u32().unwrap();
            let mut decoder=Decoder::new();decoder.feed(&[0,1,2,3,4,5,6,7,8]).unwrap();
            assert!(matches!(decoder.next_event().unwrap(),Some(Event::ServerSeed(_))));decoder.submitted([1,2,3,4]).unwrap();
            decoder.feed(&wire).unwrap();
            assert_eq!(decoder.next_event().unwrap(),Some(Event::DateOfBirth{in_client:flag}),"case {case}");
            assert_eq!(decoder.bytes.len(),remaining,"case {case}");
            assert!(decoder.next_event().unwrap().is_none());
            assert_eq!(decoder.state==State::NeedRequest,can_submit);
            assert_eq!((game,index,writes),(10,7,0));
        }
        assert_eq!(r.remaining(),0);
    }
}
