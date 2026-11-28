//! Original wv map archive readiness, independent of geography rendering.
#[derive(Default, Debug)]
pub struct Loading {
    pub area: Option<i32>,
    pub percent: i32,
    pub ready: bool,
}
impl Loading {
    pub fn select(&mut self, area: Option<i32>, mut available: impl FnMut(u32, i32) -> bool) {
        if area.is_some() && area != self.area {
            self.area = area;
            self.percent = 0;
            self.ready = false;
            self.advance(&mut available);
        }
    }
    pub fn advance(&mut self, mut available: impl FnMut(u32, i32) -> bool) {
        let Some(id) = self.area else {
            self.percent = 100;
            self.ready = true;
            return;
        };
        if self.percent < 33 {
            if !available(1, id) {
                return;
            }
            self.percent = 33;
        }
        if self.percent == 33 {
            if !available(2, id) {
                return;
            }
            self.percent = 66;
        }
        if self.percent == 66 {
            self.percent = 100;
            self.ready = true;
        }
    }
}
