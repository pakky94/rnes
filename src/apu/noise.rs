const PERIOD_LOOKUP_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];
pub(crate) struct Noise {
    length_counter_halt: bool,
    constant_volume: bool,
    volume: u8,
    mode_flag: bool,
    period: u16,

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
            length_counter: 0,
        }
    }

    pub(crate) fn tick(&mut self) {}

    fn get_volume(&self) -> u8 {
        if self.length_counter == 0 {
            0
        } else {
            if self.constant_volume {
                self.volume
            } else {
                //self.envelope_decay_level
                todo!()
            }
        }
    }

    pub(crate) fn write0(&mut self, value: u8) {
        self.length_counter_halt = (value & 0b0010_0000) != 0;
        self.constant_volume = (value & 0b0001_0000) != 0;
        self.volume = value & 0b1111;
    }

    pub(crate) fn write2(&mut self, value: u8) {
        self.mode_flag = (value & 0b1000_0000) != 0;
        let period_idx = (value & 0b0000_1111) as usize;
        self.period = PERIOD_LOOKUP_TABLE[period_idx];
    }

    pub(crate) fn write3(&mut self, value: u8) {
        self.length_counter = (value & 0b1111_1000) >> 3;
        // TODO: envelope restart
    }
}
