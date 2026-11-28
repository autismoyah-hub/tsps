//! World-map pointer drag state (GameRenderOverview.aa).
#[derive(Debug)]
pub struct Pointer {
    pub drag_pointer: [i32; 2],
    pub drag_center: [i32; 2],
    pub last_press: [i32; 2],
    pub last_time: i64,
    pub double_click: bool,
    pub coord: Option<[i32; 3]>,
}
impl Default for Pointer {
    fn default() -> Self {
        Self {
            drag_pointer: [-1; 2],
            drag_center: [-1; 2],
            last_press: [0; 2],
            last_time: 0,
            double_click: true,
            coord: None,
        }
    }
}
impl Pointer {
    pub fn project_click(
        &mut self,
        area: Option<&crate::map_area::Area>,
        center: [i32; 2],
        zoom: f32,
        display: [i32; 2],
        origin: [i32; 2],
        point: [i32; 2],
        press: bool,
        admin: bool,
        now: i64,
    ) -> Option<crate::protocol::Packet> {
        self.coord = area.and_then(|area| {
            let x = (center[0] as f32
                + (point[0].wrapping_sub(origin[0]) as f32 - display[0] as f32 * zoom / 2.0) / zoom)
                as i32;
            let y = (center[1] as f32
                - (point[1].wrapping_sub(origin[1]) as f32 - display[1] as f32 * zoom / 2.0) / zoom)
                as i32;
            area.coord(
                x.wrapping_add(area.bounds[0].wrapping_mul(64)),
                y.wrapping_add(area.bounds[1].wrapping_mul(64)),
            )
        });
        let coord = self.coord?;
        if !press {
            return None;
        }
        let mut writer = crate::writer::Writer::new();
        let opcode = if admin {
            writer.u32(0);
            writer.u8((coord[0] as u8).wrapping_add(128));
            writer.u16(coord[2] as u16);
            writer.u16_add(coord[1] as u16, false);
            50
        } else {
            let dx = point[0].wrapping_sub(self.last_press[0]);
            let dy = point[1].wrapping_sub(self.last_press[1]);
            if self.double_click
                && (now.wrapping_sub(self.last_time) > 500
                    || !(-25..=25).contains(&dx)
                    || !(-25..=25).contains(&dy))
            {
                return None;
            }
            writer.u32_le((coord[0] << 28 | coord[1] << 14 | coord[2]) as u32);
            self.last_time = 0;
            99
        };
        Some(crate::protocol::Packet {
            opcode,
            payload: writer.into_vec(),
        })
    }

    pub fn reset_drag(&mut self) {
        self.drag_pointer = [-1; 2];
        self.drag_center = [-1; 2];
    }
    pub fn pan(
        &mut self,
        center: &mut [i32; 2],
        target: [i32; 2],
        zoom: f32,
        point: [i32; 2],
        held: bool,
        press: bool,
        blocked: bool,
        now: i64,
    ) {
        if !target.iter().all(|&v| v != -1) && (press || held) && !blocked {
            if press {
                self.drag_pointer = point;
                self.drag_center = *center;
            }
            if self.drag_center[0] != -1 {
                let delta = [
                    point[0].wrapping_sub(self.drag_pointer[0]),
                    point[1].wrapping_sub(self.drag_pointer[1]),
                ];
                *center = [
                    self.drag_center[0].wrapping_sub((delta[0] as f32 / zoom) as i32),
                    self.drag_center[1].wrapping_add((delta[1] as f32 / zoom) as i32),
                ];
            }
        } else {
            self.reset_drag();
        }
        if press {
            self.last_press = point;
            self.last_time = now;
        }
    }
}

/// Elapsed map-click time follows a monotonic clock after its wall-clock origin.
pub fn millis() -> i64 {
    static CLOCK: std::sync::OnceLock<(std::time::Instant, i64)> = std::sync::OnceLock::new();
    let (epoch, base) = CLOCK.get_or_init(|| {
        (
            std::time::Instant::now(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        )
    });
    base.wrapping_add(epoch.elapsed().as_millis() as i64)
}
