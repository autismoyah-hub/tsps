//! Revision 239 exchange offers and trading-post listings (GameGrandExchangeOffer, mp, dk.bz).
use crate::{
    buffer::Reader,
    vm::{Object, Stacks},
    Error, Result,
};

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |v| v.as_millis() as i64)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Offer {
    pub state: u8,
    pub item: i32,
    pub price: i32,
    pub quantity: i32,
    pub completed: i32,
    pub spent: i32,
}
impl Offer {
    pub fn decode(r: &mut Reader<'_>) -> Result<Self> {
        Ok(Self {
            state: r.u8()?,
            item: r.u16()? as i32,
            price: r.u32()? as i32,
            quantity: r.u32()? as i32,
            completed: r.u32()? as i32,
            spent: r.u32()? as i32,
        })
    }
    pub fn slot(r: &mut Reader<'_>) -> Result<(u8, Self)> {
        let slot = r.u8()?;
        if slot >= 8 {
            return Err(Error::Invalid("exchange offer slot"));
        }
        let offer = Self::decode(r)?;
        Ok((
            slot,
            if offer.state == 0 {
                Self::default()
            } else {
                offer
            },
        ))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Listing {
    pub name: String,
    pub previous_name: String,
    pub world: i32,
    pub timestamp: i64,
    pub offer: Offer,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub server_time: i64,
    pub listings: Vec<Listing>,
}
impl Snapshot {
    pub fn decode(r: &mut Reader<'_>) -> Result<Option<Self>> {
        if r.u8()? != 1 {
            return Ok(None);
        }
        let server_time = r.u64()? as i64;
        let item = r.u16()? as i32;
        let state = 2 | if r.u8()? == 1 { 8 } else { 0 };
        let count = r.u16()? as usize;
        let mut listings = Vec::with_capacity(count);
        for _ in 0..count {
            listings.push(Listing {
                name: r.string()?,
                previous_name: r.string()?,
                world: r.u16()? as i32,
                timestamp: r.u64()? as i64,
                offer: Offer {
                    state,
                    item,
                    price: r.u32()? as i32,
                    quantity: r.u32()? as i32,
                    ..Default::default()
                },
            });
        }
        Ok(Some(Self {
            server_time,
            listings,
        }))
    }
}
#[derive(Default)]
pub struct Market {
    pub offers: [Offer; 8],
    pub listings: Option<Vec<Listing>>,
    pub time_offset: i64,
}
impl Market {
    pub fn replace(&mut self, snapshot: Option<Snapshot>, now: i64) {
        self.listings = snapshot.map(|s| {
            self.time_offset = now.wrapping_sub(s.server_time);
            s.listings
        });
    }
    pub fn execute(&mut self, op: u16, s: &mut Stacks, world: i32, now: i64) -> Result<bool> {
        match op {
            3903..=3908 | 3910..=3913 => {
                let index = s.pop_int()? as usize;
                let o = self
                    .offers
                    .get(index)
                    .ok_or(Error::Invalid("exchange offer index"))?;
                let status = o.state & 7;
                s.push_int(match op {
                    3903 => ((o.state >> 3) & 1) as i32,
                    3904 => o.item,
                    3905 => o.price,
                    3906 => o.quantity,
                    3907 => o.completed,
                    3908 => o.spent,
                    3910 => (status == 0) as i32,
                    3911 => (status == 2) as i32,
                    3912 => (status == 5) as i32,
                    _ => (status == 1) as i32,
                })?;
            }
            3914..=3918 => {
                let priority = if op == 3916 { s.pop_int()? == 1 } else { false };
                let ascending = s.pop_int()? == 1;
                if let Some(list) = &mut self.listings {
                    // All five comparators are total orders; stable sorting preserves equal offers.
                    list.sort_by(|a, b| {
                        let order = match op {
                            3914 => a.name.encode_utf16().cmp(b.name.encode_utf16()),
                            3915 => a.offer.price.cmp(&b.offer.price),
                            3916 => {
                                if priority
                                    && a.world != b.world
                                    && (a.world == world || b.world == world)
                                {
                                    (b.world == world).cmp(&(a.world == world))
                                } else {
                                    a.world.cmp(&b.world)
                                }
                            }
                            3917 => a.timestamp.cmp(&b.timestamp),
                            _ => a.offer.quantity.cmp(&b.offer.quantity),
                        };
                        if ascending {
                            order
                        } else {
                            order.reverse()
                        }
                    });
                }
            }
            3919 => s.push_int(self.listings.as_ref().map_or(0, |v| v.len() as i32))?,
            3920..=3926 => {
                let index = s.pop_int()? as usize;
                let l = self
                    .listings
                    .as_ref()
                    .and_then(|v| v.get(index))
                    .ok_or(Error::Invalid("trading-post listing index"))?;
                match op {
                    3921 => s.push_object(Object::String(l.name.clone()))?,
                    3922 => s.push_object(Object::String(l.previous_name.clone()))?,
                    3923 => s.push_object(Object::String(age(now
                        .wrapping_sub(self.time_offset)
                        .wrapping_sub(l.timestamp))))?,
                    _ => s.push_int(match op {
                        3920 => l.world,
                        3924 => l.offer.quantity,
                        3925 => l.offer.price,
                        _ => l.offer.item,
                    })?,
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
/// The intermediate products are Java int operations, even though the elapsed time is a long.
pub fn age(elapsed: i64) -> String {
    let hours = (elapsed / 3_600_000) as i32;
    let minutes = (elapsed.wrapping_sub(hours.wrapping_mul(3_600_000) as i64) / 60_000) as i32;
    let seconds = (elapsed
        .wrapping_sub(hours.wrapping_mul(3_600_000) as i64)
        .wrapping_sub(minutes.wrapping_mul(60_000) as i64)
        / 1000) as i32;
    format!(
        "{}:{}{}:{}{}",
        hours,
        minutes / 10,
        minutes % 10,
        seconds / 10,
        seconds % 10
    )
}
