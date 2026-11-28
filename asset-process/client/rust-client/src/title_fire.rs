//! Title rune fire, recovered from cs, mn.ag and bu.af.
use crate::{buffer::Reader, raster::Raster, sprite::SpriteSheet, Error, Result};
pub fn colors(bytes: &[u8]) -> Result<[[u32; 5]; 3]> {
    let mut r = Reader::new(bytes);
    let mut colors = [[0; 5]; 3];
    loop {
        match r.u8()? {
            0 => break,
            1 => {
                r.u24()?;
            }
            2 => {
                for _ in 0..11 {
                    let head = r.u8()?;
                    if head & 128 != 0 {
                        r.u24()?;
                    } else {
                        r.u8()?;
                    }
                }
            }
            3 => {
                for row in &mut colors {
                    for c in row {
                        *c = r.u24()?;
                    }
                }
            }
            5 => {
                r.u32()?;
                r.u32()?;
            }
            _ => {}
        }
    }
    if r.remaining() != 0 {
        return Err(Error::Invalid("fire colors trailing bytes"));
    }
    Ok(colors)
}
pub fn palette(colors: [u32; 5]) -> [u32; 256] {
    std::array::from_fn(|i| {
        let (a, b, t) = (colors[i / 64], colors[i / 64 + 1], (i % 64) as i32);
        [16, 8, 0].into_iter().fold(0, |out, shift| {
            let a = ((a >> shift) & 255) as i32;
            let b = ((b >> shift) & 255) as i32;
            out | (((a + t * (b - a + 1) / 64) & 255) as u32) << shift
        })
    })
}
fn blend(a: u32, b: u32, t: u32) -> u32 {
    ((((a & 0xff00ff) * (256 - t) + (b & 0xff00ff) * t) & 0xff00ff00)
        + (((a & 0xff00) * (256 - t) + (b & 0xff00) * t) & 0xff0000))
        >> 8
}
pub struct Fire {
    runes: SpriteSheet,
    palettes: [[u32; 256]; 3],
    heat: Vec<i32>,
    scratch: Vec<i32>,
    noise: Vec<i32>,
    noise_scratch: Vec<i32>,
    offsets: [i32; 256],
    noise_offset: usize,
    phase: i32,
    sparks: i32,
    transitions: [i32; 2],
    last: i32,
    random: u64,
}
impl Fire {
    pub fn new(runes: SpriteSheet, colors: [[u32; 5]; 3], seed: u64) -> Result<Self> {
        if runes.sprites.len() < 12
            || runes
                .sprites
                .iter()
                .any(|s| s.x + s.width + 16 > 128 || s.y + s.height + 16 > 256)
        {
            return Err(Error::Invalid("title rune dimensions"));
        }
        let mut fire = Self {
            runes,
            palettes: colors.map(palette),
            heat: vec![0; 32768],
            scratch: vec![0; 32768],
            noise: vec![0; 32768],
            noise_scratch: vec![0; 32768],
            offsets: [0; 256],
            noise_offset: 0,
            phase: 0,
            sparks: 0,
            transitions: [0; 2],
            last: 0,
            random: (seed ^ 0x5deece66d) & ((1 << 48) - 1),
        };
        fire.seed_noise(None);
        Ok(fire)
    }
    fn bits(&mut self, n: u32) -> u64 {
        self.random = self.random.wrapping_mul(0x5deece66d).wrapping_add(11) & ((1 << 48) - 1);
        self.random >> (48 - n)
    }
    fn random(&mut self, n: usize) -> usize {
        let value = ((self.bits(26) << 27) + self.bits(27)) as f64 / (1u64 << 53) as f64;
        (value * n as f64) as usize
    }
    fn seed_noise(&mut self, rune: Option<usize>) {
        self.noise.fill(0);
        for _ in 0..5000 {
            let i = self.random(32768);
            self.noise[i] = self.random(256) as i32;
        }
        for _ in 0..20 {
            for y in 1..255 {
                for x in 1..127 {
                    let i = y * 128 + x;
                    self.noise_scratch[i] = (self.noise[i - 128]
                        + self.noise[i + 128]
                        + self.noise[i - 1]
                        + self.noise[i + 1])
                        / 4;
                }
            }
            std::mem::swap(&mut self.noise, &mut self.noise_scratch);
        }
        if let Some(rune) = rune {
            let sprite = &self.runes.sprites[rune];
            for y in 0..sprite.height {
                for x in 0..sprite.width {
                    if sprite.indices[y * sprite.width + x] != 0 {
                        self.noise[(y + sprite.y + 16) * 128 + x + sprite.x + 16] = 0;
                    }
                }
            }
        }
    }
    pub fn advance(&mut self, cycle: i32) {
        if self.last == 0 {
            self.last = cycle;
        }
        let mut ticks = cycle.wrapping_sub(self.last);
        if ticks >= 256 {
            ticks = 0;
        }
        self.last = cycle;
        if ticks <= 0 {
            return;
        }
        let n = ticks as usize;
        self.noise_offset += n * 128;
        if self.noise_offset > 32768 {
            self.noise_offset -= 32768;
            let rune = self.random(12);
            self.seed_noise(Some(rune));
        }
        for i in 0..(256 - n) * 128 {
            self.heat[i] = (self.heat[i + n * 128]
                - ticks * self.noise[(i + self.noise_offset) & 32767] / 6)
                .max(0);
        }
        for y in 256 - n..256 {
            for x in 0..128 {
                self.heat[y * 128 + x] = if self.random(100) < 50 && x > 10 && x < 118 {
                    255
                } else {
                    0
                };
            }
        }
        for value in &mut self.transitions {
            if *value > 0 {
                *value -= ticks * 4;
            }
        }
        if self.transitions == [0, 0] {
            match self.random((2000 / ticks) as usize) {
                0 => self.transitions[0] = 1024,
                1 => self.transitions[1] = 1024,
                _ => {}
            }
        }
        self.offsets.copy_within(n.., 0);
        for y in 256 - n..256 {
            self.offsets[y] = (libm::sin(self.phase as f64 / 14.0) * 16.0
                + libm::sin(self.phase as f64 / 15.0) * 14.0
                + libm::sin(self.phase as f64 / 16.0) * 12.0) as i32;
            self.phase = self.phase.wrapping_add(1);
        }
        self.sparks += ticks;
        let radius = (ticks + (cycle & 1)) / 2;
        if radius > 0 {
            for _ in 0..self.sparks * 100 {
                let x = self.random(124) + 2;
                let y = self.random(128) + 128;
                self.heat[y * 128 + x] = 192;
            }
            self.sparks = 0;
            for y in 0..256 {
                let mut sum = 0;
                for x in -radius..128 {
                    if x + radius < 128 {
                        sum += self.heat[y * 128 + (x + radius) as usize];
                    }
                    if x - radius - 1 >= 0 {
                        sum -= self.heat[y * 128 + (x - radius - 1) as usize];
                    }
                    if x >= 0 {
                        self.scratch[y * 128 + x as usize] = sum / (radius * 2 + 1);
                    }
                }
            }
            for x in 0..128 {
                let mut sum = 0;
                for y in -radius..256 {
                    if y + radius < 256 {
                        sum += self.scratch[(y + radius) as usize * 128 + x];
                    }
                    if y - radius - 1 >= 0 {
                        sum -= self.scratch[(y - radius - 1) as usize * 128 + x];
                    }
                    if y >= 0 {
                        self.heat[y as usize * 128 + x] = sum / (radius * 2 + 1);
                    }
                }
            }
        }
    }
    pub fn draw(&self, raster: &mut Raster, x: i32) {
        let colors = if self.transitions[0] > 0 {
            Some((self.transitions[0], &self.palettes[1]))
        } else if self.transitions[1] > 0 {
            Some((self.transitions[1], &self.palettes[2]))
        } else {
            None
        };
        let palette: [u32; 256] = std::array::from_fn(|i| match colors {
            Some((t, other)) if t > 768 => blend(self.palettes[0][i], other[i], (1024 - t) as u32),
            Some((t, other)) if t > 256 => other[i],
            Some((t, other)) => blend(other[i], self.palettes[0][i], (256 - t) as u32),
            None => self.palettes[0][i],
        });
        let clip = raster.clip();
        for row in 1..255 {
            let left = x + (256 - row as i32) * self.offsets[row] / 256;
            let y = row + 8;
            if y >= raster.height() {
                continue;
            }
            for column in 0..128 {
                let px = left + column as i32;
                if px < clip[0] as i32 || px >= clip[2] as i32 {
                    continue;
                }
                let alpha = self.heat[(row - 1) * 128 + column] as usize;
                if alpha != 0 {
                    raster.blend(px as usize, y, palette[alpha], alpha as u16);
                }
            }
        }
    }
}
