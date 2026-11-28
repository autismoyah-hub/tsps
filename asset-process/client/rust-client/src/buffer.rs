use crate::{Error, Result};

/// Explicit byte transforms used by the revision-specific packet layouts.
#[derive(Debug, Clone, Copy)]
pub enum Transform {
    None,
    Add,
    Neg,
    Sub,
}
impl Transform {
    pub fn decode(self, v: u8) -> u8 {
        match self {
            Self::None => v,
            Self::Add => v.wrapping_sub(128),
            Self::Neg => v.wrapping_neg(),
            Self::Sub => 128u8.wrapping_sub(v),
        }
    }
    pub fn encode(self, v: u8) -> u8 {
        match self {
            Self::None => v,
            Self::Add => v.wrapping_add(128),
            Self::Neg => v.wrapping_neg(),
            Self::Sub => 128u8.wrapping_sub(v),
        }
    }
}

#[derive(Clone)]
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    pub fn position(&self) -> usize {
        self.pos
    }
    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }
    pub fn bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(len).ok_or(Error::Truncated)?;
        let value = self.data.get(self.pos..end).ok_or(Error::Truncated)?;
        self.pos = end;
        Ok(value)
    }
    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.bytes(1)?[0])
    }
    pub fn transformed(&mut self, t: Transform) -> Result<u8> {
        Ok(t.decode(self.u8()?))
    }
    pub fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(self.bytes(2)?.try_into().unwrap()))
    }
    pub fn u16_le(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.bytes(2)?.try_into().unwrap()))
    }
    pub fn u16_add(&mut self, little: bool) -> Result<u16> {
        let b = self.bytes(2)?;
        Ok(if little {
            ((b[1] as u16) << 8) | b[0].wrapping_sub(128) as u16
        } else {
            ((b[0] as u16) << 8) | b[1].wrapping_sub(128) as u16
        })
    }
    pub fn u24(&mut self) -> Result<u32> {
        let b = self.bytes(3)?;
        Ok(((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32)
    }
    pub fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(self.bytes(4)?.try_into().unwrap()))
    }
    pub fn u32_le(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }
    /// Wire byte order 2,1,4,3 (most significant byte numbered 1).
    pub fn u32_middle(&mut self) -> Result<u32> {
        let b = self.bytes(4)?;
        Ok(u32::from_be_bytes([b[1], b[0], b[3], b[2]]))
    }
    /// Wire byte order 3,4,1,2.
    pub fn u32_inverse_middle(&mut self) -> Result<u32> {
        let b = self.bytes(4)?;
        Ok(u32::from_be_bytes([b[2], b[3], b[0], b[1]]))
    }
    pub fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_be_bytes(self.bytes(8)?.try_into().unwrap()))
    }
    pub fn unsigned_smart(&mut self) -> Result<u16> {
        let head = *self.data.get(self.pos).ok_or(Error::Truncated)?;
        if head < 128 {
            Ok(self.u8()? as u16)
        } else {
            Ok(self.u16()? - 32768)
        }
    }
    /// Chained unsigned smart, with 32767 continuation chunks (xm.dh).
    pub fn incremental_smart(&mut self) -> Result<u32> {
        let mut value = 0u32;
        loop {
            let chunk = self.unsigned_smart()? as u32;
            value = value
                .checked_add(chunk)
                .ok_or(Error::Invalid("incremental smart overflow"))?;
            if chunk != 32767 {
                return Ok(value);
            }
        }
    }
    pub fn signed_smart(&mut self) -> Result<i16> {
        let head = *self.data.get(self.pos).ok_or(Error::Truncated)?;
        if head < 128 {
            Ok(self.u8()? as i16 - 64)
        } else {
            Ok((self.u16()? as i32 - 49152) as i16)
        }
    }
    pub fn large_smart(&mut self) -> Result<u32> {
        let head = *self.data.get(self.pos).ok_or(Error::Truncated)?;
        if head & 128 != 0 {
            Ok(self.u32()? & 0x7fff_ffff)
        } else {
            Ok(self.u16()? as u32)
        }
    }
    pub fn varint(&mut self) -> Result<u32> {
        let mut value = 0u32;
        for _ in 0..5 {
            let b = self.u8()?;
            if value > (u32::MAX >> 7) {
                return Err(Error::Invalid("varint overflow"));
            }
            value = (value << 7) | (b & 127) as u32;
            if b & 128 == 0 {
                return Ok(value);
            }
        }
        Err(Error::Invalid("unterminated varint"))
    }
    /// xm.dp: least-significant base-128 group first.
    pub fn varint_le(&mut self) -> Result<u32> {
        let mut value = 0u32;
        for shift in (0..35).step_by(7) {
            let byte = self.u8()?;
            if shift == 28 && byte > 15 {
                return Err(Error::Invalid("little varint overflow"));
            }
            value |= ((byte & 127) as u32) << shift;
            if byte & 128 == 0 {
                return Ok(value);
            }
        }
        Err(Error::Invalid("unterminated little varint"))
    }
    /// xm.db: signed ZigZag over xm.dp.
    pub fn varint_le_signed(&mut self) -> Result<i32> {
        let v = self.varint_le()?;
        Ok((v >> 1) as i32 ^ -((v & 1) as i32))
    }
    pub fn string(&mut self) -> Result<String> {
        let len = self.data[self.pos..]
            .iter()
            .position(|&v| v == 0)
            .ok_or(Error::Truncated)?;
        let bytes = self.bytes(len)?;
        self.pos += 1;
        Ok(bytes.iter().map(|&v| cp1252(v)).collect())
    }
}

pub fn cp1252(b: u8) -> char {
    const EXT: [char; 32] = [
        '€', '?', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '?', 'Ž', '?', '?', '‘',
        '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '?', 'ž', 'Ÿ',
    ];
    if (128..160).contains(&b) {
        EXT[(b - 128) as usize]
    } else {
        b as char
    }
}

pub struct BitReader<'a> {
    data: &'a [u8],
    bit: usize,
}
impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, bit: 0 }
    }
    pub fn bits(&mut self, count: u8) -> Result<u32> {
        if count > 32 {
            return Err(Error::Invalid("bit width exceeds 32"));
        }
        let end = self
            .bit
            .checked_add(count as usize)
            .ok_or(Error::Truncated)?;
        if end.div_ceil(8) > self.data.len() {
            return Err(Error::Truncated);
        }
        let mut value = 0;
        for _ in 0..count {
            value = (value << 1) | ((self.data[self.bit / 8] >> (7 - self.bit % 8)) & 1) as u32;
            self.bit += 1;
        }
        Ok(value)
    }
    pub fn bytes_consumed(&self) -> usize {
        self.bit.div_ceil(8)
    }
    pub fn remaining_bits(&self)->usize {self.data.len()*8-self.bit}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wire_vectors() {
        let mut r = Reader::new(&[0x12, 0xb4, 0xb4, 0x12, 0x34, 0x12, 0x78, 0x56]);
        assert_eq!(r.u16_add(false), Ok(0x1234));
        assert_eq!(r.u16_add(true), Ok(0x1234));
        assert_eq!(r.u32_middle(), Ok(0x12345678));
        assert_eq!(r.u8(), Err(Error::Truncated));
        assert_eq!(
            Reader::new(&[0x56, 0x78, 0x12, 0x34]).u32_inverse_middle(),
            Ok(0x12345678)
        );
    }
    #[test]
    fn smarts_and_text() {
        let mut r = Reader::new(&[127, 0x80, 0x80, 0xff, 0xff, 0x81, 0, 0x80, b'A', 0]);
        assert_eq!(r.unsigned_smart(), Ok(127));
        assert_eq!(r.unsigned_smart(), Ok(128));
        assert_eq!(r.unsigned_smart(), Ok(32767));
        assert_eq!(r.varint(), Ok(128));
        assert_eq!(r.string().unwrap(), "€A");
        assert!(Reader::new(&[0xff; 5]).varint().is_err());
    }
    #[test]
    fn fragmented_bits() {
        let mut r = BitReader::new(&[0b10110101, 0b01100011]);
        assert_eq!(r.bits(3), Ok(5));
        assert_eq!(r.bits(7), Ok(0b1010101));
        assert_eq!(r.bits(6), Ok(0b100011));
        assert_eq!(r.bits(1), Err(Error::Truncated));
    }
    #[test]
    fn all_byte_transforms() {
        for t in [
            Transform::None,
            Transform::Add,
            Transform::Neg,
            Transform::Sub,
        ] {
            for v in 0..=255 {
                assert_eq!(t.decode(t.encode(v)), v);
            }
        }
    }
}
