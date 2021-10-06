pub(crate) struct Triangle {
    control_flag: bool,
    linear_counter_reload_value: u8,
    linear_counter_reload_flag: bool,
    timer: u16,
    length_counter_load: u8,

    cycle: u16,
    out_value: u8,
    crescent: bool,
    pub length_counter: u8,
    pub linear_counter: u8,
}

impl Triangle {
    pub(crate) fn new() -> Self {
        Self {
            control_flag: true,
            linear_counter_reload_value: 0,
            linear_counter_reload_flag: false,
            timer: 0,
            length_counter_load: 0,

            cycle: 0,
            out_value: 0,
            crescent: true,
            length_counter: 0,
            linear_counter: 0,
        }
    }

    pub(crate) fn tick(&mut self, linear_counter: bool, length_counter_tick: bool) {
        if self.cycle == 0 {
            self.cycle = self.timer;
            //if self.timer != 0 {
            self.advance_out_value();
            //}
        } else {
            self.cycle -= 1;
        }

        if !self.control_flag && length_counter_tick {
            if self.length_counter != 0 {
                self.length_counter -= 1;
            }
        }

        if linear_counter {
            if self.linear_counter_reload_flag {
                self.linear_counter = self.linear_counter_reload_value;
            }
            if !self.control_flag {
                self.linear_counter_reload_flag = false;
            }
        }
    }

    fn advance_out_value(&mut self) {
        if self.crescent {
            if self.out_value == 15 {
                self.crescent = false;
            } else {
                self.out_value += 1;
            }
        } else {
            if self.out_value == 0 {
                self.crescent = true;
            } else {
                self.out_value -= 1;
            }
        }
    }

    pub(crate) fn next_value(&mut self) -> u8 {
        if self.linear_counter != 0 && self.length_counter != 0 {
            self.out_value
        } else {
            //eprintln!(
            //"out_val: {} linear_counter: {}    length_counter: {}",
            //self.out_value, self.linear_counter, self.length_counter
            //);
            0
        }
    }

    pub(crate) fn write0(&mut self, value: u8) {
        self.control_flag = (value & 0b10000000) != 0;
        self.linear_counter_reload_value = value & 0b0111_1111;
    }

    pub(crate) fn write1(&mut self, _value: u8) {}

    pub(crate) fn write2(&mut self, value: u8) {
        let t_low = value as u16;
        let t_high = self.timer & 0xFF00;
        self.timer = t_high | t_low;
    }

    pub(crate) fn write3(&mut self, value: u8) {
        let t_low = self.timer & 0x00FF;
        let t_high = ((value & 0b00000111) as u16) << 8;
        self.timer = t_high | t_low;

        self.length_counter_load = (value & 0b11111000) >> 3;

        self.length_counter = super::LENGTH_COUNTER_TABLE[self.length_counter_load as usize];
        self.linear_counter_reload_flag = true;
    }
}
