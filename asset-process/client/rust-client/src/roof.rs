//! Camera-dependent maximum scene plane (original qi.cm / mk.cr).
//! Coordinates are local fine units; pitch uses the client's 16384-unit circle.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub plane: u8,
    pub size: [i32; 2],
    pub hidden: bool,
    pub eye: [i32; 3],
    pub pitch: i32,
    pub tracked: [i32; 2],
    pub orb: Option<[i32; 2]>,
}
impl View {
    fn inside(&self, tile: [i32; 2]) -> bool {
        (0..2).all(|i| tile[i] >= 0 && tile[i] < self.size[i])
    }
    pub fn following(&self, roof: impl Fn([i32; 2]) -> bool) -> u8 {
        if self.hidden {
            return self.plane;
        }
        let tracked = self.tracked.map(|v| v >> 7);
        if self.pitch < 2480 {
            let target = self.orb.unwrap_or(self.tracked).map(|v| v >> 7);
            let mut tile = [self.eye[0] >> 7, self.eye[2] >> 7];
            if !self.inside(tile) || !self.inside(target) || roof(tile) {
                return self.plane;
            }
            let delta = [(target[0] - tile[0]).abs(), (target[1] - tile[1]).abs()];
            let major = usize::from(delta[0] <= delta[1]);
            let minor = 1 - major;
            if delta[major] > 0 {
                let step = delta[minor] * 65536 / delta[major];
                let mut fraction = 32768;
                while tile[major] != target[major] {
                    tile[major] += (target[major] - tile[major]).signum();
                    if roof(tile) {
                        return self.plane;
                    }
                    fraction += step;
                    if fraction >= 65536 {
                        fraction -= 65536;
                        tile[minor] += (target[minor] - tile[minor]).signum();
                        if roof(tile) {
                            return self.plane;
                        }
                    }
                }
            }
        }
        if !self.inside(tracked) || roof(tracked) {
            self.plane
        } else {
            3
        }
    }
    pub fn locked(&self, roof: impl Fn([i32; 2]) -> bool, elevation: i32) -> u8 {
        let tile = [self.eye[0] >> 7, self.eye[2] >> 7];
        if self.hidden
            || !self.inside(tile)
            || (elevation.wrapping_sub(self.eye[1]) < 800 && roof(tile))
        {
            self.plane
        } else {
            3
        }
    }
}
