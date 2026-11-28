//! Cached definition resources and floor-only world-map region rasterization.
use crate::{
    disk_cache::DiskCache,
    floor::{self, Overlay},
    map_color,
    map_geography::Geography,
    map_mask::Masks,
    map_tile,
    sprite_pixels::Pixels,
    texture, Error, Result,
};
use std::collections::HashMap;
pub struct Floors {
    pub overlays: HashMap<u16, Overlay>,
    pub textures: HashMap<u16, texture::Definition>,
    pub palette: Vec<u32>,
}
impl Floors {
    pub fn load(cache: &mut DiskCache, brightness: f64) -> Result<Self> {
        let overlays = cache
            .group(2, 4, [0; 4])?
            .into_iter()
            .map(|(id, bytes)| {
                Ok((
                    u16::try_from(id).map_err(|_| Error::Invalid("map overlay id"))?,
                    Overlay::decode(&bytes)?,
                ))
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            overlays,
            textures: texture::Definition::load_all(cache)?,
            palette: floor::palette(brightness)?,
        })
    }
    pub fn render(
        &self,
        geography: &Geography,
        ground: &[u32],
        background: u32,
        size: usize,
    ) -> Result<Pixels> {
        if geography.tiles.len() != 4096 || ground.len() != 4096 {
            return Err(Error::Invalid("map floor region dimensions"));
        }
        let masks = Masks::new(size)?;
        let side = 64 * size;
        let mut pixels = vec![0; side * side];
        let mut tile_pixels = vec![0; size * size];
        let mut colors = Vec::with_capacity(4);
        for x in 0..64 {
            for y in 0..64 {
                let tile = &geography.tiles[x * 64 + y];
                if tile.layers.is_empty() {
                    continue;
                }
                colors.clear();
                for layer in &tile.layers {
                    let id = (layer.overlay as i16 as i32) - 1;
                    let def = u16::try_from(id).ok().and_then(|id| self.overlays.get(&id));
                    let average = def
                        .and_then(|v| v.texture)
                        .and_then(|id| self.textures.get(&(id as u16)))
                        .map_or(0, |t| t.average_hsl as i32);
                    colors.push(map_color::overlay_color(
                        def,
                        background | 0xff000000,
                        average,
                        &self.palette,
                    )?);
                }
                tile_pixels.fill(0);
                map_tile::paint(
                    &mut tile_pixels,
                    tile,
                    &masks,
                    background | 0xff000000,
                    ground[y * 64 + x],
                    &colors,
                )?;
                for row in 0..size {
                    let index = ((63 - y) * size + row) * side + x * size;
                    pixels[index..index + size]
                        .copy_from_slice(&tile_pixels[row * size..(row + 1) * size]);
                }
            }
        }
        Ok(Pixels {
            canvas: [side; 2],
            offset: [0; 2],
            size: [side; 2],
            pixels,
        })
    }
}
