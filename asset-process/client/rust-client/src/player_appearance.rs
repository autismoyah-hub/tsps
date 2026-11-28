//! Revision 239 player appearance block (GamePlayer.af).
use crate::{
    buffer::Reader, item::Definitions, item_model::AppearanceOverride, player_colors,
    player_model::Appearance, Error, Result,
};
#[derive(Clone, Debug)]
pub struct Update {
    pub appearance: Appearance,
    pub skull: i8,
    pub overhead: i8,
    pub animations: [i32; 7],
    pub name: String,
    pub combat: u8,
    pub skill: u16,
    pub hidden: bool,
    pub force_refresh: bool,
    pub actions: [String; 3],
    pub team: i32,
}
fn equipment(r: &mut Reader<'_>) -> Result<u32> {
    let high = r.u8()?;
    Ok(if high == 0 {
        0
    } else {
        (high as u32) << 8 | r.u8()? as u32
    })
}
fn model(r: &mut Reader<'_>) -> Result<Option<u32>> {
    let id = r.u32()?;
    Ok((id != u32::MAX).then_some(id))
}
pub fn decode(bytes: &[u8], items: &Definitions) -> Result<Update> {
    let mut r = Reader::new(bytes);
    let gender = r.u8()?;
    let skull = r.u8()? as i8;
    let overhead = r.u8()? as i8;
    let mut appearance = Appearance {
        gender,
        ..Default::default()
    };
    let mut team = 0;
    for slot in 0..12 {
        let id = equipment(&mut r)?;
        if slot == 0 && id == 65535 {
            appearance.npc = Some(r.u16()?);
            break;
        }
        appearance.equipment[slot] = id;
        if id >= 2048 {
            if let Some(item) = items.items.get(&(id - 2048)) {
                if item.team != 0 {
                    team = item.team;
                }
            }
        }
    }
    for id in &mut appearance.base_equipment {
        *id = equipment(&mut r)?;
    }
    for (i, color) in appearance.colors.iter_mut().enumerate() {
        let n = r.u8()?;
        *color = if (n as usize) < player_colors::COLORS[i].len() {
            n
        } else {
            0
        };
    }
    let mut animations = [-1; 7];
    for v in &mut animations {
        let id = r.u16()?;
        if id != 65535 {
            *v = id as i32;
        }
    }
    let name = r.string()?;
    let combat = r.u8()?;
    let skill = r.u16()?;
    let hidden = r.u8()? == 1;
    let mask = r.u16()?;
    let force_refresh = mask & 32768 != 0;
    for slot in 0..12 {
        if mask & (1 << (12 - slot)) != 0 {
            let id = appearance.equipment[slot].wrapping_sub(2048);
            let item = items.items.get(&id);
            let mut value = AppearanceOverride {
                recolor: item
                    .filter(|d| !d.recolor.is_empty())
                    .map(|d| d.recolor.iter().map(|v| v.1).collect()),
                retexture: item
                    .filter(|d| !d.retexture.is_empty())
                    .map(|d| d.retexture.iter().map(|v| v.1).collect()),
                ..Default::default()
            };
            let flags = r.u8()?;
            for (flag, values) in [(1, &mut value.recolor), (2, &mut value.retexture)] {
                if flags & flag != 0 {
                    let indices = r.u8()?;
                    for index in [indices & 15, indices >> 4] {
                        if index != 15 {
                            let color = r.u16()?;
                            if let Some(values) = values.as_mut().filter(|v| v.len() == 2) {
                                *values
                                    .get_mut(index as usize)
                                    .ok_or(Error::Invalid("appearance override index"))? = color;
                            }
                        }
                    }
                }
            }
            if flags & 4 != 0 {
                value.wear_model = [model(&mut r)?, model(&mut r)?];
            }
            if flags & 8 != 0 {
                value.head_model = [model(&mut r)?, model(&mut r)?];
            }
            appearance.overrides[slot] = Some(value);
        }
    }
    let actions = [r.string()?, r.string()?, r.string()?];
    appearance.extra = r.u8()? as i8;
    if r.remaining() != 0 {
        return Err(Error::Invalid("player appearance trailing bytes"));
    }
    Ok(Update {
        appearance,
        skull,
        overhead,
        animations,
        name,
        combat,
        skill,
        hidden,
        force_refresh,
        actions,
        team,
    })
}
