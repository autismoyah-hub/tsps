//! Original 4x4 minimap tile masks and masked, fixed-point sprite sampling.
use crate::{raster::Raster, sprite_pixels::Pixels, Error, Result};
const ROTATIONS: [[usize; 16]; 4] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [12, 8, 4, 0, 13, 9, 5, 1, 14, 10, 6, 2, 15, 11, 7, 3],
    [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
    [3, 7, 11, 15, 2, 6, 10, 14, 1, 5, 9, 13, 0, 4, 8, 12],
];
const SHAPES: [[u8; 16]; 13] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1],
    [1, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0],
    [0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1],
    [0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0],
    [1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0, 1, 1],
    [1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1],
];
/// None pixels preserve the previous layer. Model overlay zero still overwrites.
pub fn tile(shape: u8, rotation: u8, underlay: u32, overlay: u32) -> Result<[Option<u32>; 16]> {
    let shape = SHAPES
        .get(shape as usize)
        .ok_or(Error::Invalid("minimap tile shape"))?;
    let rotation = ROTATIONS
        .get(rotation as usize)
        .ok_or(Error::Invalid("minimap tile rotation"))?;
    Ok(std::array::from_fn(|i| {
        if shape[rotation[i]] != 0 {
            Some(overlay)
        } else {
            (underlay != 0).then_some(underlay)
        }
    }))
}
#[derive(Clone, Debug)]
pub struct Mask {
    pub width: usize,
    pub starts: Vec<i32>,
    pub widths: Vec<i32>,
}
impl Mask {
    /// GameWidget.eq, after sprite normalization to its full canvas.
    pub fn from_sprite(sprite: &Pixels) -> Self {
        let [width, height] = sprite.canvas;
        let mut starts = Vec::with_capacity(height);
        let mut widths = Vec::with_capacity(height);
        for y in 0..height {
            let zero = |x: usize| {
                x < sprite.offset[0]
                    || y < sprite.offset[1]
                    || x >= sprite.offset[0] + sprite.size[0]
                    || y >= sprite.offset[1] + sprite.size[1]
                    || sprite.pixels[(y - sprite.offset[1]) * sprite.size[0] + x - sprite.offset[0]]
                        == 0
            };
            let start = (0..width).find(|&x| zero(x)).unwrap_or(0);
            let end = (start..width)
                .rev()
                .find(|&x| zero(x))
                .map_or(width, |x| x + 1);
            starts.push(start as i32);
            widths.push((end - start) as i32);
        }
        Self {
            width,
            starts,
            widths,
        }
    }
}
/// GameSpritePixels.bi/my. Angle is the revision-239 14-bit camera angle.
pub fn rotated(
    sprite: &Pixels,
    raster: &mut Raster,
    rect: [i32; 4],
    source: [i32; 2],
    angle: u16,
    scale: i32,
    origin: [i32; 2],
    mask: &Mask,
) -> Result<()> {
    if scale == 0 || angle >= 16384 || mask.starts.len() != mask.widths.len() {
        return Err(Error::Invalid("minimap sampling arguments"));
    }
    let theta = angle as f64 * 3.834951969714103E-4;
    let sin = (((theta.sin() * 65536.0) as i32 as i64) << 8) / scale as i64;
    let cos = (((theta.cos() * 65536.0) as i32 as i64) << 8) / scale as i64;
    let sin = sin as i32;
    let cos = cos as i32;
    let [x, y, w, h] = rect;
    let left = -w / 2;
    let top = -h / 2;
    let mut sx = source[0]
        .wrapping_shl(16)
        .wrapping_add(top.wrapping_mul(sin))
        .wrapping_add(left.wrapping_mul(cos));
    let mut sy = source[1]
        .wrapping_shl(16)
        .wrapping_add(top.wrapping_mul(cos))
        .wrapping_sub(left.wrapping_mul(sin));
    let clip = raster.clip();
    for row in 0..h {
        let dy = y + row;
        let m = dy - origin[1];
        if m >= 0 && (m as usize) < mask.starts.len() && mask.starts[m as usize] >= 0 {
            let (mut xx, mut yy) = (sx, sy);
            for col in 0..w {
                let dx = x + col;
                let mx = dx - origin[0];
                if mx >= mask.starts[m as usize]
                    && mx < mask.starts[m as usize] + mask.widths[m as usize]
                    && dx >= clip[0] as i32
                    && dx < clip[2] as i32
                    && dy >= clip[1] as i32
                    && dy < clip[3] as i32
                {
                    let (px, py) = (xx >> 16, yy >> 16);
                    if px >= 0
                        && py >= 0
                        && px < sprite.size[0] as i32
                        && py < sprite.size[1] as i32
                    {
                        let pixel = sprite.pixels[py as usize * sprite.size[0] + px as usize];
                        if pixel != 0 {
                            raster.blend(dx as usize, dy as usize, pixel, 256);
                        }
                    }
                }
                xx = xx.wrapping_add(cos);
                yy = yy.wrapping_sub(sin);
            }
        }
        sx = sx.wrapping_add(sin);
        sy = sy.wrapping_add(cos);
    }
    Ok(())
}

use crate::{
    floor::{self, Floors},
    scene::{Scene, SIZE},
    texture::Definition,
};
use std::collections::HashMap;
pub fn tile_pixels(
    shape: usize,
    rotation: usize,
    underlay: u32,
    overlay: u32,
) -> Result<[u32; 16]> {
    let mask = SHAPES
        .get(shape)
        .ok_or(Error::Invalid("minimap tile shape"))?;
    let rotation = ROTATIONS
        .get(rotation)
        .ok_or(Error::Invalid("minimap tile rotation"))?;
    Ok(std::array::from_fn(|i| {
        if mask[rotation[i]] == 0 {
            underlay
        } else {
            overlay
        }
    }))
}
fn shade(hsl: i32) -> usize {
    ((hsl & 65408) + ((hsl & 127) * 96 / 128).clamp(2, 126)) as usize
}
pub fn terrain(
    scene: &Scene,
    floors: &Floors,
    textures: &HashMap<u16, Definition>,
    plane: usize,
    hue_offset: i32,
    light_offset: i32,
) -> Result<Raster> {
    if plane >= 4 {
        return Err(Error::Invalid("minimap plane"));
    }
    let colors = floor::palette(0.8)?;
    let mut image = vec![0xff000000; SIZE * 4 * SIZE * 4];
    let view_plane = plane;
    for plane in view_plane..=(view_plane + 1).min(3) {
        for x in 1..SIZE - 1 {
            for y in 1..SIZE - 1 {
                let tile = scene.tile(plane, x, y).unwrap();
                if !visible_layer(tile.flags, view_plane, plane) || !scene.loaded(plane, x, y) {
                    continue;
                }
                let mut underlay = 0;
                let id = tile.underlay & 32767;
                if id != 0 {
                    let (mut hue, mut saturation, mut light, mut multiplier, mut count) =
                        (0, 0, 0, 0, 0);
                    // The original rolling window adds +5 and removes -5 before use.
                    for dx in x.saturating_sub(4)..=(x + 5).min(SIZE - 1) {
                        for dy in y.saturating_sub(4)..=(y + 5).min(SIZE - 1) {
                            let id = scene.tile(plane, dx, dy).unwrap().underlay & 32767;
                            if id != 0 {
                                let f = floors
                                    .underlays
                                    .get(&(id - 1))
                                    .ok_or(Error::Invalid("missing floor underlay"))?;
                                hue += f.hue;
                                saturation += f.saturation;
                                light += f.lightness;
                                multiplier += f.hue_multiplier;
                                count += 1;
                            }
                        }
                    }
                    if count != 0 {
                        let h = (hue * 256 / multiplier + hue_offset) & 255;
                        let l = (light / count + light_offset).clamp(0, 255);
                        underlay = colors[shade(floor::pack_hsl(h, saturation / count, l))];
                    }
                }
                let overlay_id = (tile.overlay as u16) & 32767;
                let (shape, overlay) = if overlay_id == 0 {
                    (0, 0)
                } else {
                    let f = floors
                        .overlays
                        .get(&(overlay_id - 1))
                        .ok_or(Error::Invalid("missing floor overlay"))?;
                    let color = if let Some(hsl) = f.secondary {
                        colors[shade(floor::pack_hsl(
                            (hsl.hue + hue_offset) & 255,
                            hsl.saturation,
                            (hsl.lightness + light_offset).clamp(0, 255),
                        ))]
                    } else if let Some(texture) = f.texture {
                        colors[shade(
                            textures
                                .get(&(texture as u16))
                                .ok_or(Error::Invalid("missing floor texture"))?
                                .average_hsl as i32,
                        )]
                    } else if f.rgb == 0xff00ff {
                        0
                    } else {
                        let hsl = f.primary;
                        colors[shade(floor::pack_hsl(
                            (hsl.hue + hue_offset) & 255,
                            hsl.saturation,
                            (hsl.lightness + light_offset).clamp(0, 255),
                        ))]
                    };
                    (tile.overlay_shape as usize + 1, color)
                };
                let pixels = if shape <= 1 {
                    let color = if shape == 0 { underlay } else { overlay };
                    [(color != 0).then_some(color); 16]
                } else {
                    self::tile(shape as u8, tile.overlay_rotation, underlay, overlay)?
                };
                for dy in 0..4 {
                    for dx in 0..4 {
                        if let Some(color) = pixels[dy * 4 + dx] {
                            image[((SIZE - 1 - y) * 4 + dy) * (SIZE * 4) + x * 4 + dx] =
                                0xff000000 | color;
                        }
                    }
                }
            }
        }
    }
    Raster::from_pixels(SIZE * 4, SIZE * 4, image)
}
#[derive(Clone)]
pub struct EntityView {
    pub delta: [i32; 2],
    pub yaw: u16,
    pub sprite: Option<std::sync::Arc<Pixels>>,
    pub markers: Vec<([i32; 2], usize)>,
}
/// Ground-item point before minimap offset/rotation; tile coordinates are view-local.
pub fn ground_position(
    tile: [i32; 2],
    child: bool,
    entity: Option<&crate::world_entity::Entity>,
) -> [i32; 2] {
    // GameClient.pc centres root dots but transforms child tile corners.
    let fine = tile.map(|n| n * 128 + if child { 0 } else { 64 });
    if child {
        entity.map_or(fine, |e| {
            let p = e.transform_horizontal(fine);
            [p[0] as i32, p[2] as i32]
        })
    } else {
        fine
    }
}
/// Original minimap code truncates the scaled difference, not each operand.
pub fn offset(position: [i32; 2], centre: [i32; 2], scale: f32) -> [i32; 2] {
    std::array::from_fn(|i| (position[i].wrapping_sub(centre[i]) as f32 * scale) as i32)
}
#[derive(Clone)]
pub struct View {
    pub steering: Option<([i32; 2], crate::steering::Steering)>,
    pub terrain: std::sync::Arc<Pixels>,
    pub assets: std::sync::Arc<Assets>,
    pub markers: Vec<([i32; 2], usize)>,
    pub players: Vec<([i32; 2], usize)>,
    pub entities: Vec<EntityView>,
    pub icons: Vec<([i32; 2], std::sync::Arc<Pixels>)>,
    pub destination: Option<[i32; 2]>,
    pub hint: Option<[i32; 2]>,
    pub compass_disabled: bool,
    pub source: [i32; 2],
    pub angle: u16,
    pub disabled: bool,
    pub local_visible: bool,
}
impl View {
    pub fn draw(&self, raster: &mut Raster, position: [i32; 2], mask: &Mask) -> Result<()> {
        for (row, (&start, &width)) in mask.starts.iter().zip(&mask.widths).enumerate() {
            if start >= 0 {
                raster.fill_rect(
                    position[0] + start,
                    position[1] + row as i32,
                    width,
                    1,
                    0,
                    256,
                );
            }
        }
        if !self.disabled {
            rotated(
                &self.terrain,
                raster,
                [
                    position[0],
                    position[1],
                    mask.width as i32,
                    mask.starts.len() as i32,
                ],
                self.source,
                self.angle,
                256,
                position,
                mask,
            )?;
            for (delta, sprite) in &self.icons {
                marker(sprite, raster, position, *delta, self.angle, mask)?;
            }
            for &(delta, index) in &self.markers {
                if let Some(sprite) = self.assets.dots.get(index) {
                    marker(sprite, raster, position, delta, self.angle, mask)?;
                }
            }
            for entity in &self.entities {
                if let Some(sprite) = &entity.sprite {
                    entity_marker(
                        sprite,
                        raster,
                        position,
                        entity.delta,
                        self.angle,
                        entity.yaw,
                        0.03125,
                        mask,
                    )?;
                }
                for &(delta, index) in &entity.markers {
                    if let Some(sprite) = self.assets.dots.get(index) {
                        marker(sprite, raster, position, delta, self.angle, mask)?;
                    }
                }
            }
            for &(delta, index) in &self.players {
                if let Some(sprite) = self.assets.dots.get(index) {
                    marker(sprite, raster, position, delta, self.angle, mask)?;
                }
            }
            if let (Some(delta), Some(sprite)) = (self.hint, self.assets.flags.get(1)) {
                hint_marker(
                    sprite,
                    &self.assets.edge,
                    raster,
                    position,
                    delta,
                    self.angle,
                    mask,
                )?;
            }
            if let (Some(delta), Some(sprite)) = (self.destination, self.assets.flags.first()) {
                marker(sprite, raster, position, delta, self.angle, mask)?;
            }
            if let Some((pointer, state)) = self.steering {
                if self.assets.flags.len() >= 4 {
                    let centre = [
                        position[0] + mask.width as i32 / 2,
                        position[1] + mask.starts.len() as i32 / 2,
                    ];
                    if mask.contains([pointer[0] - position[0], pointer[1] - position[1]]) {
                        let heading = (((pointer[1] - centre[1]) as f64)
                            .atan2((pointer[0] - centre[0]) as f64)
                            / 0.0030679615
                            + 0.5)
                            .floor() as i32
                            - 512;
                        let relative = (heading - (self.angle as i32 >> 3)) & 2047;
                        let angle = ((relative + 64) / 128 * 128) as u16;
                        crate::steering::draw_arrow(
                            &self.assets.flags[2],
                            raster,
                            centre,
                            self.angle,
                            angle,
                        );
                    }
                    if state.remaining > 0 {
                        crate::steering::draw_arrow(
                            &self.assets.flags[3],
                            raster,
                            centre,
                            self.angle,
                            state.direction as u16 * 128,
                        );
                    }
                }
            }
            if self.local_visible {
                raster.fill_rect(
                    position[0] + mask.width as i32 / 2 - 1,
                    position[1] + mask.starts.len() as i32 / 2 - 1,
                    3,
                    3,
                    0xffffff,
                    256,
                );
            }
        }
        Ok(())
    }
}
/// Cache 17:3 (zk): graphics default sprite groups.
pub fn sprite_groups(bytes: &[u8]) -> Result<[Option<u32>; 11]> {
    Ok(graphics_defaults(bytes)?.0)
}
/// Cache 17:3 opcode 5: hover and active steering model IDs.
pub fn steering_models(bytes: &[u8]) -> Result<[Option<u32>; 2]> {
    Ok(graphics_defaults(bytes)?.1)
}
fn graphics_defaults(bytes: &[u8]) -> Result<([Option<u32>; 11], [Option<u32>; 2])> {
    let mut r = crate::buffer::Reader::new(bytes);
    let mut ids = [None; 11];
    let mut models = [None; 2];
    loop {
        match r.u8()? {
            0 => break,
            1 => {
                r.u24()?;
            }
            2 => {
                for id in &mut ids {
                    let head = r.u8()?;
                    *id = if head & 128 != 0 {
                        Some(((head as u32 & 127) << 24) | r.u24()?)
                    } else {
                        let value = ((head as u32) << 8) | r.u8()? as u32;
                        (value != 32767).then_some(value)
                    };
                }
            }
            3 => {
                r.bytes(45)?;
            }
            5 => {
                for id in &mut models { let value = r.u32()?; *id = (value != u32::MAX).then_some(value); }
            }
            _ => {}
        }
    }
    if r.remaining() != 0 {
        return Err(Error::Invalid("graphics defaults trailing bytes"));
    }
    Ok((ids, models))
}
pub struct Assets {
    pub compass: Pixels,
    pub dots: Vec<Pixels>,
    pub scenes: Vec<Pixels>,
    pub flags: Vec<Pixels>,
    pub edge: Pixels,
}
impl Assets {
    pub fn load(cache: &mut crate::disk_cache::DiskCache) -> Result<Self> {
        let ids = sprite_groups(&cache.file(17, 3, 0, [0; 4])?)?;
        let compass = Pixels::load(
            cache,
            ids[0].ok_or(Error::Invalid("compass sprite missing"))?,
        )?;
        let sheet = crate::sprite::SpriteSheet::decode(
            &cache.file(
                8,
                ids[8].ok_or(Error::Invalid("map dots missing"))?,
                0,
                [0; 4],
            )?,
            16 << 20,
        )?;
        let dots = (0..sheet.sprites.len())
            .map(|i| Pixels::from_sheet(&sheet, i))
            .collect::<Result<Vec<_>>>()?;
        let sheet = crate::sprite::SpriteSheet::decode(
            &cache.file(
                8,
                ids[2].ok_or(Error::Invalid("map scenes missing"))?,
                0,
                [0; 4],
            )?,
            16 << 20,
        )?;
        let scenes = (0..sheet.sprites.len())
            .map(|i| Pixels::from_sheet(&sheet, i))
            .collect::<Result<Vec<_>>>()?;
        let sheet = crate::sprite::SpriteSheet::decode(
            &cache.file(
                8,
                ids[6].ok_or(Error::Invalid("map markers missing"))?,
                0,
                [0; 4],
            )?,
            16 << 20,
        )?;
        let flags = (0..sheet.sprites.len())
            .map(|i| Pixels::from_sheet(&sheet, i))
            .collect::<Result<Vec<_>>>()?;
        let edge = Pixels::load(
            cache,
            ids[1].ok_or(Error::Invalid("map edge arrow missing"))?,
        )?;
        Ok(Self {
            compass,
            dots,
            scenes,
            flags,
            edge,
        })
    }
}
/// bu.ap: nearby dots are unmasked; distant dots are constrained to the widget mask.
pub fn marker(
    sprite: &Pixels,
    raster: &mut Raster,
    position: [i32; 2],
    delta: [i32; 2],
    angle: u16,
    mask: &Mask,
) -> Result<()> {
    let distance = delta[0]
        .wrapping_mul(delta[0])
        .wrapping_add(delta[1].wrapping_mul(delta[1]));
    if distance > 6400 {
        return Ok(());
    }
    let theta = angle as f64 * 3.834951969714103E-4;
    let sin = (theta.sin() * 65536.0) as i32;
    let cos = (theta.cos() * 65536.0) as i32;
    let dx = (delta[1]
        .wrapping_mul(sin)
        .wrapping_add(delta[0].wrapping_mul(cos)))
        >> 16;
    let dy = (delta[1]
        .wrapping_mul(cos)
        .wrapping_sub(delta[0].wrapping_mul(sin)))
        >> 16;
    let x = dx + mask.width as i32 / 2 - sprite.canvas[0] as i32 / 2;
    let y = mask.starts.len() as i32 / 2 - dy - sprite.canvas[1] as i32 / 2;
    if distance <= 2500 {
        return sprite.draw(raster, position[0] + x, position[1] + y, 256);
    }
    let clip = raster.clip();
    for row in 0..sprite.size[1] {
        let my = y + row as i32;
        if my < 0 || my >= mask.starts.len() as i32 {
            continue;
        }
        for col in 0..sprite.size[0] {
            let mx = x + col as i32;
            let dx = position[0] + mx;
            let dy = position[1] + my;
            if mx < mask.starts[my as usize]
                || mx >= mask.starts[my as usize] + mask.widths[my as usize]
                || dx < clip[0] as i32
                || dx >= clip[2] as i32
                || dy < clip[1] as i32
                || dy >= clip[3] as i32
            {
                continue;
            }
            let pixel = sprite.pixels[row * sprite.size[0] + col];
            if pixel != 0 {
                raster.blend(dx as usize, dy as usize, pixel, 256);
            }
        }
    }
    Ok(())
}
pub fn visible_layer(flags: u8, view_plane: usize, plane: usize) -> bool {
    if plane == view_plane {
        flags & 24 == 0
    } else {
        view_plane < 3 && plane == view_plane + 1 && flags & 8 != 0
    }
}
/// mu.ao's small wall/corner/diagonal marks; sprite-backed objects use map-scene art.
pub fn object_pixels(shape: u8, rotation: u8) -> Result<[bool; 16]> {
    if rotation > 3 {
        return Err(Error::Invalid("minimap object rotation"));
    }
    let mut result = [false; 16];
    let edge = |result: &mut [bool; 16], r: u8| {
        for i in 0..4 {
            result[match r {
                0 => i * 4,
                1 => i,
                2 => i * 4 + 3,
                _ => 12 + i,
            }] = true;
        }
    };
    if matches!(shape, 0 | 2) {
        edge(&mut result, rotation);
    }
    if shape == 2 {
        edge(&mut result, (rotation + 1) & 3);
    }
    if shape == 3 {
        result[[0, 3, 15, 12][rotation as usize]] = true;
    }
    if shape == 9 {
        for i in 0..4 {
            result[if rotation & 1 == 0 {
                (3 - i) * 4 + i
            } else {
                i * 5
            }] = true;
        }
    }
    Ok(result)
}
pub fn objects(
    scene: &Scene,
    defs: &HashMap<u32, crate::object::Definition>,
    sprites: &[Pixels],
    view_plane: usize,
    raster: &mut Raster,
) -> Result<()> {
    let mut objects = scene
        .objects
        .iter()
        .filter(|o| {
            !o.rejected && (o.shape <= 3 || o.shape == 9 || o.shape == 10 || o.shape == 11 || o.shape >= 12)
                && o.x > 0
                && o.y > 0
                && (o.x as usize) < SIZE - 1
                && (o.y as usize) < SIZE - 1
                && scene
                    .tile(o.plane as usize, o.x as usize, o.y as usize)
                    .is_some_and(|t| visible_layer(t.flags, view_plane, o.plane as usize))
        })
        .collect::<Vec<_>>();
    objects.sort_by_key(|o| {
        (
            o.y,
            o.x,
            o.plane,
            if o.shape <= 3 {
                0
            } else if o.shape == 22 {
                2
            } else {
                1
            },
        )
    });
    for o in objects {
        let def = defs
            .get(&o.id)
            .ok_or(Error::Invalid("minimap object definition"))?;
        if let Some(id) = def.map_scene {
            if let Some(sprite) = sprites.get(id as usize) {
                let x = o.x as i32 * 4 + (def.size[0] as i32 * 4 - sprite.size[0] as i32) / 2;
                let y = (SIZE as i32 - o.y as i32 - def.size[1] as i32) * 4
                    + (def.size[1] as i32 * 4 - sprite.size[1] as i32) / 2;
                sprite.draw(raster, x, y, 256)?;
            }
        } else {
            // The client jitters these colors per rebuild; use the center of its range.
            let color = if o.shape == 9 {
                if def.interactive != 0 {
                    15597568
                } else {
                    15658734
                }
            } else if def.interactive != 0 {
                0xee0000
            } else {
                0xeeeeee
            };
            for (i, filled) in object_pixels(o.shape, o.rotation)?.into_iter().enumerate() {
                if filled {
                    raster.fill_rect(
                        o.x as i32 * 4 + (i % 4) as i32,
                        (SIZE as i32 - 1 - o.y as i32) * 4 + (i / 4) as i32,
                        1,
                        1,
                        color,
                        256,
                    );
                }
            }
        }
    }
    Ok(())
}

pub fn icons(
    scene: &Scene,
    defs: &HashMap<u32, crate::object::Definition>,
    cache: &mut crate::disk_cache::DiskCache,
    plane: usize,
) -> Result<Vec<([i32; 2], std::sync::Arc<Pixels>)>> {
    let elements = crate::map_element::Definition::load_all(cache)?;
    let mut sprites = HashMap::new();
    let mut out = Vec::new();
    for obj in &scene.objects {
        if obj.rejected || obj.shape != 22 || obj.plane as usize != plane {
            continue;
        }
        let Some(element) = defs
            .get(&obj.id)
            .and_then(|d| d.map_icon)
            .and_then(|id| elements.get(&(id as u32)))
        else {
            continue;
        };
        if !element.minimap {
            continue;
        }
        let Some(id) = element.sprites[0] else {
            continue;
        };
        if !sprites.contains_key(&id) {
            sprites.insert(id, std::sync::Arc::new(Pixels::load(cache, id)?));
        }
        out.push((
            [scene.base_x + obj.x as i32, scene.base_y + obj.y as i32],
            sprites[&id].clone(),
        ));
    }
    out.sort_by_key(|(position, _)| *position);
    Ok(out)
}

impl Mask {
    /// kw.iv includes the right endpoint, unlike the raster clip.
    pub fn contains(&self, point: [i32; 2]) -> bool {
        let [x, y] = point;
        y >= 0
            && self
                .starts
                .get(y as usize)
                .zip(self.widths.get(y as usize))
                .is_some_and(|(&start, &width)| x >= start && x <= start.wrapping_add(width))
    }
}
/// GameWorldMapIcon.av root-world click calculation. Returns local tile and centered click.
pub fn click_target(
    mask: &Mask,
    point: [i32; 2],
    local_fine: [i32; 2],
    angle: u16,
    zoom: f32,
) -> Result<Option<([i32; 2], [i32; 2])>> {
    if angle >= 16384 || !zoom.is_finite() || zoom <= 0.0 {
        return Err(Error::Invalid("minimap click arguments"));
    }
    if !mask.contains(point) {
        return Ok(None);
    }
    let dx = point[0] - mask.width as i32 / 2;
    let dy = point[1] - mask.starts.len() as i32 / 2;
    let theta = angle as f64 * 3.834951969714103E-4;
    let sin = (theta.sin() * 65536.0) as i32;
    let cos = (theta.cos() * 65536.0) as i32;
    let rx = cos.wrapping_mul(dx).wrapping_add(sin.wrapping_mul(dy)) >> 16;
    let ry = cos.wrapping_mul(dy).wrapping_sub(sin.wrapping_mul(dx)) >> 16;
    let scale = zoom / 128.0;
    let offset_x = (rx as f32 / scale) as i32;
    let offset_y = (ry as f32 / scale) as i32;
    Ok(Some((
        [
            local_fine[0].wrapping_add(offset_x) >> 7,
            local_fine[1].wrapping_sub(offset_y) >> 7,
        ],
        [dx, dy],
    )))
}
/// Select the alternate mask for clicking, with the original missing-sprite fallback.
pub fn widget_mask(
    cache: &mut crate::disk_cache::DiskCache,
    widget: &crate::widget::Widget,
    alternate: bool,
) -> Result<Option<Mask>> {
    let id = if alternate && widget.sprites[1] != -1 {
        widget.sprites[1]
    } else {
        widget.sprites[0]
    };
    if id == -1 {
        return Ok(None);
    }
    // GameWidget.eq returns no mask when aq/si.af cannot yet load the sprite.
    // The click is ignored; a later input attempt can use the recovered asset.
    let mut sprite = match Pixels::load(cache, id as u32) {
        Ok(sprite) => sprite,
        Err(Error::PendingAsset { .. }) => return Ok(None),
        Err(error) => return Err(error),
    };
    sprite.widget_effects(
        widget.flip_horizontal,
        widget.flip_vertical,
        widget.outline,
        widget.sprite_shadow,
    );
    Ok(Some(Mask::from_sprite(&sprite)))
}

/// vs.aq: nearby hint markers and rotating arrows for targets 65..300 pixels away.
pub fn hint_marker(
    sprite: &Pixels,
    edge: &Pixels,
    raster: &mut Raster,
    position: [i32; 2],
    delta: [i32; 2],
    angle: u16,
    mask: &Mask,
) -> Result<()> {
    let distance = delta[0]
        .wrapping_mul(delta[0])
        .wrapping_add(delta[1].wrapping_mul(delta[1]));
    if distance <= 4225 || distance >= 90000 {
        return marker(sprite, raster, position, delta, angle, mask);
    }
    let theta = angle as f64 * 3.834951969714103E-4;
    let sin = (theta.sin() * 65536.0) as i32;
    let cos = (theta.cos() * 65536.0) as i32;
    let dx = cos
        .wrapping_mul(delta[0])
        .wrapping_add(sin.wrapping_mul(delta[1]))
        >> 16;
    let dy = cos
        .wrapping_mul(delta[1])
        .wrapping_sub(sin.wrapping_mul(delta[0]))
        >> 16;
    let heading = (dx as f64).atan2(dy as f64);
    let radius = mask.width as i32 / 2 - 25;
    let x = position[0] + mask.width as i32 / 2 - 10 + (heading.sin() * radius as f64) as i32;
    let y =
        position[1] + mask.starts.len() as i32 / 2 - 20 - (heading.cos() * radius as f64) as i32;
    // GameSpritePixels.il includes the injected odd-framebuffer origin correction.
    let mut source = [15, 15];
    if heading < std::f64::consts::FRAC_PI_2 || heading > std::f64::consts::PI * 1.5 {
        source[0] += (raster.width() % 2) as i32;
        source[1] += (raster.height() % 2) as i32;
    }
    let sin = (heading.sin() * 65536.0) as i32;
    let cos = (heading.cos() * 65536.0) as i32;
    let mut sx = (source[0] << 16) - 10 * sin - 10 * cos;
    let mut sy = (source[1] << 16) - 10 * cos + 10 * sin;
    let clip = raster.clip();
    for row in 0..20 {
        let (mut u, mut v) = (sx, sy);
        for col in 0..20 {
            let (px, py) = (u >> 16, v >> 16);
            let (dx, dy) = (x + col, y + row);
            if px >= 0
                && py >= 0
                && (px as usize) < edge.size[0]
                && (py as usize) < edge.size[1]
                && dx >= clip[0] as i32
                && dy >= clip[1] as i32
                && dx < clip[2] as i32
                && dy < clip[3] as i32
            {
                let color = edge.pixels[py as usize * edge.size[0] + px as usize];
                if color != 0 {
                    raster.blend(dx as usize, dy as usize, color, 256);
                }
            }
            u += cos;
            v -= sin;
        }
        sx += sin;
        sy += cos;
    }
    Ok(())
}

/// GameClient.xg: a world-entity sprite rotates with the camera and entity yaw.
pub fn entity_marker(
    sprite: &Pixels,
    raster: &mut Raster,
    position: [i32; 2],
    delta: [i32; 2],
    angle: u16,
    yaw: u16,
    scale: f32,
    mask: &Mask,
) -> Result<()> {
    let distance = delta[0]
        .wrapping_mul(delta[0])
        .wrapping_add(delta[1].wrapping_mul(delta[1]));
    if distance > 6400 {
        return Ok(());
    }
    let zoom = (scale * 8192.0) as i32;
    if zoom <= 0 {
        return Err(Error::Invalid("world entity minimap scale"));
    }
    let theta = angle as f64 * 3.834951969714103E-4;
    let sin = (theta.sin() * 65536.0) as i32;
    let cos = (theta.cos() * 65536.0) as i32;
    let dx = delta[1]
        .wrapping_mul(sin)
        .wrapping_add(delta[0].wrapping_mul(cos))
        >> 16;
    let dy = delta[1]
        .wrapping_mul(cos)
        .wrapping_sub(delta[0].wrapping_mul(sin))
        >> 16;
    let angle = (angle as i32 - (16383 - ((yaw as i32) << 3) + 8192)) & 16383;
    let width = (sprite.canvas[0] as i32).wrapping_mul(zoom) >> 8;
    let height = (sprite.canvas[1] as i32).wrapping_mul(zoom) >> 8;
    let x = position[0] + mask.width as i32 / 2 + dx - width / 2;
    let y = position[1] + mask.starts.len() as i32 / 2 - dy - height / 2;
    rotated(
        sprite,
        raster,
        [x, y, width, height],
        [sprite.size[0] as i32 / 2, sprite.size[1] as i32 / 2],
        angle as u16,
        zoom,
        position,
        mask,
    )
}

/// GameMapElementConfig.cx permits an unavailable sprite. The native async
/// cache retries the complete icon snapshot without aborting live gameplay.
pub(crate) fn commit_icons(
    current: &mut Vec<([i32; 2], std::sync::Arc<Pixels>)>,
    next: Result<Vec<([i32; 2], std::sync::Arc<Pixels>)>>,
) -> Result<bool> {
    match next {
        Ok(icons) => { *current = icons; Ok(true) }
        Err(Error::PendingAsset { .. }) => Ok(false),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod icon_retry_tests {
    use super::*;
    fn icon(at: i32) -> Vec<([i32; 2], std::sync::Arc<Pixels>)> {
        vec![([at, 0], std::sync::Arc::new(Pixels {
            canvas: [1, 1], offset: [0, 0], size: [1, 1], pixels: vec![0xff123456],
        }))]
    }
    #[test]
    fn pending_icons_retry_without_losing_the_completed_snapshot() {
        let mut current = icon(1);
        assert!(!commit_icons(&mut current, Err(Error::PendingAsset { archive: 8, group: 1492 })).unwrap());
        assert_eq!(current[0].0, [1, 0]);
        assert!(commit_icons(&mut current, Ok(icon(2))).unwrap());
        assert_eq!(current.len(), 1);
        assert_eq!(current[0].0, [2, 0]);
        assert!(commit_icons(&mut current, Err(Error::Invalid("malformed sprite"))).is_err());
        assert_eq!(current[0].0, [2, 0]);
        assert!(commit_icons(&mut current, Ok(Vec::new())).unwrap());
        assert!(current.is_empty());
    }
}
