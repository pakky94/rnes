pub(crate) struct Envelope {
    start_flag: bool,
    dividerloop_flag: bool,
    divider: u8,
    decay_level: u8,
    start_parameter: u8,
}

impl Envelope {
    pub(crate) fn new() -> Self {
        Self {
            
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

    pub(crate) fn get_volume(&self) {
        self.decay_level
    }
}
