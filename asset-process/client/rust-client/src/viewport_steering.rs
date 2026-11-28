//! Original viewport movement-mode selection and camera-ray steering.
#[derive(Clone, Copy, Debug)]
pub struct ViewHit {
    pub view: u16,
    pub policy: u8,
    pub depth: f32,
}
/// pu.ao's strict partition order matters when several scenes have equal depth.
pub fn movement_views(hits: &[ViewHit]) -> Vec<ViewHit> {
    fn sort(a: &mut [ViewHit]) {
        if a.len() < 2 {
            return;
        }
        let end = a.len() - 1;
        let middle = end / 2;
        a.swap(middle, end);
        let pivot = a[end].depth;
        let mut at = 0;
        for i in 0..end {
            if a[i].depth < pivot {
                a.swap(i, at);
                at += 1;
            }
        }
        a.swap(at, end);
        sort(&mut a[..at]);
        sort(&mut a[at + 1..]);
    }
    let mut sorted = hits.to_vec();
    sort(&mut sorted);
    let mut selected = [None; 3];
    for hit in sorted.iter().rev() {
        if (hit.depth > 0.0 || hit.view == 0) && hit.policy < 3 {
            selected[hit.policy as usize].get_or_insert(hit.view);
        }
    }
    sorted
        .into_iter()
        .filter(|hit| {
            hit.policy > 0 && hit.policy < 3 && selected[hit.policy as usize] == Some(hit.view)
        })
        .collect()
}
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub size: [i32; 2],
    pub focal: i32,
    pub far: i32,
    pub angles: [i32; 2],
    pub eye: [i32; 3],
    pub focus: [i32; 3],
}
impl Ray {
    /// hr.ad / cc.aq / io.av: quantize both endpoints before normalization.
    pub fn endpoints(self, point: [i32; 2]) -> Option<[[i32; 3]; 2]> {
        if self.focal <= 0 {
            return None;
        }
        let radians = |v: i32| (v & 16383) as f64 * 3.834951969714103E-4;
        let (sp, cp) = (
            radians(self.angles[0]).sin() as f32,
            radians(self.angles[0]).cos() as f32,
        );
        let (sy, cy) = (
            radians(self.angles[1]).sin() as f32,
            radians(self.angles[1]).cos() as f32,
        );
        let endpoint = |depth: f32| {
            let x = (point[0] - self.size[0] / 2) as f32 * depth / self.focal as f32;
            let y = depth * (point[1] - self.size[1] / 2) as f32 / self.focal as f32;
            let py = cp * y + depth * sp;
            let pz = depth * cp - sp * y;
            [
                (x * cy - sy * pz) as i32,
                py as i32,
                (x * sy + pz * cy) as i32,
            ]
        };
        Some([endpoint(50.0), endpoint(self.far as f32)])
    }
    pub fn intersection(self, point: [i32; 2]) -> Option<[f32; 3]> {
        let [near, far] = self.endpoints(point)?;
        let mut ray = std::array::from_fn::<_, 3, _>(|i| (far[i] - near[i]) as f32);
        let inv = 1.0 / (ray[1] * ray[1] + ray[0] * ray[0] + ray[2] * ray[2]).sqrt();
        for v in &mut ray {
            *v *= inv;
        }
        if ray[1].abs() < 1.0e-5 {
            return None;
        }
        let distance = -(self.eye[1] as f32 - self.focus[1] as f32) / ray[1];
        if distance < 0.0 {
            return None;
        }
        Some(std::array::from_fn(|i| {
            self.eye[i] as f32 + ray[i] * distance
        }))
    }
    pub fn heading(self, point: [i32; 2]) -> Option<u8> {
        let hit = self.intersection(point)?;
        let x = hit[0] - self.focus[0] as f32;
        let z = hit[2] - self.focus[2] as f32;
        let angle = (-x as f64).atan2(-z as f64) as f32;
        let angle = (2048.0 * (angle as f64 / std::f64::consts::TAU)) as i32 & 2047;
        Some(((angle + 64 & 2047) / 128) as u8)
    }
}
