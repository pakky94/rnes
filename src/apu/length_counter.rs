pub(crate) struct LengthCounter {
    pub length_counter: u8,
}

impl LengthCounter {
    pub(crate) fn new() -> Self {
        Self {
            length_counter: 0,
        }
    }

    pub(crate) fn tick(&mut self) {
        if self.length_counter != 0 {
            self.length_counter -= 1;
        }
    }
}
