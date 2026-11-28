//! CPU framebuffer and clipped indexed-sprite drawing (original yf.yx).
use crate::{sprite::SpriteSheet, Error, Result};

pub struct Raster {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
    clip: [usize; 4],
}
impl Raster {
    pub fn from_pixels(width: usize, height: usize, pixels: Vec<u32>) -> Result<Self> {
        if width.checked_mul(height) != Some(pixels.len()) || pixels.len() > 16 << 20 {
            return Err(Error::Invalid("framebuffer dimensions"));
        }
        Ok(Self {
            width,
            height,
            pixels,
            clip: [0, 0, width, height],
        })
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn new(width: usize, height: usize, max_pixels: usize) -> Result<Self> {
        let count = width
            .checked_mul(height)
            .ok_or(Error::Invalid("raster dimensions"))?;
        if count > max_pixels || width > i32::MAX as usize || height > i32::MAX as usize {
            return Err(Error::Invalid("raster size limit"));
        }
        Ok(Self {
            width,
            height,
            pixels: vec![0; count],
            clip: [0, 0, width, height],
        })
    }
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }
    pub(crate) fn pixels_mut(&mut self) -> &mut [u32] { &mut self.pixels }
    pub fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: u32, alpha: u16) {
        let left = (x as i64).max(self.clip[0] as i64);
        let top = (y as i64).max(self.clip[1] as i64);
        let right = (x as i64 + width.max(0) as i64).min(self.clip[2] as i64);
        let bottom = (y as i64 + height.max(0) as i64).min(self.clip[3] as i64);
        for y in top..bottom {
            for x in left..right {
                self.blend(x as usize, y as usize, color, alpha);
            }
        }
    }
    /// Original yu.dz/ef interpolation keeps its phase when the top is clipped.
    pub fn fill_gradient(&mut self, rect: [i32; 4], colors: [u32; 2], alpha: Option<[u8; 2]>) {
        let [x, y, width, height] = rect;
        if width <= 0 || height <= 0 { return; }
        let left = (x as i64).max(self.clip[0] as i64);
        let top = (y as i64).max(self.clip[1] as i64);
        let right = (x as i64 + width as i64).min(self.clip[2] as i64);
        let bottom = (y as i64 + height as i64).min(self.clip[3] as i64);
        if left >= right || top >= bottom { return; }
        let step = 65536 / height as u32;
        for row in top..bottom {
            let phase = (row - y as i64) as u32 * step;
            let a = (65536 - phase) >> 8;
            let b = phase >> 8;
            let rb = ((colors[0] & 0xff00ff).wrapping_mul(a).wrapping_add((colors[1] & 0xff00ff).wrapping_mul(b))) & 0xff00ff00;
            let g = ((colors[0] & 0xff00).wrapping_mul(a).wrapping_add((colors[1] & 0xff00).wrapping_mul(b))) & 0xff0000;
            let color = rb.wrapping_add(g) >> 8;
            let opacity = alpha.map(|v| ((v[0] as u32 * a + v[1] as u32 * b) & 0xff00) >> 8);
            if opacity == Some(0) { continue; }
            let scale = |color: u32, alpha: u32| {
                (((color & 0xff00ff).wrapping_mul(alpha) >> 8) & 0xff00ff)
                    + (((color & 0xff00).wrapping_mul(alpha) >> 8) & 0xff00)
            };
            for column in left..right {
                let target = &mut self.pixels[row as usize * self.width + column as usize];
                *target = match opacity {
                    None => 0xff000000 | color,
                    Some(opacity) => scale(color, opacity) + scale(*target, 255 - opacity),
                };
            }
        }
    }

    /// Legacy widget rectangles use separate rounding for filled alpha spans.
    /// The software GameClient.mr path writes RGB without adding an alpha byte.
    pub fn widget_rect(&mut self, rect: [i32; 4], color: u32, alpha: u16, filled: bool) {
        let [x, y, w, h] = rect;
        if filled { self.widget_span(rect, color, alpha, true); }
        else {
            self.widget_span([x, y, w, 1], color, alpha, false);
            self.widget_span([x, y + h - 1, w, 1], color, alpha, false);
            if h >= 3 {
                self.widget_span([x, y + 1, 1, h - 2], color, alpha, false);
                self.widget_span([x + w - 1, y + 1, 1, h - 2], color, alpha, false);
            }
        }
    }
    fn widget_span(&mut self, rect: [i32; 4], color: u32, alpha: u16, separate: bool) {
        let [x, y, w, h] = rect;
        if alpha == 256 { self.fill_rect(x, y, w, h, color, alpha); return; }
        let left = (x as i64).max(self.clip[0] as i64);
        let top = (y as i64).max(self.clip[1] as i64);
        let right = (x as i64 + w.max(0) as i64).min(self.clip[2] as i64);
        let bottom = (y as i64 + h.max(0) as i64).min(self.clip[3] as i64);
        let a = alpha as u32;
        let b = 256 - a;
        let scale = |c: u32, a: u32| (((c & 0xff00ff).wrapping_mul(a) >> 8) & 0xff00ff)
            + (((c & 0xff00).wrapping_mul(a) >> 8) & 0xff00);
        for row in top..bottom {
            for column in left..right {
                let target = &mut self.pixels[row as usize * self.width + column as usize];
                *target = if separate { scale(color, a) + scale(*target, b) }
                    else {
                        let rb = ((color & 0xff00ff).wrapping_mul(a).wrapping_add((*target & 0xff00ff).wrapping_mul(b))) & 0xff00ff00;
                        let g = ((color & 0xff00).wrapping_mul(a).wrapping_add((*target & 0xff00).wrapping_mul(b))) & 0xff0000;
                        (rb | g) >> 8
                    };
            }
        }
    }
    pub(crate) fn blend(&mut self, x: usize, y: usize, color: u32, alpha: u16) {
        let a = alpha.min(256) as u32;
        let target = &mut self.pixels[y * self.width + x];
        if a == 256 {
            *target = color | 0xff000000;
        } else if a != 0 {
            let b = 256 - a;
            let rb = ((color & 0xff00ff) * a + (*target & 0xff00ff) * b) & 0xff00ff00;
            let g = ((color & 0xff00) * a + (*target & 0xff00) * b) & 0xff0000;
            *target = 0xff000000 | ((rb | g) >> 8);
        }
    }
    pub fn draw_mask(
        &mut self,
        mask: &[u8],
        width: usize,
        height: usize,
        x: i32,
        y: i32,
        color: u32,
        alpha: u16,
    ) -> Result<()> {
        if width.checked_mul(height) != Some(mask.len()) {
            return Err(Error::Invalid("glyph mask dimensions"));
        }
        let left = (x as i64).max(self.clip[0] as i64);
        let top = (y as i64).max(self.clip[1] as i64);
        let right = (x as i64 + width as i64).min(self.clip[2] as i64);
        let bottom = (y as i64 + height as i64).min(self.clip[3] as i64);
        for dy in top..bottom {
            for dx in left..right {
                if mask[(dy - y as i64) as usize * width + (dx - x as i64) as usize] != 0 {
                    // GameFontTypeFace.ak truncates the source and destination products separately.
                    let a = u32::from(alpha.min(256));
                    let scale = |color: u32, weight: u32| (((color & 0xff00ff).wrapping_mul(weight) & 0xff00ff00) + ((color & 0xff00).wrapping_mul(weight) & 0xff0000)) >> 8;
                    let pixel = &mut self.pixels[dy as usize * self.width + dx as usize];
                    *pixel = 0xff000000 | (scale(color, a) + scale(*pixel, 256 - a));
                }
            }
        }
        Ok(())
    }
    pub fn clear(&mut self, argb: u32) {
        self.pixels.fill(argb);
    }
    pub fn set_clip(&mut self, left: i32, top: i32, right: i32, bottom: i32) {
        let left = (left.max(0) as usize).min(self.width);
        let top = (top.max(0) as usize).min(self.height);
        self.clip = [
            left,
            top,
            (right.max(0) as usize).min(self.width).max(left),
            (bottom.max(0) as usize).min(self.height).max(top),
        ];
    }
    pub fn clip(&self)->[usize;4]{self.clip}
    pub fn draw_indexed(
        &mut self,
        sheet: &SpriteSheet,
        index: usize,
        x: i32,
        y: i32,
    ) -> Result<()> {
        let sprite = sheet
            .sprites
            .get(index)
            .ok_or(Error::Invalid("sprite index"))?;
        let x = i64::from(x) + sprite.x as i64;
        let y = i64::from(y) + sprite.y as i64;
        let left = x.max(self.clip[0] as i64);
        let top = y.max(self.clip[1] as i64);
        let right = (x + sprite.width as i64).min(self.clip[2] as i64);
        let bottom = (y + sprite.height as i64).min(self.clip[3] as i64);
        for dy in top..bottom {
            for dx in left..right {
                let source = (dy - y) as usize * sprite.width + (dx - x) as usize;
                let palette_index = *sprite
                    .indices
                    .get(source)
                    .ok_or(Error::Invalid("sprite pixel count"))?
                    as usize;
                if palette_index != 0 {
                    let color = *sheet
                        .palette
                        .get(palette_index)
                        .ok_or(Error::Invalid("sprite palette index"))?;
                    self.pixels[dy as usize * self.width + dx as usize] = color | 0xff000000;
                }
            }
        }
        Ok(())
    }
    /// Standalone 32-bit BMP for inspecting native rendering without a JVM.
    /// Explicit BMP CLI and exercise outputs keep this format; native
    /// adapter, shutdown and interface-preview evidence uses `png()`.
    pub fn bmp(&self) -> Result<Vec<u8>> {
        let size = self
            .pixels
            .len()
            .checked_mul(4)
            .and_then(|n| n.checked_add(54))
            .ok_or(Error::Invalid("BMP size"))?;
        let size = u32::try_from(size).map_err(|_| Error::Invalid("BMP size"))?;
        let mut out = Vec::with_capacity(size as usize);
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&[0; 4]);
        out.extend_from_slice(&54u32.to_le_bytes());
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&(self.width as i32).to_le_bytes());
        out.extend_from_slice(&(-(self.height as i32)).to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(&[0; 24]);
        for pixel in &self.pixels {
            out.extend_from_slice(&pixel.to_le_bytes());
        }
        Ok(out)
    }
    /// Opaque 8-bit RGB PNG with the same displayed colours as `bmp()`, which is
    /// BI_RGB and ignores the high byte. Fixed zlib level 6, filter 0, no interlace.
    pub fn png(&self) -> Result<Vec<u8>> {
        let width = u32::try_from(self.width).ok().filter(|&w| w > 0 && w <= i32::MAX as u32);
        let height = u32::try_from(self.height).ok().filter(|&h| h > 0 && h <= i32::MAX as u32);
        let (Some(width), Some(height)) = (width, height) else {
            return Err(Error::Invalid("PNG dimensions"));
        };
        let row = self.width.checked_mul(3).and_then(|n| n.checked_add(1)).ok_or(Error::Invalid("PNG size"))?;
        let raw = row.checked_mul(self.height).ok_or(Error::Invalid("PNG size"))?;
        let mut scanlines = Vec::with_capacity(raw);
        for line in self.pixels.chunks_exact(self.width) {
            scanlines.push(0);
            for pixel in line {
                scanlines.extend_from_slice(&[(pixel >> 16) as u8, (pixel >> 8) as u8, *pixel as u8]);
            }
        }
        if scanlines.len() != raw {
            return Err(Error::Invalid("PNG size"));
        }
        let mut header = Vec::with_capacity(13);
        header.extend_from_slice(&width.to_be_bytes());
        header.extend_from_slice(&height.to_be_bytes());
        header.extend_from_slice(&[8, 2, 0, 0, 0]);
        let data = miniz_oxide::deflate::compress_to_vec_zlib(&scanlines, 6);
        let mut out = Vec::with_capacity(data.len() + 57);
        out.extend_from_slice(b"\x89PNG\r\n\x1a\n");
        for (kind, body) in [(b"IHDR", &header[..]), (b"IDAT", &data[..]), (b"IEND", &[][..])] {
            let length = u32::try_from(body.len()).ok().filter(|&n| n <= i32::MAX as u32)
                .ok_or(Error::Invalid("PNG chunk size"))?;
            out.extend_from_slice(&length.to_be_bytes());
            let start = out.len();
            out.extend_from_slice(kind);
            out.extend_from_slice(body);
            let crc = crate::crypto::crc32(&out[start..]);
            out.extend_from_slice(&crc.to_be_bytes());
        }
        Ok(out)
    }
}

#[cfg(test)]
mod png_tests {
    use super::Raster;
    /// Chunks as (type, data), checking each stored length and CRC.
    fn chunks(png: &[u8]) -> Vec<([u8; 4], Vec<u8>)> {
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        let (mut at, mut out) = (8, Vec::new());
        while at < png.len() {
            let length = u32::from_be_bytes(png[at..at + 4].try_into().unwrap()) as usize;
            let kind: [u8; 4] = png[at + 4..at + 8].try_into().unwrap();
            let data = png[at + 8..at + 8 + length].to_vec();
            let crc = u32::from_be_bytes(png[at + 8 + length..at + 12 + length].try_into().unwrap());
            assert_eq!(crc, crate::crypto::crc32(&png[at + 4..at + 8 + length]), "CRC of {:?}", kind);
            out.push((kind, data));
            at += 12 + length;
        }
        assert_eq!(at, png.len());
        out
    }
    /// Odd width, alpha variants that must not change displayed RGB, primaries and the clear colour.
    fn known() -> Raster {
        let pixels = vec![
            0xffff0000, 0x0000ff00, 0x7f0000ff, 0xffffffff, 0x00000000, 0xff29251f, 0x80123456,
            0x00fedcba, 0xff010203, 0x7fa0b0c0, 0xff808080, 0x00ff00ff, 0xffffff00, 0x0000ffff,
            0xff000001, 0x01020304, 0xfffefefe, 0x00405060, 0x7f7f7f7f, 0xff00ff00, 0x12345678,
        ];
        Raster::from_pixels(7, 3, pixels).unwrap()
    }
    fn decoded_rgb(png: &[u8]) -> ([u32; 2], Vec<u8>) {
        let chunks = chunks(png);
        assert_eq!(chunks.iter().map(|c| c.0).collect::<Vec<_>>(), [*b"IHDR", *b"IDAT", *b"IEND"]);
        let ihdr = &chunks[0].1;
        assert_eq!(ihdr.len(), 13);
        let size = [u32::from_be_bytes(ihdr[0..4].try_into().unwrap()), u32::from_be_bytes(ihdr[4..8].try_into().unwrap())];
        assert_eq!(&ihdr[8..], &[8, 2, 0, 0, 0], "8-bit RGB, deflate, filter 0, no interlace");
        assert!(chunks[2].1.is_empty());
        let raw = miniz_oxide::inflate::decompress_to_vec_zlib(&chunks[1].1).unwrap();
        let row = 1 + 3 * size[0] as usize;
        assert_eq!(raw.len(), row * size[1] as usize);
        let mut rgb = Vec::new();
        for line in raw.chunks_exact(row) {
            assert_eq!(line[0], 0, "filter type 0 per scanline");
            rgb.extend_from_slice(&line[1..]);
        }
        (size, rgb)
    }
    #[test] fn png_structure_and_pixels_roundtrip_ignoring_alpha() {
        let raster = known();
        let (size, rgb) = decoded_rgb(&raster.png().unwrap());
        assert_eq!(size, [7, 3]);
        let expected: Vec<u8> = raster.pixels().iter().flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8]).collect();
        assert_eq!(rgb, expected);
    }
    #[test] fn png_matches_displayed_bmp_colours() {
        let raster = known();
        let bmp = raster.bmp().unwrap();
        let (_, rgb) = decoded_rgb(&raster.png().unwrap());
        // BMP pixels are little-endian B,G,R,X after the 54-byte header.
        let from_bmp: Vec<u8> = bmp[54..].chunks_exact(4).flat_map(|p| [p[2], p[1], p[0]]).collect();
        assert_eq!(rgb, from_bmp);
    }
    #[test] fn png_is_deterministic_and_handles_edges() {
        let raster = known();
        assert_eq!(raster.png().unwrap(), raster.png().unwrap());
        let one = Raster::from_pixels(1, 1, vec![0x00abcdef]).unwrap();
        assert_eq!(decoded_rgb(&one.png().unwrap()), ([1, 1], vec![0xab, 0xcd, 0xef]));
        let wide = Raster::from_pixels(5, 1, vec![1, 2, 3, 4, 5]).unwrap();
        assert_eq!(decoded_rgb(&wide.png().unwrap()).0, [5, 1]);
        let tall = Raster::from_pixels(1, 4, vec![9; 4]).unwrap();
        assert_eq!(decoded_rgb(&tall.png().unwrap()).0, [1, 4]);
        for (w, h) in [(0, 0), (0, 3), (3, 0)] {
            assert!(Raster::from_pixels(w, h, Vec::new()).unwrap().png().is_err(), "{w}x{h} must not produce a PNG");
        }
    }
    #[cfg(windows)]
    #[test] fn png_decodes_with_independent_gdiplus() {
        let raster = known();
        let decoded = crate::image_decode::decode(&raster.png().unwrap(), 64).unwrap();
        assert_eq!((decoded.canvas, decoded.size), ([7, 3], [7, 3]));
        let expected: Vec<u32> = raster.pixels().iter().map(|p| 0xff00_0000 | (p & 0x00ff_ffff)).collect();
        assert_eq!(decoded.pixels, expected);
    }
}
