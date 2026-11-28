//! GameClient's screen-space mouse cross (mk/mh/me/md), independent of map flags.
use crate::{disk_cache::DiskCache, raster::Raster, sprite::SpriteSheet, sprite_pixels::Pixels, Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind { Walk, Action }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cross {
    pub position: [i32; 2],
    kind: Option<Kind>,
    elapsed: u16,
}
impl Cross {
    pub fn show(&mut self, kind: Kind, position: [i32; 2]) {
        self.position = position;
        self.kind = Some(kind);
        self.elapsed = 0;
    }
    /// Original game tick: decoded md += 20, me = 0 at md >= 400.
    pub fn tick(&mut self) {
        if self.kind.is_some() {
            self.elapsed += 20;
            if self.elapsed >= 400 { self.kind = None; }
        }
    }
    pub fn frame(&self) -> Option<usize> {
        self.kind.map(|kind| usize::from(self.elapsed / 100)
            + if kind == Kind::Action { 4 } else { 0 })
    }
}

/// Authentic graphics-defaults cross sprites; shared by every world renderer.
#[derive(Default)]
pub struct Sprites(Option<Vec<Pixels>>);
impl Sprites {
    /// GameIntProjection.ab loads bs.mn before gameplay can accept a click.
    pub fn load(&mut self, cache: &mut DiskCache) -> Result<()> {
        if self.0.is_none() {
            let groups = crate::minimap::sprite_groups(&cache.file(17, 3, 0, [0; 4])?)?;
            let group = groups[7].ok_or(Error::Invalid("click cross sprites missing"))?;
            let sheet = SpriteSheet::decode(&cache.file(8, group, 0, [0; 4])?, 16 << 20)?;
            self.0 = Some((0..8).map(|i| Pixels::from_sheet(&sheet, i)).collect::<Result<_>>()?);
        }
        Ok(())
    }
    pub fn draw(&mut self, cache: &mut DiskCache, raster: &mut Raster, cross: Cross, enabled: bool) -> Result<()> {
        if !enabled { return Ok(()); }
        let Some(frame) = cross.frame() else { return Ok(()); };
        self.load(cache)?;
        self.0.as_ref().unwrap()[frame].draw(raster,
            cross.position[0].wrapping_sub(8), cross.position[1].wrapping_sub(8), 256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_encoded_tick_and_four_frame_lifetime() {
        assert_eq!((-85412044i32).wrapping_mul(-12016871), 20);
        for (kind, base) in [(Kind::Walk, 0), (Kind::Action, 4)] {
            let mut cross = Cross::default();
            assert_eq!(cross.frame(), None);
            cross.show(kind, [319, 241]);
            for tick in 0..20 {
                assert_eq!(cross.frame(), Some(base + tick / 5));
                assert_eq!(cross.position, [319, 241]);
                cross.tick();
            }
            assert_eq!(cross.frame(), None);
            for _ in 0..100_000 { cross.tick(); }
            assert_eq!(cross.frame(), None);
        }
    }
    #[test]
    fn repeated_click_replaces_color_position_and_restarts_animation() {
        let mut cross = Cross::default();
        cross.show(Kind::Action, [400, 300]);
        for _ in 0..19 { cross.tick(); }
        cross.show(Kind::Walk, [1, 2]);
        assert_eq!(cross.frame(), Some(0));
        assert_eq!(cross.position, [1, 2]);
        cross.tick();
        cross.show(Kind::Action, [700, 490]);
        assert_eq!(cross.frame(), Some(4));
        assert_eq!(cross.position, [700, 490]);
    }
}
