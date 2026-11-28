/// ISAAC with wrapping 32-bit arithmetic and the client's reverse result order.
#[derive(Clone)]
pub struct Isaac {
    memory: [u32; 256],
    results: [u32; 256],
    a: u32,
    b: u32,
    c: u32,
    left: usize,
}
impl Isaac {
    pub fn new(seed: &[u32]) -> Self {
        assert!(seed.len() <= 256, "ISAAC seed exceeds 256 words");
        let mut s = Self {
            memory: [0; 256],
            results: [0; 256],
            a: 0,
            b: 0,
            c: 0,
            left: 0,
        };
        s.results[..seed.len()].copy_from_slice(seed);
        let mut v = [0x9e3779b9u32; 8];
        for _ in 0..4 {
            mix(&mut v);
        }
        for i in (0..256).step_by(8) {
            for (j, x) in v.iter_mut().enumerate() {
                *x = x.wrapping_add(s.results[i + j]);
            }
            mix(&mut v);
            s.memory[i..i + 8].copy_from_slice(&v);
        }
        for i in (0..256).step_by(8) {
            for (j, x) in v.iter_mut().enumerate() {
                *x = x.wrapping_add(s.memory[i + j]);
            }
            mix(&mut v);
            s.memory[i..i + 8].copy_from_slice(&v);
        }
        s.generate();
        s
    }
    fn generate(&mut self) {
        self.c = self.c.wrapping_add(1);
        self.b = self.b.wrapping_add(self.c);
        for i in 0..256 {
            let x = self.memory[i];
            self.a ^= match i & 3 {
                0 => self.a << 13,
                1 => self.a >> 6,
                2 => self.a << 2,
                _ => self.a >> 16,
            };
            self.a = self.a.wrapping_add(self.memory[(i + 128) & 255]);
            let y = self.memory[((x >> 2) & 255) as usize]
                .wrapping_add(self.a)
                .wrapping_add(self.b);
            self.memory[i] = y;
            self.b = self.memory[((y >> 10) & 255) as usize].wrapping_add(x);
            self.results[i] = self.b;
        }
        self.left = 256;
    }
    pub fn next_word(&mut self) -> u32 {
        if self.left == 0 {
            self.generate();
        }
        self.left -= 1;
        self.results[self.left]
    }
    pub fn peek(&mut self) -> u32 {
        if self.left == 0 {
            self.generate();
        }
        self.results[self.left - 1]
    }
}
fn mix(v: &mut [u32; 8]) {
    v[0] ^= v[1] << 11;
    v[3] = v[3].wrapping_add(v[0]);
    v[1] = v[1].wrapping_add(v[2]);
    v[1] ^= v[2] >> 2;
    v[4] = v[4].wrapping_add(v[1]);
    v[2] = v[2].wrapping_add(v[3]);
    v[2] ^= v[3] << 8;
    v[5] = v[5].wrapping_add(v[2]);
    v[3] = v[3].wrapping_add(v[4]);
    v[3] ^= v[4] >> 16;
    v[6] = v[6].wrapping_add(v[3]);
    v[4] = v[4].wrapping_add(v[5]);
    v[4] ^= v[5] << 10;
    v[7] = v[7].wrapping_add(v[4]);
    v[5] = v[5].wrapping_add(v[6]);
    v[5] ^= v[6] >> 4;
    v[0] = v[0].wrapping_add(v[5]);
    v[6] = v[6].wrapping_add(v[7]);
    v[6] ^= v[7] << 8;
    v[1] = v[1].wrapping_add(v[6]);
    v[7] = v[7].wrapping_add(v[0]);
    v[7] ^= v[0] >> 9;
    v[2] = v[2].wrapping_add(v[7]);
    v[0] = v[0].wrapping_add(v[1]);
}

/// Encrypt/decrypt full blocks; the client's partial trailing block is untouched.
pub fn xtea(data: &mut [u8], key: [u32; 4], decrypt: bool) {
    for block in data.chunks_exact_mut(8) {
        let mut a = u32::from_be_bytes(block[..4].try_into().unwrap());
        let mut b = u32::from_be_bytes(block[4..].try_into().unwrap());
        const D: u32 = 0x9e3779b9;
        let mut sum = if decrypt { D.wrapping_mul(32) } else { 0 };
        for _ in 0..32 {
            if decrypt {
                b = b.wrapping_sub(
                    ((a << 4 ^ a >> 5).wrapping_add(a))
                        ^ sum.wrapping_add(key[((sum >> 11) & 3) as usize]),
                );
                sum = sum.wrapping_sub(D);
                a = a.wrapping_sub(
                    ((b << 4 ^ b >> 5).wrapping_add(b)) ^ sum.wrapping_add(key[(sum & 3) as usize]),
                );
            } else {
                a = a.wrapping_add(
                    ((b << 4 ^ b >> 5).wrapping_add(b)) ^ sum.wrapping_add(key[(sum & 3) as usize]),
                );
                sum = sum.wrapping_add(D);
                b = b.wrapping_add(
                    ((a << 4 ^ a >> 5).wrapping_add(a))
                        ^ sum.wrapping_add(key[((sum >> 11) & 3) as usize]),
                );
            }
        }
        block[..4].copy_from_slice(&a.to_be_bytes());
        block[4..].copy_from_slice(&b.to_be_bytes());
    }
}

pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ if crc & 1 != 0 { 0xedb88320 } else { 0 };
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xtea_zero_vector() {
        let mut b = [0u8; 9];
        b[8] = 42;
        xtea(&mut b, [0; 4], false);
        assert_eq!(&b[..8], &[0xde, 0xe9, 0xd4, 0xd8, 0xf7, 0x13, 0x1e, 0xd9]);
        xtea(&mut b, [0; 4], true);
        assert_eq!(b, [0, 0, 0, 0, 0, 0, 0, 0, 42]);
    }
    #[test]
    fn checksum() {
        assert_eq!(crc32(b"123456789"), 0xcbf43926);
    }
    #[test]
    fn isaac_zero_vector() {
        let mut i = Isaac::new(&[0; 4]);
        assert_eq!(i.peek(), 0x182600f3);
        assert_eq!(i.next_word(), 0x182600f3);
        assert_eq!(i.next_word(), 0x300b4a8d);
    }
}
