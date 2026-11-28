//! GameModel.ay cylinder bounds and GameModel.by root-view frustum rejection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub position: [i32; 3],
    pub angle: u16,
    pub selection: Option<crate::model_pick::Shape>,
    pub radius: i32,
    pub height: i32,
    pub below: i32,
}
impl Bounds {
    /// Models joined by GameModel's constructor share one draw origin.
    pub fn union(self, other: Self) -> Option<Self> {
        (self.position == other.position).then_some(Self {
            position: self.position,
            angle: self.angle,
            selection: self.selection.zip(other.selection).map(|(a, b)| a.union(b)),
            radius: self.radius.max(other.radius),
            height: self.height.max(other.height),
            below: self.below.max(other.below),
        })
    }
    pub fn with_selection(mut self, vertices: &[[f32; 3]], angle: u16, single_tile: bool) -> Self {
        self.angle = angle;
        self.selection = Some(crate::model_pick::Shape::new(vertices, angle, single_tile));
        self
    }
    pub fn new(vertices: &[[f32; 3]], position: [i32; 3]) -> Self {
        let (mut up, mut down, mut radius) = (0.0f32, 0.0f32, 0.0f32);
        for &[x, y, z] in vertices {
            if -y > up {
                up = -y;
            }
            if y > down {
                down = y;
            }
            let squared = x * x + z * z;
            if squared > radius {
                radius = squared;
            }
        }
        Self {
            position,
            angle: 0,
            selection: None,
            radius: (radius as f64).sqrt().ceil() as i32,
            height: up.ceil() as i32,
            below: down.ceil() as i32,
        }
    }
}
pub struct Frustum {
    pub eye: [i32; 3],
    pub trig: [i32; 4],
    pub focal: i32,
    pub far: i32,
    /// Left, right, top and bottom limits relative to the projection center.
    pub clip: [i32; 4],
}
pub fn trig(pitch: i32, yaw: i32) -> [i32; 4] {
    let angle = |v: i32| (v & 16383) as f64 * 3.834951969714103E-4;
    [
        (angle(pitch).sin() * 65536.0) as i32,
        (angle(pitch).cos() * 65536.0) as i32,
        (angle(yaw).sin() * 65536.0) as i32,
        (angle(yaw).cos() * 65536.0) as i32,
    ]
}
impl Frustum {
    pub fn visible(&self, bounds: Bounds) -> bool {
        let [x, y, z]: [i32; 3] =
            std::array::from_fn(|i| bounds.position[i].wrapping_sub(self.eye[i]));
        let [sp, cp, sy, cy] = self.trig;
        let yaw_z = z.wrapping_mul(cy).wrapping_sub(x.wrapping_mul(sy)) >> 16;
        let center_z = y.wrapping_mul(sp).wrapping_add(yaw_z.wrapping_mul(cp)) >> 16;
        let reach = bounds.radius.wrapping_mul(cp) >> 16;
        let front = center_z.wrapping_add(reach);
        if front <= 50 || center_z >= self.far {
            return false;
        }
        let screen_x = z.wrapping_mul(sy).wrapping_add(x.wrapping_mul(cy)) >> 16;
        let project = |v: i32| v.wrapping_mul(self.focal) / front;
        if project(screen_x.wrapping_sub(bounds.radius)) >= self.clip[1]
            || project(screen_x.wrapping_add(bounds.radius)) <= self.clip[0]
        {
            return false;
        }
        let screen_y = y.wrapping_mul(cp).wrapping_sub(yaw_z.wrapping_mul(sp)) >> 16;
        let pitch_radius = bounds.radius.wrapping_mul(sp) >> 16;
        let bottom = pitch_radius.wrapping_add(bounds.below.wrapping_mul(cp) >> 16);
        let top = pitch_radius.wrapping_add(bounds.height.wrapping_mul(cp) >> 16);
        project(screen_y.wrapping_add(bottom)) > self.clip[2]
            && project(screen_y.wrapping_sub(top)) < self.clip[3]
    }
}
