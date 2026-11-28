//! Original vh/vk camera interpolation, including revision-specific easing formulas.
#[derive(Clone, Debug)]
pub struct Scalar {
    pub start: i32,
    pub end: i32,
    pub duration: i32,
    pub elapsed: i32,
    pub mode: u8,
}
#[derive(Clone, Debug)]
pub struct Path {
    pub clock: Scalar,
    start: [i32; 3],
    end: [i32; 3],
    arc: Option<[f64; 5]>,
}
impl Path {
    pub fn new(
        start: [i32; 3],
        end: [i32; 3],
        middle: Option<[i32; 2]>,
        duration: i32,
        mode: i32,
    ) -> Self {
        let arc = middle.map(|[mx, mz]| {
            let [sx, _, sz] = start;
            let [ex, _, ez] = end;
            if mx.wrapping_sub(sx).wrapping_mul(ez.wrapping_sub(mz))
                == ex.wrapping_sub(mx).wrapping_mul(mz.wrapping_sub(sz))
            {
                return [ex as f64, ez as f64, 0.0, 0.0, 0.0];
            }
            let ax = mx.wrapping_add(sx) as f64 / 2.0;
            let az = sz.wrapping_add(mz) as f64 / 2.0;
            let bx = mx.wrapping_add(ex) as f64 / 2.0;
            let bz = ez.wrapping_add(mz) as f64 / 2.0;
            let a = -1.0 * mx.wrapping_sub(sx) as f64 / mz.wrapping_sub(sz) as f64;
            let b = ex.wrapping_sub(mx) as f64 * -1.0 / ez.wrapping_sub(mz) as f64;
            let x = (bz + (a * ax - bx * b) - az) / (a - b);
            let z = a * (x - ax) + az;
            let radius = (libm::pow(x - sx as f64, 2.0) + libm::pow(z - sz as f64, 2.0)).sqrt();
            let first = (sz as f64 - z).atan2(sx as f64 - x);
            let middle = (mz as f64 - z).atan2(mx as f64 - x);
            let mut last = (ez as f64 - z).atan2(ex as f64 - x);
            if !((first <= middle && middle <= last) || (last <= middle && middle <= first)) {
                last += std::f64::consts::PI * if first - last > 0.0 { 2.0 } else { -2.0 };
            }
            [x, z, radius, first, last]
        });
        Self {
            clock: Scalar::new(0, 1, duration, mode),
            start,
            end,
            arc,
        }
    }
    pub fn value(&self) -> [i32; 3] {
        let t = self.clock.progress();
        let round = |v: f64| (v + 0.5).floor() as i64 as i32;
        let mut point = std::array::from_fn(|i| {
            round(self.end[i].wrapping_sub(self.start[i]) as f64 * t + self.start[i] as f64)
        });
        if let Some([x, z, radius, first, last]) = self.arc {
            let angle = (last - first) * t + first;
            point[0] = round(x + radius * angle.cos());
            point[2] = round(z + radius * angle.sin());
        }
        point
    }
}
impl Scalar {
    pub fn new(start: i32, end: i32, duration: i32, mode: i32) -> Self {
        Self {
            start,
            end,
            duration,
            elapsed: 0,
            mode: if (0..=27).contains(&mode) {
                mode as u8
            } else {
                0
            },
        }
    }
    pub fn advance(&mut self) {
        if self.elapsed < self.duration {
            self.elapsed += 1;
        }
    }
    pub fn complete(&self) -> bool {
        self.elapsed >= self.duration
    }
    pub fn progress(&self) -> f64 {
        ease(self.elapsed, self.duration, self.mode)
    }
    pub fn value(&self) -> i32 {
        let value = self.end.wrapping_sub(self.start) as f64 * self.progress() + self.start as f64;
        (value + 0.5).floor() as i64 as i32
    }
}
pub fn ease(elapsed: i32, duration: i32, mode: u8) -> f64 {
    ease_millis(elapsed as i64, duration as i64, mode)
}
pub fn ease_millis(elapsed: i64, duration: i64, mode: u8) -> f64 {
    let t = if duration > 0 {
        (elapsed as f32 / duration as f32).clamp(0.0, 1.0) as f64
    } else {
        1.0
    };
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let pi = std::f64::consts::PI;
    let pow = libm::pow;
    match mode {
        1 => 1.0 - (pi * t / 2.0).cos(),
        2 => (t * pi / 2.0).sin(),
        3 => -((pi * t).cos() - 1.0) / 2.0,
        4 => t * t,
        5 => 1.0 - (1.0 - t) * (1.0 - t),
        6 => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - pow(2.0 + -2.0 * t, 2.0) / 2.0
            }
        }
        7 => t * t * t,
        8 => 1.0 - pow(1.0 - t, 3.0),
        9 => {
            if t < 0.5 {
                t * (4.0 * t * t)
            } else {
                1.0 - pow(2.0 + t * -2.0, 3.0) / 2.0
            }
        }
        10 => t * (t * (t * t)),
        11 => 1.0 - pow(1.0 - t, 4.0),
        12 => {
            if t < 0.5 {
                t * (t * (8.0 * t)) * t
            } else {
                1.0 - pow(2.0 + t * -2.0, 4.0) / 2.0
            }
        }
        13 => t * (t * (t * t)) * t,
        14 => 1.0 - pow(1.0 - t, 5.0),
        15 => {
            if t < 0.5 {
                t * (t * (t * (8.0 * t * t)))
            } else {
                1.0 - pow(-2.0 * t + 2.0, 5.0) / 2.0
            }
        }
        16 => pow(2.0, 10.0 * t - 10.0),
        17 => 1.0 - pow(2.0, t * -10.0),
        // Preserve the installed client's +10 exponent in the first half.
        18 => {
            if t < 0.5 {
                pow(2.0, t * 20.0 + 10.0) / 2.0
            } else {
                (2.0 - pow(2.0, -20.0 * t + 10.0)) / 2.0
            }
        }
        19 => 1.0 - (1.0 - pow(t, 2.0)).sqrt(),
        20 => (1.0 - pow(t - 1.0, 2.0)).sqrt(),
        21 => {
            if t < 0.5 {
                (1.0 - (1.0 - pow(t * 2.0, 2.0)).sqrt()) / 2.0
            } else {
                ((1.0 - pow(2.0 + t * -2.0, 2.0)).sqrt() + 1.0) / 2.0
            }
        }
        22 => t * (t * 2.70158) * t - 1.70158 * t * t,
        23 => 1.0 + 2.70158 * pow(t - 1.0, 3.0) + 1.70158 * pow(t - 1.0, 2.0),
        24 => {
            if t < 0.5 {
                pow(t * 2.0, 2.0) * (t * 7.189819 - 2.5949095) / 2.0
            } else {
                (pow(t * 2.0 - 2.0, 2.0) * (2.5949095 + 3.5949095 * (2.0 * t - 2.0)) + 2.0) / 2.0
            }
        }
        25 => -pow(2.0, 10.0 * t - 10.0) * ((t * 10.0 - 10.75) * (pi * 2.0 / 3.0)).sin(),
        26 => pow(2.0, -10.0 * t) * ((pi * 2.0 / 3.0) * (t * 10.0 - 0.75)).sin() + 1.0,
        27 => {
            let sine = ((20.0 * t - 11.125) * (pi * 4.0 / 9.0)).sin();
            if t < 0.5 {
                -(pow(2.0, t * 20.0 - 10.0) * sine) / 2.0
            } else {
                pow(2.0, -20.0 * t + 10.0) * sine / 2.0 + 1.0
            }
        }
        _ => t,
    }
}
