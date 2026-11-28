//! Rank/moderator images shared by original GameFontTypeFace text paths.
use crate::{disk_cache::DiskCache, sprite::SpriteSheet, Result};
pub fn decode(bytes: &[u8]) -> Result<Vec<SpriteSheet>> {
    let sheet = SpriteSheet::decode(bytes, 16 << 20)?;
    let SpriteSheet {
        width,
        height,
        palette,
        sprites,
    } = sheet;
    Ok(sprites
        .into_iter()
        .map(|sprite| SpriteSheet {
            width,
            height,
            palette: palette.clone(),
            sprites: vec![sprite],
        })
        .collect())
}
pub fn load(cache: &mut DiskCache) -> Result<Vec<SpriteSheet>> {
    let groups = crate::minimap::sprite_groups(&cache.file(17, 3, 0, [0; 4])?)?;
    let Some(group) = groups[10] else {
        return Ok(Vec::new());
    };
    decode(&cache.file(8, group, 0, [0; 4])?)
}
