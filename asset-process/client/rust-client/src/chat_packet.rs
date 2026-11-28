//! Revision 239 private/friends-channel/clan chat headers, filtering and text decoding.
use crate::{
    buffer::{cp1252, Reader},
    clan_channel::Channels,
    huffman::Huffman,
    social::Social,
    Error, Result,
};
#[derive(Clone, Debug)]
pub struct Incoming {
    pub opcode: u16,
    pub sender: String,
    pub slot: i8,
    pub channel: u64,
    pub uid: Option<u64>,
    pub player_type: u8,
    pub text: Vec<u8>,
}
impl Incoming {
    pub fn decode(opcode: u16, bytes: &[u8]) -> Result<Self> {
        if !matches!(opcode, 15 | 19 | 61 | 62 | 127) {
            return Err(Error::Invalid("chat message opcode"));
        }
        let mut r = Reader::new(bytes);
        let slot = if matches!(opcode, 62 | 127) {
            r.u8()? as i8
        } else {
            0
        };
        let sender = if opcode != 62 {
            r.string()?
        } else {
            String::new()
        };
        let channel = if opcode == 61 { r.u64()? } else { 0 };
        let uid = if opcode != 15 {
            Some(((r.u16()? as u64) << 32) + r.u24()? as u64)
        } else {
            None
        };
        let player_type = if matches!(opcode, 19 | 61 | 127) {
            r.u8()?
        } else {
            0
        };
        let text = r.bytes(r.remaining())?.to_vec();
        Ok(Self {
            opcode,
            sender,
            slot,
            channel,
            uid,
            player_type,
            text,
        })
    }
}
/// kl's moderator icon, privileged private-message color and ignore-filter policy.
pub fn player_type(id: u8) -> Result<(Option<u8>, bool, bool)> {
    let icon = match id {
        0 => None,
        1 => Some(0),
        2 => Some(1),
        3 => Some(2),
        4 => Some(3),
        5 => Some(10),
        6 => Some(22),
        7..=15 => Some(id + 34),
        16 => Some(52),
        _ => return Err(Error::Invalid("chat player type")),
    };
    Ok((icon, matches!(id, 1 | 2), id != 2))
}
pub fn base37(mut value: u64) -> Option<String> {
    if value == 0 || value >= 6582952005840035281 || value % 37 == 0 {
        return None;
    }
    let alphabet = b"_abcdefghijklmnopqrstuvwxyz0123456789";
    let mut chars: Vec<char> = Vec::new();
    while value != 0 {
        let mut c = alphabet[(value % 37) as usize] as char;
        value /= 37;
        if c == '_' {
            let previous = chars.last_mut().unwrap();
            *previous = previous.to_ascii_uppercase();
            c = '\u{a0}';
        }
        chars.push(c);
    }
    chars.reverse();
    chars[0] = chars[0].to_ascii_uppercase();
    Some(chars.into_iter().collect())
}
pub fn sentence_case(text: &str) -> String {
    let mut state = 2;
    let mut result = String::new();
    for c in text.chars() {
        let (upper, lower, is_upper) = crate::string_compare::casing(c as u16);
        let c = if state == 0 {
            char::from_u32(lower as u32).unwrap()
        } else if (state == 2 || is_upper) && !matches!(c, '\u{b5}' | '\u{192}') {
            char::from_u32(upper as u32).unwrap()
        } else {
            c
        };
        if c.is_alphabetic() {
            state = 0;
        } else if matches!(c, '.' | '?' | '!') {
            state = 2;
        } else if !matches!(c, ' ' | '\u{a0}') || state != 2 {
            state = 1;
        }
        result.push(c);
    }
    result
}
pub fn escape(text: &str) -> String {
    let mut result = String::new();
    for c in text.chars() {
        match c {
            '<' => result.push_str("<lt>"),
            '>' => result.push_str("<gt>"),
            _ => result.push(c),
        }
    }
    result
}
pub fn decompress(bytes: &[u8], codec: &Huffman) -> String {
    let decode = || -> Result<String> {
        let mut r = Reader::new(bytes);
        let length = r.unsigned_smart()? as usize;
        let (bytes, _) = codec.decode(r.bytes(r.remaining())?, length.min(32767), 32767)?;
        Ok(bytes.into_iter().filter(|&b| b != 0).map(cp1252).collect())
    };
    decode().unwrap_or_else(|_| "Cabbage".into())
}
pub struct Delivery {
    recent: [u64; 100],
    next: usize,
    pub restricted: bool,
}
impl Default for Delivery {
    fn default() -> Self {
        Self {
            recent: [0; 100],
            next: 0,
            restricted: false,
        }
    }
}
pub struct Message {
    pub kind: u16,
    pub sender: String,
    pub prefix: String,
    pub text: String,
}
impl Delivery {
    pub fn receive(
        &mut self,
        p: Incoming,
        social: &Social,
        channels: &Channels,
        codec: &Huffman,
    ) -> Result<Option<Message>> {
        if p.uid.is_some_and(|uid| self.recent.contains(&uid)) {
            return Ok(None);
        }
        let mut prefix = String::new();
        if matches!(p.opcode, 62 | 127) {
            let channel = if p.slot < 0 {
                &channels.guest
            } else {
                channels
                    .channels
                    .get(p.slot as usize)
                    .ok_or(Error::Invalid("chat clan slot"))?
            };
            let Some(channel) = channel else {
                return Ok(None);
            };
            prefix = channel.borrow().name.clone().unwrap_or_default();
        }
        let (icon, privileged, filter) = if matches!(p.opcode, 19 | 61 | 127) {
            player_type(p.player_type)?
        } else {
            (None, false, false)
        };
        if (p.opcode == 19 || filter) && social.is_ignored(&p.sender) {
            return Ok(None);
        }
        if matches!(p.opcode, 19 | 61) && self.restricted {
            return Ok(None);
        }
        if let Some(uid) = p.uid {
            self.recent[self.next] = uid;
            self.next = (self.next + 1) % 100;
        }
        let mut text = decompress(&p.text, codec);
        if matches!(p.opcode, 15 | 19 | 61) {
            text = sentence_case(&text);
        }
        if p.opcode != 62 {
            text = escape(&text);
        }
        if p.opcode == 61 {
            prefix = base37(p.channel).unwrap_or_default();
        }
        let kind = match p.opcode {
            15 => 6,
            19 => {
                if privileged {
                    7
                } else {
                    3
                }
            }
            61 => 9,
            62 => {
                if p.slot >= 0 {
                    43
                } else {
                    46
                }
            }
            127 => {
                if p.slot >= 0 {
                    41
                } else {
                    44
                }
            }
            _ => unreachable!(),
        };
        let sender = if let Some(icon) = icon {
            format!("<img={icon}>{}", p.sender)
        } else {
            p.sender
        };
        Ok(Some(Message {
            kind,
            sender,
            prefix,
            text,
        }))
    }
}
/// GameItemComposition.ag: CP1252 text preceded by its uncompressed smart length.
fn compressed(writer: &mut crate::writer::Writer, text: &str, codec: &Huffman) -> Result<()> {
    let bytes = crate::writer::encode_cp1252(text);
    let length = u16::try_from(bytes.len()).map_err(|_| Error::Invalid("chat text length"))?;
    writer.unsigned_smart(length)?;
    writer.bytes(&codec.encode(&bytes)?);
    Ok(())
}
pub fn private_message(
    recipient: &str,
    text: &str,
    codec: &Huffman,
) -> Result<crate::protocol::Packet> {
    let mut writer = crate::writer::Writer::new();
    writer.string(recipient)?;
    compressed(&mut writer, text, codec)?;
    Ok(crate::protocol::Packet {
        opcode: 111,
        payload: writer.into_vec(),
    })
}
/// GamePacketBuffer.ag. Prefix recognition is ordered: color, then effect.
pub fn public_message(
    kind: i32,
    text: &str,
    channel: i32,
    codec: &Huffman,
) -> Result<crate::protocol::Packet> {
    let mut text = text;
    let mut color = 0u8;
    let mut pattern = Vec::new();
    let prefixes = [
        "yellow:", "red:", "green:", "cyan:", "purple:", "white:", "flash1:", "flash2:", "flash3:",
        "glow1:", "glow2:", "glow3:", "rainbow:",
    ];
    if let Some((id, prefix)) = prefixes.iter().enumerate().find(|(_, p)| {
        text.get(..p.len())
            .is_some_and(|s| s.eq_ignore_ascii_case(p))
    }) {
        color = id as u8;
        text = &text[prefix.len()..];
    } else if text
        .get(..7)
        .is_some_and(|s| s.eq_ignore_ascii_case("pattern"))
    {
        let tail = &text.as_bytes()[7..];
        for &b in tail {
            if b == b':' {
                break;
            }
            if pattern.len() == 8 || !b.is_ascii_alphanumeric() {
                pattern.clear();
                break;
            }
            pattern.push(if b.is_ascii_digit() {
                b - b'0'
            } else {
                b.to_ascii_lowercase() - b'a' + 10
            });
        }
        if !pattern.is_empty() && tail.get(pattern.len()) == Some(&b':') {
            color = 12 + pattern.len() as u8;
            text = &text[8 + pattern.len()..];
        } else {
            pattern.clear();
        }
    }
    let mut effect = 0;
    for (id, prefix) in ["wave:", "wave2:", "shake:", "scroll:", "slide:"]
        .iter()
        .enumerate()
    {
        if text
            .get(..prefix.len())
            .is_some_and(|s| s.eq_ignore_ascii_case(prefix))
        {
            effect = id as u8 + 1;
            text = &text[prefix.len()..];
            break;
        }
    }
    let mut writer = crate::writer::Writer::new();
    writer.u8(kind as u8);
    writer.u8(color);
    writer.u8(effect);
    writer.bytes(&pattern);
    compressed(&mut writer, text, codec)?;
    if kind == 3 {
        writer.u8(channel as u8);
    }
    Ok(crate::protocol::Packet {
        opcode: 69,
        payload: writer.into_vec(),
    })
}
/// le.ca: surface/underground area restriction with the south-east exclusion.
pub fn restricted_area(x: i32, y: i32) -> bool {
    (((3053..=3156).contains(&x) && (3056..=3136).contains(&y))
        || ((3072..=3118).contains(&x) && (9492..=9535).contains(&y)))
        && !((3139..=3199).contains(&x) && (3008..=3062).contains(&y))
}
/// dk.aj history delivery. Actor overhead state is maintained separately.
pub fn player_message(
    p: &crate::player_update::Chat,
    name: &str,
    hidden: bool,
    social: &Social,
    restricted: bool,
) -> Result<Option<Message>> {
    let (icon, privileged, filter) = player_type(p.icon)?;
    if hidden || restricted || (filter && social.is_ignored(name)) {
        return Ok(None);
    }
    let kind = match (privileged, p.automatic) {
        (true, true) => 91,
        (true, false) => 1,
        (false, true) => 90,
        (false, false) => 2,
    };
    let sender = icon.map_or_else(|| name.to_owned(), |icon| format!("<img={icon}>{name}"));
    Ok(Some(Message {
        kind,
        sender,
        prefix: String::new(),
        text: escape(&sentence_case(&p.text)),
    }))
}
