//! World-map wall strokes from GameWorldMapRegion.ah/be.
use crate::raster::Raster;
pub fn wall(
    raster: &mut Raster,
    [x, y]: [i32; 2],
    size: i32,
    shape: u8,
    rotation: u8,
    interactive: bool,
) {
    if size <= 0 {
        return;
    }
    let rotation = rotation & 3;
    let color = if interactive { 0xffcc0000 } else { 0xffcccccc };
    let edge = |raster: &mut Raster, rotation: u8, color: u32| match rotation & 3 {
        0 => raster.fill_rect(x, y, 1, size, color, 256),
        1 => raster.fill_rect(x, y, size, 1, color, 256),
        2 => raster.fill_rect(x + size - 1, y, 1, size, color, 256),
        _ => raster.fill_rect(x, y + size - 1, size, 1, color, 256),
    };
    match shape {
        0 => edge(raster, rotation, color),
        2 => {
            edge(raster, rotation, 0xffcccccc);
            edge(raster, rotation + 1, color);
        }
        3 => {
            let (dx, dy) = match rotation {
                0 => (0, 0),
                1 => (size - 1, 0),
                2 => (size - 1, size - 1),
                _ => (0, size - 1),
            };
            raster.fill_rect(x + dx, y + dy, 1, 1, color, 256);
        }
        9 => {
            for i in 0..size {
                raster.fill_rect(
                    x + i,
                    y + if rotation & 1 == 0 { size - 1 - i } else { i },
                    1,
                    1,
                    color,
                    256,
                );
            }
        }
        _ => {}
    }
}

pub fn scene(
    raster: &mut Raster,
    [x, y]: [i32; 2],
    size: i32,
    shape: u8,
    rotation: u8,
    object_size: [u8; 2],
    sprite: Option<&crate::sprite_pixels::Pixels>,
) -> crate::Result<()> {
    if !matches!(shape, 10 | 11 | 22) || size <= 0 {
        return Ok(());
    }
    if let Some(sprite) = sprite {
        let height = object_size[if rotation & 1 != 0 { 0 } else { 1 }] as i32;
        sprite.draw_scaled(
            raster,
            [x * size, (64 - height - y) * size, size * 2, size * 2],
            256,
        )?;
    }
    Ok(())
}

pub fn region(
    raster: &mut Raster,
    geography: &crate::map_geography::Geography,
    size: i32,
    definitions: &std::collections::HashMap<u32, crate::object::Definition>,
    sprites: &[crate::sprite_pixels::Pixels],
) -> crate::Result<()> {
    if geography.tiles.len() != 4096 || !(1..=64).contains(&size) {
        return Err(crate::Error::Invalid("map decoration dimensions"));
    }
    for x in 0..64 {
        for y in 0..64 {
            let tile = &geography.tiles[x * 64 + y];
            for decoration in tile
                .layers
                .iter()
                .flat_map(|l| &l.decorations)
                .filter(|d| matches!(d.shape, 0..=3 | 9))
            {
                let def = definitions
                    .get(&(decoration.id as u32))
                    .ok_or(crate::Error::Invalid("map wall definition"))?;
                wall(
                    raster,
                    [x as i32 * size, (63 - y) as i32 * size],
                    size,
                    decoration.shape,
                    decoration.rotation,
                    def.interactive != 0,
                );
            }
            for decoration in tile
                .layers
                .iter()
                .flat_map(|l| &l.decorations)
                .filter(|d| matches!(d.shape, 10 | 11 | 22))
            {
                let def = definitions
                    .get(&(decoration.id as u32))
                    .ok_or(crate::Error::Invalid("map scene definition"))?;
                let sprite = def
                    .map_scene
                    .map(|id| {
                        sprites
                            .get(id as usize)
                            .ok_or(crate::Error::Invalid("map scene sprite"))
                    })
                    .transpose()?;
                scene(
                    raster,
                    [x as i32, y as i32],
                    size,
                    decoration.shape,
                    decoration.rotation,
                    def.size,
                    sprite,
                )?;
            }
        }
    }
    Ok(())
}
