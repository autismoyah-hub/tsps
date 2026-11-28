//! Original engine canvas limits, placement and delayed window-status scheduling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Geometry {
    pub canvas: [i32; 2],
    pub origin: [i32; 2],
}
/// GameGameEngine.aio: minimum content bounds, maximum canvas, horizontal centering.
pub fn geometry(physical: [i32; 2], minimum: [i32; 2], maximum: [i32; 2]) -> Geometry {
    let content = std::array::from_fn::<_, 2, _>(|i| physical[i].max(minimum[i]).max(1));
    let canvas = std::array::from_fn(|i| content[i].min(maximum[i]));
    Geometry {
        canvas,
        origin: [(content[0] - canvas[0]) / 2, 0],
    }
}
#[derive(Clone, Debug)]
pub struct State {
    pub mode: i32,
    pub maximum: [i32; 2],
    pub resize_requested: bool,
    pub game_state: i32,
    pub deadline: Option<u64>,
    physical: Option<[i32; 2]>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            mode: 1,
            maximum: [765, 503],
            resize_requested: false,
            game_state: 0,
            deadline: None,
            physical: None,
        }
    }
}
impl State {
    /// el.cg clears the deadline even if the selected mode and limits are unchanged.
    pub fn set_mode(&mut self, mode: i32) {
        self.deadline = None;
        self.mode = if mode >= 2 { 2 } else { 1 };
        let maximum = if self.mode == 1 {
            [765, 503]
        } else {
            [7680, 2160]
        };
        self.resize_requested |= maximum != self.maximum;
        self.maximum = maximum;
    }
    /// bj.gg only accepts the two public modes; invalid inputs leave timing untouched.
    pub fn script_mode(&mut self, mode: i32) -> bool {
        if !matches!(mode, 1 | 2) {
            return false;
        }
        self.set_mode(mode);
        true
    }
    /// Canvas resize invokes GameClient.ig even if the maximum keeps canvas size unchanged.
    pub fn frame(
        &mut self,
        physical: [i32; 2],
        now: u64,
        game_state: i32,
    ) -> (Geometry, bool, bool) {
        self.game_state = game_state;
        let resized = self.resize_requested || self.physical != Some(physical);
        if resized {
            self.physical = Some(physical);
            self.resize_requested = false;
            self.deadline = Some(now.saturating_add(500));
        }
        let send_status = if matches!(game_state, 10 | 20 | 30)
            && self.deadline.is_some_and(|deadline| now > deadline)
        {
            self.set_mode(self.mode);
            game_state >= 25
        } else {
            false
        };
        (
            geometry(physical, [0, 0], self.maximum),
            resized,
            send_status,
        )
    }
}
