//! MIDI channel controls used by the native sample synthesizer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Channel {
    pub volume: i32,
    pub pan: i32,
    pub expression: i32,
    pub bend: i32,
    pub modulation: i32,
    pub portamento: i32,
    pub flags: i32,
    pub parameter: i32,
    pub bend_range: i32,
    pub sample_offset: i32,
    pub retrigger: i32,
    pub retrigger_rate: i32,
    pub bank: i32,
    pub program: i32,
    pub default_program: i32,
}

/// Voice operations accompany controller changes; the mixer must consume them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Actions {
    pub release_portamento: bool,
    pub reset_retrigger: bool,
    pub silence: bool,
    pub release_notes: bool,
    pub clear_groups: bool,
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            volume: 12800,
            pan: 8192,
            expression: 16383,
            bend: 8192,
            modulation: 0,
            portamento: 8192,
            flags: 0,
            parameter: 32767,
            bend_range: 256,
            sample_offset: 0,
            retrigger: 8192,
            retrigger_rate: retrigger_rate(8192),
            bank: 0,
            program: 0,
            default_program: 0,
        }
    }
}

fn retrigger_rate(value: i32) -> i32 {
    (2097152.0 * 2.0f64.powf((value as f32 * 5.493164e-4f32) as f64) + 0.5) as i32
}
fn high(old: i32, value: i32) -> i32 {
    (old & !16256) + (value << 7)
}
fn low(old: i32, value: i32) -> i32 {
    (old & !127) + value
}

impl Channel {
    pub fn reset_controllers(&mut self) -> Actions {
        let actions = Actions {
            release_portamento: self.flags & 2 != 0,
            reset_retrigger: self.flags & 4 != 0,
            ..Actions::default()
        };
        *self = Self {
            bank: self.bank,
            program: self.program,
            default_program: self.default_program,
            ..Self::default()
        };
        actions
    }

    pub fn reset(&mut self) -> Actions {
        let mut actions = self.reset_controllers();
        self.program = self.default_program;
        self.bank = self.default_program & !127;
        actions.silence = true;
        actions.clear_groups = true;
        actions
    }

    pub fn program_change(&mut self, value: u8) -> Actions {
        let program = self.bank + (value & 127) as i32;
        let changed = self.program != program;
        self.program = program;
        Actions {
            clear_groups: changed,
            ..Actions::default()
        }
    }

    pub fn control(&mut self, control: u8, value: u8) -> Actions {
        let value = (value & 127) as i32;
        let mut actions = Actions::default();
        match control {
            0 => self.bank = (self.bank & !2080768) + (value << 14),
            32 => self.bank = high(self.bank, value),
            1 => self.modulation = high(self.modulation, value),
            33 => self.modulation = low(self.modulation, value),
            5 => self.portamento = high(self.portamento, value),
            37 => self.portamento = low(self.portamento, value),
            7 => self.volume = high(self.volume, value),
            39 => self.volume = low(self.volume, value),
            10 => self.pan = high(self.pan, value),
            42 => self.pan = low(self.pan, value),
            11 => self.expression = high(self.expression, value),
            43 => self.expression = low(self.expression, value),
            64 | 65 | 81 => {
                let flag = match control {
                    64 => 1,
                    65 => 2,
                    _ => 4,
                };
                if value >= 64 {
                    self.flags |= flag;
                } else {
                    actions.release_portamento = flag == 2 && self.flags & 2 != 0;
                    actions.reset_retrigger = flag == 4 && self.flags & 4 != 0;
                    self.flags &= !flag;
                }
            }
            99 => self.parameter = (self.parameter & 127) + (value << 7),
            98 => self.parameter = (self.parameter & 16256) + value,
            101 => self.parameter = 16384 + (self.parameter & 127) + (value << 7),
            100 => self.parameter = 16384 + (self.parameter & 16256) + value,
            120 => actions.silence = true,
            121 => return self.reset_controllers(),
            123 => actions.release_notes = true,
            6 if self.parameter == 16384 => self.bend_range = high(self.bend_range, value),
            38 if self.parameter == 16384 => self.bend_range = low(self.bend_range, value),
            16 => self.sample_offset = high(self.sample_offset, value),
            48 => self.sample_offset = low(self.sample_offset, value),
            17 | 49 => {
                self.retrigger = if control == 17 {
                    high(self.retrigger, value)
                } else {
                    low(self.retrigger, value)
                };
                self.retrigger_rate = retrigger_rate(self.retrigger);
            }
            _ => {}
        }
        actions
    }
}
