pub(crate) struct Envelope {
    pub start_flag: bool,
    pub loop_flag: bool,
    pub start_parameter: u8,

    divider: u8,
    decay_level: u8,
}

impl Envelope {
    pub(crate) fn new() -> Self {
        Self {
            start_flag: false,
            loop_flag: false,
            start_parameter: 0,

            divider: 0,
            decay_level: 0,
        }
    }

    pub(crate) fn tick(&mut self) {
        if self.start_flag {
            self.decay_level = 15;
            self.divider = self.start_parameter;
            self.start_flag = false;
        } else {
            if self.divider == 0 {
                self.divider = self.start_parameter;
                if self.decay_level != 0 {
                    self.decay_level -= 1;
                } else {
                    if self.loop_flag {
                        self.decay_level = 15;
                    }
                }
            } else {
                self.divider -= 1;
            }
        }
    }

    pub(crate) fn get_volume(&self) -> u8 {
        self.decay_level
    }
}
