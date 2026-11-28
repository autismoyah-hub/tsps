use crate::{buffer::Transform, Error, Result};

#[derive(Debug, Default, Clone)]
pub struct Writer {
    data: Vec<u8>,
}
impl Writer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn bytes(&mut self, value: &[u8]) {
        self.data.extend_from_slice(value);
    }
    pub fn u8(&mut self, value: u8) {
        self.data.push(value);
    }
    pub fn transformed(&mut self, value: u8, transform: Transform) {
        self.u8(transform.encode(value));
    }
    pub fn u16(&mut self, value: u16) {
        self.bytes(&value.to_be_bytes());
    }
    pub fn u16_le(&mut self, value: u16) {
        self.bytes(&value.to_le_bytes());
    }
    pub fn u16_add(&mut self, value: u16, little: bool) {
        let [high, low] = value.to_be_bytes();
        if little {
            self.bytes(&[low.wrapping_add(128), high]);
        } else {
            self.bytes(&[high, low.wrapping_add(128)]);
        }
    }
    pub fn u24(&mut self, value: u32) -> Result<()> {
        if value > 0xff_ffff {
            return Err(Error::Invalid("u24 range"));
        }
        self.bytes(&value.to_be_bytes()[1..]);
        Ok(())
    }
    pub fn u32(&mut self, value: u32) {
        self.bytes(&value.to_be_bytes());
    }
    pub fn u32_le(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }
    pub fn u32_middle(&mut self, value: u32) {
        let b = value.to_be_bytes();
        self.bytes(&[b[1], b[0], b[3], b[2]]);
    }
    pub fn u32_inverse_middle(&mut self, value: u32) {
        let b = value.to_be_bytes();
        self.bytes(&[b[2], b[3], b[0], b[1]]);
    }
    pub fn u64(&mut self, value: u64) {
        self.bytes(&value.to_be_bytes());
    }
    pub fn unsigned_smart(&mut self, value: u16) -> Result<()> {
        if value > 32767 {
            return Err(Error::Invalid("unsigned smart range"));
        }
        if value < 128 {
            self.u8(value as u8);
        } else {
            self.u16(value + 32768);
        }
        Ok(())
    }
    pub fn signed_smart(&mut self, value: i16) -> Result<()> {
        if !(-16384..=16383).contains(&value) {
            return Err(Error::Invalid("signed smart range"));
        }
        if (-64..=63).contains(&value) {
            self.u8((value + 64) as u8);
        } else {
            self.u16((value as i32 + 49152) as u16);
        }
        Ok(())
    }
    pub fn large_smart(&mut self, value: u32) -> Result<()> {
        if value > 0x7fff_ffff {
            return Err(Error::Invalid("large smart range"));
        }
        if value < 32768 {
            self.u16(value as u16);
        } else {
            self.u32(value | 0x8000_0000);
        }
        Ok(())
    }
    pub fn varint(&mut self, value: u32) {
        let groups = (32 - value.leading_zeros()).max(1).div_ceil(7);
        for group in (0..groups).rev() {
            self.u8(((value >> (group * 7)) & 127) as u8 | if group != 0 { 128 } else { 0 });
        }
    }
    /// Java iterates UTF-16 code units, so a supplementary character yields two '?'.
    pub fn string(&mut self, text: &str) -> Result<()> {
        if text.contains('\0') {
            return Err(Error::Invalid("embedded string terminator"));
        }
        self.bytes(&encode_cp1252(text));
        self.u8(0);
        Ok(())
    }
    pub fn string_circumfixed(&mut self, text: &str) -> Result<()> {
        if text.contains('\0') {
            return Err(Error::Invalid("embedded string terminator"));
        }
        self.u8(0);
        self.string(text)
    }
    pub fn patch_u16(&mut self, offset: usize, value: usize) -> Result<()> {
        let value = u16::try_from(value).map_err(|_| Error::Invalid("u16 length overflow"))?;
        let end = offset
            .checked_add(2)
            .ok_or(Error::Invalid("length offset overflow"))?;
        self.data
            .get_mut(offset..end)
            .ok_or(Error::Invalid("length offset"))?
            .copy_from_slice(&value.to_be_bytes());
        Ok(())
    }
}

pub fn encode_cp1252(text: &str) -> Vec<u8> {
    text.encode_utf16()
        .map(|unit| {
            if (1..128).contains(&unit) || (160..256).contains(&unit) {
                return unit as u8;
            }
            match unit {
                0x20ac => 128,
                0x201a => 130,
                0x0192 => 131,
                0x201e => 132,
                0x2026 => 133,
                0x2020 => 134,
                0x2021 => 135,
                0x02c6 => 136,
                0x2030 => 137,
                0x0160 => 138,
                0x2039 => 139,
                0x0152 => 140,
                0x017d => 142,
                0x2018 => 145,
                0x2019 => 146,
                0x201c => 147,
                0x201d => 148,
                0x2022 => 149,
                0x2013 => 150,
                0x2014 => 151,
                0x02dc => 152,
                0x2122 => 153,
                0x0161 => 154,
                0x203a => 155,
                0x0153 => 156,
                0x017e => 158,
                0x0178 => 159,
                _ => b'?',
            }
        })
        .collect()
}

#[derive(Default)]
pub struct BitWriter {
    data: Vec<u8>,
    position: usize,
}
impl BitWriter {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn bits(&mut self, value: u32, count: u8) -> Result<()> {
        if count > 32 || (count < 32 && value >> count != 0) {
            return Err(Error::Invalid("bit field range"));
        }
        let end = self
            .position
            .checked_add(count as usize)
            .ok_or(Error::Invalid("bit offset overflow"))?;
        self.data.resize(end.div_ceil(8), 0);
        for bit in (0..count).rev() {
            self.data[self.position / 8] |= (((value >> bit) & 1) as u8) << (7 - self.position % 8);
            self.position += 1;
        }
        Ok(())
    }
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{BitReader, Reader};
    #[test]
    fn smart_boundaries() {
        let mut w = Writer::new();
        for n in [0, 127, 128, 32767] {
            w.unsigned_smart(n).unwrap();
        }
        for n in [-16384, -65, -64, 0, 63, 64, 16383] {
            w.signed_smart(n).unwrap();
        }
        for n in [0, 127, 128, 16384, u32::MAX] {
            w.varint(n);
        }
        let mut r = Reader::new(w.as_slice());
        for n in [0, 127, 128, 32767] {
            assert_eq!(r.unsigned_smart().unwrap(), n);
        }
        for n in [-16384, -65, -64, 0, 63, 64, 16383] {
            assert_eq!(r.signed_smart().unwrap(), n);
        }
        for n in [0, 127, 128, 16384, u32::MAX] {
            assert_eq!(r.varint().unwrap(), n);
        }
        let before = w.len();
        assert!(w.unsigned_smart(32768).is_err());
        assert_eq!(w.len(), before);
    }
    #[test]
    fn text_matches_java_utf16_semantics() {
        assert_eq!(encode_cp1252("€—🙂"), [128, 151, b'?', b'?']);
        let mut w = Writer::new();
        w.string("Rune€").unwrap();
        assert_eq!(w.as_slice(), b"Rune\x80\0");
        assert!(w.string_circumfixed("bad\0text").is_err());
        assert_eq!(w.len(), 6);
    }
    #[test]
    fn bit_packing_and_length_patch() {
        let mut w = BitWriter::new();
        w.bits(5, 3).unwrap();
        w.bits(0x12345678, 32).unwrap();
        w.bits(2, 2).unwrap();
        let b = w.into_vec();
        let mut r = BitReader::new(&b);
        assert_eq!(r.bits(3).unwrap(), 5);
        assert_eq!(r.bits(32).unwrap(), 0x12345678);
        assert_eq!(r.bits(2).unwrap(), 2);
        let mut w = Writer::new();
        w.u16(0);
        w.bytes(b"abc");
        w.patch_u16(0, 3).unwrap();
        assert_eq!(w.as_slice(), b"\0\x03abc");
        assert!(w.patch_u16(0, 65536).is_err());
        assert_eq!(w.as_slice(), b"\0\x03abc");
    }
}
