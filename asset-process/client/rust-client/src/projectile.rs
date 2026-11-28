//! Revision 239 homing projectile motion (GameProjectile.ap).
#[derive(Clone, Debug, Default)]
pub struct Motion {
    pub position: [f64; 3],
    pub vertical_speed: f64,
    pub moving: bool,
    pub yaw: u16,
    pub pitch: u16,
}
impl Motion {
    /// Coordinates are fine X, fine Z and elevation; targets may change each tick.
    pub fn advance(
        &mut self,
        source: [i32; 3],
        target: [i32; 3],
        start: i32,
        end: i32,
        cycle: i32,
        ticks: i32,
        slope: i32,
        offset: i32,
    ) {
        if cycle < start {
            return;
        }
        if !self.moving {
            let dx = target[0].wrapping_sub(source[0]) as f64;
            let dz = target[1].wrapping_sub(source[1]) as f64;
            let distance = (dx * dx + dz * dz).sqrt();
            self.position = [
                if distance.abs() < (0.01f32 as f64) {
                    source[0] as f64
                } else {
                    source[0] as f64 + offset as f64 * dx / distance
                },
                if distance.abs() < (0.01f32 as f64) {
                    source[1] as f64
                } else {
                    dz * offset as f64 / distance + source[1] as f64
                },
                source[2] as f64,
            ];
        }
        let remaining = end.wrapping_add(1).wrapping_sub(cycle) as f64;
        let vx = (target[0] as f64 - self.position[0]) / remaining;
        let vz = (target[1] as f64 - self.position[1]) / remaining;
        let horizontal = (vx * vx + vz * vz).sqrt();
        if !self.moving {
            self.vertical_speed = -horizontal * libm::tan(0.02454369 * slope as f64);
        }
        let acceleration = 2.0
            * (target[2] as f64 - self.position[2] - self.vertical_speed * remaining)
            / (remaining * remaining);
        self.moving = true;
        let dt = ticks as f64;
        self.position[0] += dt * vx;
        self.position[1] += dt * vz;
        self.position[2] += dt * (acceleration * 0.5) * dt + self.vertical_speed * dt;
        self.vertical_speed += dt * acceleration;
        self.yaw = (((libm::atan2(vx, vz) * 325.949) as i32 + 1024) & 2047) as u16;
        self.pitch =
            ((libm::atan2(self.vertical_speed, horizontal) * 325.949) as i32 & 2047) as u16;
    }
}

#[derive(Clone, Debug)]
pub struct Request {
    pub id: u16,
    pub source: [i32; 3],
    pub target: [i32; 3],
    pub heights: [u16; 2],
    pub entities: [i32; 2],
    pub start: i32,
    pub end: i32,
    pub slope: u8,
    pub offset: u16,
}
impl Request {
    pub fn decode(origin: [i32; 3], cycle: i32, bytes: &[u8]) -> crate::Result<Self> {
        use crate::buffer::{Reader, Transform};
        let mut r = Reader::new(bytes);
        let start = cycle.wrapping_add(r.u16()? as i32);
        let at = r.u8()?;
        let offset = r.u16_add(false)?;
        let id = r.u16_add(false)?;
        let end = cycle.wrapping_add(r.u16_le()? as i32);
        let packed = r.u32_le()?;
        let target = [
            (packed as i32) >> 28,
            ((packed >> 14) & 16383) as i32,
            (packed & 16383) as i32,
        ];
        let end_height = r.u16_add(true)?;
        let b = r.bytes(3)?;
        let source_entity =
            ((((b[0] as i32) << 16) | ((b[2] as i32) << 8) | b[1] as i32) << 8) >> 8;
        let slope = r.transformed(Transform::Neg)?;
        let start_height = r.u16()?;
        let target_entity = ((r.u24()? as i32) << 8) >> 8;
        if r.remaining() != 0 {
            return Err(crate::Error::Invalid("projectile packet trailing bytes"));
        }
        Ok(Self {
            id,
            source: [
                origin[0],
                origin[1] + ((at >> 4) & 7) as i32,
                origin[2] + (at & 7) as i32,
            ],
            target,
            heights: [start_height, end_height],
            entities: [source_entity, target_entity],
            start,
            end,
            slope,
            offset,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Projectile {
    pub request: Request,
    pub motion: Motion,
    pub clock: crate::animation_runtime::Clock,
}
impl Request {
    pub fn start(mut self, cycle: i32) -> Projectile {
        self.start = self.start.wrapping_add(cycle);
        self.end = self.end.wrapping_add(cycle);
        Projectile {
            request: self,
            motion: Default::default(),
            clock: Default::default(),
        }
    }
}

impl Request {
    pub fn specific(r: &mut crate::buffer::Reader<'_>) -> crate::Result<Self> {
        use crate::buffer::Transform;
        let start = r.u16_add(false)? as i32;
        let end = r.u16_add(false)? as i32;
        let heights = [r.u16_add(true)?, r.u16_add(true)?];
        let slope = r.transformed(Transform::Sub)?;
        let coord = r.u32_le()?;
        let source = [
            (coord as i32) >> 28,
            ((coord >> 14) & 16383) as i32,
            (coord & 16383) as i32,
        ];
        let offset = r.u16_add(true)?;
        let id = r.u16_le()?;
        let coord = r.u32_le()?;
        let target = [
            (coord as i32) >> 28,
            ((coord >> 14) & 16383) as i32,
            (coord & 16383) as i32,
        ];
        let b = r.bytes(3)?;
        let a = ((((b[1] as i32) << 16) | ((b[0] as i32) << 8) | b[2] as i32) << 8) >> 8;
        let b = ((r.u24()? as i32) << 8) >> 8;
        Ok(Self {
            id,
            source,
            target,
            heights,
            entities: [a, b],
            start,
            end,
            slope,
            offset,
        })
    }
}
