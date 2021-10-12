use super::envelope::Envelope;

const CPU_FREQ: u32 = 1789773;

pub(crate) struct Pulse {
    pub settings: PulseSettings,
    t: u16,
    cycle: u8,
    sweeper_reload_flag: bool,
    sweeper_divider_counter: u8,
    mute: bool,

    envelope: Envelope,

    pub length_counter: u8,
}

impl Pulse {
    pub(crate) fn new() -> Self {
        Self {
            settings: PulseSettings::new(),
            t: 0,
            cycle: 0,
            sweeper_reload_flag: false,
            sweeper_divider_counter: 0,
            mute: false,

            envelope: Envelope::new(),

            length_counter: 0,
        }
    }

    pub(crate) fn tick(&mut self, sweep_tick: bool, envelope_tick: bool, is_1: bool) {
        if self.t == 0 {
            self.t = self.settings.timer;
            self.cycle = (self.cycle + 1) % 8;
        } else {
            self.t -= 1;
        }

        let change_amount = self.settings.timer >> self.settings.sweep_shift;
        let change = if self.settings.sweep_negate {
            if is_1 {
                0xFFFFu16.wrapping_sub(change_amount)
            } else {
                0u16.wrapping_sub(change_amount)
            }
        } else {
            change_amount
        };
        let target_timer = self.settings.timer.wrapping_add(change);
        self.mute = (self.settings.timer < 8) || target_timer > 0x7FF;

        if sweep_tick {
            if self.sweeper_divider_counter == 0 && self.settings.sweep_enable && !self.mute {
                self.settings.timer = target_timer;
            }
            if self.sweeper_divider_counter == 0 || self.sweeper_reload_flag {
                self.sweeper_divider_counter = self.settings.sweep_period;
                self.sweeper_reload_flag = false;
            } else {
                self.sweeper_divider_counter -= 1;
            }

            // length counter stuff
            if !self.settings.length_counter_halt {
                if self.length_counter != 0 {
                    self.length_counter -= 1;
                }
            }
        }

        if envelope_tick {
            self.envelope.tick();
        }
    }

    pub(crate) fn next_value(&mut self) -> u8 {
        match self.cycle {
            0..=3 => {
                if self.settings.duty == 3 {
                    self.get_volume()
                } else {
                    0
                }
            }
            4 | 5 => {
                if self.settings.duty == 0 || self.settings.duty == 1 {
                    0
                } else {
                    self.get_volume()
                }
            }
            6 => {
                if self.settings.duty == 0 || self.settings.duty == 3 {
                    0
                } else {
                    self.get_volume()
                }
            }
            7 => {
                if self.settings.duty == 3 {
                    0
                } else {
                    self.get_volume()
                }
            }
            _ => unreachable!(),
        }
    }

    fn get_volume(&self) -> u8 {
        if self.mute || self.length_counter == 0 {
            0
        } else {
            if self.settings.constant_vol {
                self.settings.volume
            } else {
                self.envelope.get_volume()
            }
        }
    }

    pub(crate) fn write0(&mut self, value: u8) {
        self.settings.duty = (value & 0b11000000) >> 6;

        self.settings.length_counter_halt = (value & 0b0010_0000) != 0;

        self.settings.constant_vol = (value & 0b0001_0000) != 0;
        self.envelope.loop_flag = self.settings.constant_vol;

        self.settings.volume = value & 0b1111;
        self.envelope.start_parameter = self.settings.volume;
    }

    pub(crate) fn write1(&mut self, value: u8) {
        self.settings.sweep_enable = value & 0b1000_0000 != 0;
        self.settings.sweep_period = (value & 0b0111_0000) >> 4;
        self.settings.sweep_negate = value & 0b0000_1000 != 0;
        self.settings.sweep_shift = value & 0b0000_0111;
        self.sweeper_reload_flag = true;
    }

    pub(crate) fn write2(&mut self, value: u8) {
        let t_low = value as u16;
        let t_high = self.settings.timer & 0xFF00;
        self.settings.timer = t_high | t_low;
    }

    pub(crate) fn write3(&mut self, value: u8) {
        let t_low = self.settings.timer & 0x00FF;
        let t_high = ((value & 0b00000111) as u16) << 8;
        self.settings.timer = t_high | t_low;

        let length_counter_load = (value & 0b11111000) >> 3;
        self.length_counter = super::LENGTH_COUNTER_TABLE[length_counter_load as usize];

        self.cycle = 0;
        self.envelope.start_flag = true;
    }
}

#[derive(Clone)]
pub(crate) struct PulseSettings {
    pub duty: u8,
    pub constant_vol: bool,
    pub volume: u8,
    pub length_counter_halt: bool,
    pub sweep_enable: bool,
    pub sweep_period: u8,
    pub sweep_negate: bool,
    pub sweep_shift: u8,
    pub timer: u16,
}

impl PulseSettings {
    pub(crate) fn new() -> Self {
        Self {
            duty: 0,
            constant_vol: false,
            volume: 0,
            length_counter_halt: false,
            sweep_enable: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            timer: 0x3ff,
        }
    }
}
