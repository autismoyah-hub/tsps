//! Legacy widget expressions and conditions (cp.aq and ga.ao).
use crate::{
    inventory::Inventories, item::Definitions, script_data::Data, script_runtime::Variables,
    widget::Widget, Error, Result,
};
pub struct Context<'a> {
    pub data: &'a Data,
    pub variables: &'a Variables,
    pub inventories: &'a Inventories,
    pub items: &'a Definitions,
    pub combat: i32,
}
impl Context<'_> {
    pub fn evaluate(
        &self,
        widget: &Widget,
        index: usize,
        mut varbit: impl FnMut(i32) -> Result<i32>,
    ) -> i32 {
        let Some(code) = widget.cs1.get(index) else {
            return -2;
        };
        let mut run = || -> Result<i32> {
            let mut code = code.iter();
            let mut next = || code.next().copied().ok_or(Error::Truncated);
            let mut total = 0i32;
            let mut pending = 0;
            loop {
                let op = next()?;
                let mut value = 0;
                match op {
                    0 => return Ok(total),
                    1..=3 => {
                        let i = next()? as usize;
                        value = *match op {
                            1 => self.data.boosted.get(i),
                            2 => self.data.base.get(i),
                            _ => self.data.experience.get(i),
                        }
                        .ok_or(Error::Invalid("CS1 skill"))?;
                    }
                    4 | 10 => {
                        let widget = next()?.wrapping_shl(16).wrapping_add(next()?) as u32;
                        let item = next()?;
                        if item != -1
                            && (self.data.members
                                || self
                                    .items
                                    .items
                                    .get(&(item as u32))
                                    .is_none_or(|d| !d.members))
                        {
                            let items = self
                                .inventories
                                .widgets
                                .get(&widget)
                                .ok_or(Error::Invalid("CS1 inventory widget"))?;
                            for i in items {
                                if i.id == item {
                                    if op == 10 {
                                        value = 999999999;
                                        break;
                                    }
                                    value = value.wrapping_add(i.count);
                                }
                            }
                        }
                    }
                    5 | 7 | 13 => {
                        let id = u16::try_from(next()?).map_err(|_| Error::Invalid("CS1 varp"))?;
                        value = self.variables.player(id);
                        if op == 7 {
                            value = value.wrapping_mul(100) / 46875;
                        } else if op == 13 {
                            value = i32::from(value & 1i32.wrapping_shl(next()? as u32) != 0);
                        }
                    }
                    6 => {
                        let level = *self
                            .data
                            .base
                            .get(next()? as usize)
                            .ok_or(Error::Invalid("CS1 skill"))?;
                        if !(1..=99).contains(&level) {
                            return Err(Error::Invalid("CS1 level"));
                        }
                        let mut points = 0;
                        for i in 1..=level {
                            points += (i as f64 + 300.0 * libm::pow(2.0, i as f64 / 7.0)) as i32;
                        }
                        value = points / 4;
                    }
                    8 => value = self.combat,
                    9 => {
                        value = self.data.base[..24]
                            .iter()
                            .fold(0i32, |a, &v| a.wrapping_add(v))
                    }
                    11 => value = self.data.energy,
                    12 => value = self.data.weight,
                    14 => value = varbit(next()?)?,
                    15..=17 => {
                        pending = op - 14;
                        continue;
                    }
                    18 => value = self.data.position[1],
                    19 => value = self.data.position[2],
                    20 => value = next()?,
                    _ => {}
                }
                total = match pending {
                    1 => total.wrapping_sub(value),
                    2 if value != 0 => total.wrapping_div(value),
                    2 => total,
                    3 => total.wrapping_mul(value),
                    _ => total.wrapping_add(value),
                };
                pending = 0;
            }
        };
        run().unwrap_or(-1)
    }
}
pub fn condition(widget: &Widget, values: &[i32]) -> bool {
    !widget.comparisons.is_empty()
        && widget
            .comparisons
            .iter()
            .enumerate()
            .all(|(i, &(kind, want))| {
                let value = values.get(i).copied().unwrap_or(-2);
                let want = want as i32;
                match kind {
                    2 => value < want,
                    3 => value > want,
                    4 => value != want,
                    _ => value == want,
                }
            })
}
pub fn substitute(text: &str, values: &[i32]) -> String {
    let mut out = text.to_owned();
    for i in 0..5 {
        let v = values.get(i).copied().unwrap_or(-2);
        let value = if v < 999999999 {
            v.to_string()
        } else {
            "*".into()
        };
        out = out.replace(&format!("%{}", i + 1), &value);
    }
    out
}
