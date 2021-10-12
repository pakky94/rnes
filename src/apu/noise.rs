use std::ops::BitXor;

use super::envelope::Envelope;

const PERIOD_LOOKUP_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];
pub(crate) struct Noise {
    length_counter_halt: bool,
    constant_volume: bool,
    volume: u8,
    mode_flag: bool,
    period: u16,
    cycle: u16,
    shift_register: u16,

    envelope: Envelope,

    pub length_counter: u8,
}

impl Noise {
    pub(crate) fn new() -> Self {
        Self {
            length_counter_halt: false,
            constant_volume: false,
            volume: 0,
            mode_flag: false,
            period: 0,
            cycle: 0,
            shift_register: 1,

            envelope: Envelope::new(),
            length_counter: 0,
        }
    }

    pub(crate) fn tick(&mut self, envelope_tick: bool, length_counter_tick: bool) {
        if self.cycle == 0 {
            self.tick_shift_register();
            self.cycle = self.period;
        } else {
            self.cycle -= 1;
        }

        if length_counter_tick {
            if !self.length_counter_halt {
                if self.length_counter != 0 {
                    self.length_counter -= 1;
                }
            }
        }

        if envelope_tick {
            self.envelope.tick();
        }
    }

    fn tick_shift_register(&mut self) {
        let bit0 = self.shift_register & 1 != 0;
        let second_bit = if self.mode_flag {
            // bit 6
            self.shift_register & 0b100_0000 != 0
        } else {
            // bit 2
            self.shift_register & 0b10 != 0
        };

        let feedback = bit0.bitxor(second_bit);
        let feedback_bit = if feedback {
            // bit 14 set
            0x4000
        } else {
            0
        };

        self.shift_register = feedback_bit | (self.shift_register >> 1);
    }

    pub(crate) fn next_value(&self) -> u8 {
        let bit0 = self.shift_register & 1 != 0;
        if bit0 {
            0
        } else {
            let env = self.get_volume();
            //eprintln!("{}", env);
            env
        }
    }

    fn get_volume(&self) -> u8 {
        if self.length_counter == 0 {
            0
        } else {
            self.envelope.get_volume()
        }
    }

    pub(crate) fn write0(&mut self, value: u8) {
        self.length_counter_halt = (value & 0b0010_0000) != 0;
        self.constant_volume = (value & 0b0001_0000) != 0;
        self.volume = value & 0b1111;
        self.envelope.start_parameter = self.volume;
    }

    pub(crate) fn write2(&mut self, value: u8) {
        self.mode_flag = (value & 0b1000_0000) != 0;
        let period_idx = (value & 0b0000_1111) as usize;
        self.period = PERIOD_LOOKUP_TABLE[period_idx];
    }

    pub(crate) fn write3(&mut self, value: u8) {
        self.length_counter = (value & 0b1111_1000) >> 3;
        self.envelope.start_flag = true;
    }
}
