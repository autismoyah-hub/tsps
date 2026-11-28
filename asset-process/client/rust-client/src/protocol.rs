use crate::{crypto::Isaac, protocol239::PacketSpec, Error, Result};

#[derive(Debug, PartialEq, Eq)]
pub struct Packet {
    pub opcode: u16,
    pub payload: Vec<u8>,
}

/// TCP framing commits cipher state only when a whole packet has arrived.
/// Use the client seed for outgoing and seed+50 for incoming ISAAC.
pub struct Decoder {
    buffer: Vec<u8>,
    cipher: Option<Isaac>,
    max_buffer: usize,
    group_remaining: i32,
    receive_progress: u64,
    observed_opcode_bytes: usize,
    observed_prefix_bytes: usize,
}
impl Decoder {
    pub fn new(cipher: Option<Isaac>) -> Self {
        Self {
            buffer: Vec::new(),
            cipher,
            max_buffer: 1 << 20,
            group_remaining: 0,
            receive_progress: 0,
            observed_opcode_bytes: 0,
            observed_prefix_bytes: 0,
        }
    }
    pub fn feed(&mut self, data: &[u8]) -> Result<()> {
        if data.len() > self.max_buffer.saturating_sub(self.buffer.len()) {
            return Err(Error::Invalid("packet buffer limit"));
        }
        self.buffer.extend_from_slice(data);
        Ok(())
    }
    pub fn group_remaining(&self) -> i32 {
        self.group_remaining
    }
    /// Changes when GameClient.ib would consume opcode bytes or a complete body.
    /// Prefix/body fragments and bytes behind a group gate do not reset its timer.
    pub fn receive_progress(&self) -> u64 { self.receive_progress }
    fn observe_prefix(&mut self) {
        let Some(&wire) = self.buffer.first() else { return; };
        let mut cipher = self.cipher.clone();
        let first = wire.wrapping_sub(cipher.as_mut().map_or(0, |c| c.next_word() as u8));
        let width = if first < 128 { 1 } else { 2 };
        let available = self.buffer.len().min(width);
        if available > self.observed_opcode_bytes {
            self.receive_progress = self.receive_progress.wrapping_add((available - self.observed_opcode_bytes) as u64);
            self.observed_opcode_bytes = available;
        }
        self.observed_prefix_bytes = available;
        let opcode = if width == 1 { first as i16 } else {
            let Some(&wire) = self.buffer.get(1) else { return; };
            let second = wire.wrapping_sub(cipher.as_mut().map_or(0, |c| c.next_word() as u8));
            ((first as i16 - 128) << 8) | second as i16
        };
        if let Some(spec) = crate::protocol239::SERVER_PACKETS.iter().find(|s| s.opcode == opcode) {
            let length_bytes = match spec.size { -1 => 1, -2 => 2, _ => 0 };
            if self.buffer.len() >= width + length_bytes {
                self.observed_prefix_bytes += length_bytes;
            }
        }
    }
    /// GameClient.ib availability gate, with agq's consumed-byte accounting.
    pub fn next_server_packet(&mut self) -> Result<Option<Packet>> {
        let grouped = self.group_remaining > 0;
        if grouped && self.buffer.len() - self.observed_prefix_bytes < self.group_remaining as usize {
            return Ok(None);
        }
        let previously_consumed = self.observed_prefix_bytes;
        self.observe_prefix();
        let before = self.buffer.len();
        let Some(mut packet) = self.next_packet(crate::protocol239::SERVER_PACKETS)? else {
            if grouped {
                self.group_remaining -= (self.observed_prefix_bytes - previously_consumed) as i32;
            }
            return Ok(None);
        };
        self.observed_opcode_bytes = 0;
        self.observed_prefix_bytes = 0;
        self.receive_progress = self.receive_progress.wrapping_add(1);
        // GameClient.ib / GamePacketBuffer.al decrypt the entire URL payload,
        // including bytes after its string terminator, before the next opcode.
        if packet.opcode == 39 {
            for byte in &mut packet.payload {
                *byte = byte.wrapping_sub(self.cipher.as_mut().map_or(0, |c| c.next_word() as u8));
            }
        }
        if packet.opcode == 93 {
            self.group_remaining = i16::from_be_bytes([packet.payload[0], packet.payload[1]]) as i32;
        }
        if grouped {
            self.group_remaining -= (before - self.buffer.len() - previously_consumed) as i32;
        }
        Ok(Some(packet))
    }
    pub fn next_packet(&mut self, specs: &[PacketSpec]) -> Result<Option<Packet>> {
        if self.buffer.is_empty() {
            return Ok(None);
        }
        let mut cipher = self.cipher.clone();
        let mut pos = 0;
        let mut byte = || -> Option<u8> {
            let value = *self.buffer.get(pos)?;
            pos += 1;
            Some(value.wrapping_sub(cipher.as_mut().map_or(0, |c| c.next_word() as u8)))
        };
        let first = byte().unwrap();
        let opcode = if first < 128 {
            first as u16
        } else {
            let Some(second) = byte() else {
                return Ok(None);
            };
            ((first as u16 - 128) << 8) | second as u16
        };
        let spec = specs
            .iter()
            .find(|s| s.opcode == opcode as i16)
            .ok_or(Error::Invalid("unknown opcode"))?;
        let size = match spec.size {
            -1 => {
                let Some(&n) = self.buffer.get(pos) else {
                    return Ok(None);
                };
                pos += 1;
                n as usize
            }
            -2 => {
                let Some(b) = self.buffer.get(pos..pos + 2) else {
                    return Ok(None);
                };
                pos += 2;
                u16::from_be_bytes([b[0], b[1]]) as usize
            }
            n if n >= 0 => n as usize,
            _ => return Err(Error::Invalid("packet size marker")),
        };
        let Some(payload) = self.buffer.get(pos..pos + size) else {
            return Ok(None);
        };
        let packet = Packet {
            opcode,
            payload: payload.to_vec(),
        };
        self.buffer.drain(..pos + size);
        self.cipher = cipher;
        Ok(Some(packet))
    }
}

pub fn encode(spec: &PacketSpec, payload: &[u8], cipher: &mut Option<Isaac>) -> Result<Vec<u8>> {
    if !(0..=32767).contains(&spec.opcode) {
        return Err(Error::Invalid("wire opcode"));
    }
    match spec.size {
        -1 if payload.len() <= 255 => {}
        -2 if payload.len() <= 65535 => {}
        n if n >= 0 && payload.len() == n as usize => {}
        _ => return Err(Error::Invalid("packet payload length")),
    }
    let mut out = Vec::new();
    let mut opcode_byte =
        |b: u8| out.push(b.wrapping_add(cipher.as_mut().map_or(0, |c| c.next_word() as u8)));
    if spec.opcode >= 128 {
        opcode_byte(((spec.opcode >> 8) + 128) as u8);
    }
    opcode_byte(spec.opcode as u8);
    match spec.size {
        -1 => out.push(payload.len() as u8),
        -2 => out.extend_from_slice(&(payload.len() as u16).to_be_bytes()),
        _ => {}
    }
    out.extend_from_slice(payload);
    Ok(out)
}

/// Frame actual client writes without changing the original declaration inventory.
pub fn encode_client(opcode: u16, payload: &[u8], cipher: &mut Option<Isaac>) -> Result<Vec<u8>> {
    let mut spec = *crate::protocol239::CLIENT_PACKETS.iter()
        .find(|spec| spec.opcode as u16 == opcode)
        .ok_or(Error::Invalid("client opcode"))?;
    // ot.gu326 / GamePlayerComposition.al enqueue exactly20 bytes; jf.cn declares26.
    // dw flushes the actual node length. The JVM writer/enqueue oracle verifies this.
    if opcode == 77 { spec.size = 20; }
    encode(&spec, payload, cipher)
}

pub fn js5_handshake(revision: u32, key: [u32; 4]) -> [u8; 21] {
    let mut out = [0; 21];
    out[0] = 15;
    out[1..5].copy_from_slice(&revision.to_be_bytes());
    for (slot, word) in out[5..].chunks_exact_mut(4).zip(key) {
        slot.copy_from_slice(&word.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol239::{CLIENT_PACKETS, SERVER_PACKETS};
    #[test]
    fn encrypted_fragmentation() {
        let specs = [
            PacketSpec {
                name: "TEST",
                opcode: 254,
                size: -2,
            },
            PacketSpec {
                name: "EMPTY",
                opcode: 3,
                size: 0,
            },
        ];
        let mut cipher = Some(Isaac::new(&[1, 2, 3, 4]));
        let mut wire = encode(&specs[0], b"hello", &mut cipher).unwrap();
        wire.extend(encode(&specs[1], &[], &mut cipher).unwrap());
        for split in 0..wire.len() {
            let mut d = Decoder::new(Some(Isaac::new(&[1, 2, 3, 4])));
            d.feed(&wire[..split]).unwrap();
            let mut packets = Vec::new();
            while let Some(p) = d.next_packet(&specs).unwrap() {
                packets.push(p);
            }
            d.feed(&wire[split..]).unwrap();
            while let Some(p) = d.next_packet(&specs).unwrap() {
                packets.push(p);
            }
            assert_eq!(
                packets,
                vec![
                    Packet {
                        opcode: 254,
                        payload: b"hello".to_vec()
                    },
                    Packet {
                        opcode: 3,
                        payload: vec![]
                    }
                ]
            );
        }
    }
    #[test]
    fn bad_length_does_not_advance_isaac() {
        let s = PacketSpec {
            name: "FIXED",
            opcode: 1,
            size: 2,
        };
        let mut c = Some(Isaac::new(&[]));
        let before = c.clone().unwrap().next_word();
        assert!(encode(&s, &[], &mut c).is_err());
        assert_eq!(c.unwrap().next_word(), before);
    }
    #[test]
    fn revision_tables_roundtrip() {
        for specs in [SERVER_PACKETS, CLIENT_PACKETS] {
            let mut c = Some(Isaac::new(&[5, 6, 7, 8]));
            let mut d = Decoder::new(c.clone());
            for s in specs {
                let data = vec![
                    42;
                    if s.size < 0 {
                        259.min(if s.size == -1 { 255 } else { 65535 })
                    } else {
                        s.size as usize
                    }
                ];
                d.feed(&encode(s, &data, &mut c).unwrap()).unwrap();
                assert_eq!(
                    d.next_packet(specs).unwrap(),
                    Some(Packet {
                        opcode: s.opcode as u16,
                        payload: data
                    })
                );
            }
        }
    }
    #[test]
    fn js5_vector() {
        assert_eq!(
            js5_handshake(239, [1, 2, 3, 4]),
            [15, 0, 0, 0, 239, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4]
        );
    }
}
