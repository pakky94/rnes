#[derive(Clone, Debug)]
pub struct InputData {
    pub a: bool,
    pub b: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub start: bool,
    pub select: bool,
}

impl InputData {
    pub fn new() -> Self {
        Self {
            a: false,
            b: false,
            up: false,
            down: false,
            left: false,
            right: false,
            start: false,
            select: false,
        }
    }
}

pub(crate) struct Controller {
    input_data: InputData,
    last_write: bool,
    buffer: InputData,
    current_tick: usize,
}

impl Controller {
    pub(crate) fn new() -> Self {
        Self {
            input_data: InputData::new(),
            last_write: false,
            buffer: InputData::new(),
            current_tick: 0,
        }
    }

    pub(crate) fn set_input(&mut self, input: InputData) {
        self.input_data = input;
    }

    pub(crate) fn write_to(&mut self, val: u8) {
        self.last_write = val % 2 == 1;
    }

    pub(crate) fn tick(&mut self) {
        if self.last_write {
            self.buffer = self.input_data.clone();
            self.current_tick = 0;
        }
    }

    pub(crate) fn read_from(&mut self) -> u8 {
        match self.current_tick {
            0 => {
                self.current_tick += 1;
                if self.buffer.a {
                    1
                } else {
                    0
                }
            }
            1 => {
                self.current_tick += 1;
                if self.buffer.b {
                    1
                } else {
                    0
                }
            }
            2 => {
                self.current_tick += 1;
                if self.buffer.select {
                    1
                } else {
                    0
                }
            }
            3 => {
                self.current_tick += 1;
                if self.buffer.start {
                    1
                } else {
                    0
                }
            }
            4 => {
                self.current_tick += 1;
                if self.buffer.up {
                    1
                } else {
                    0
                }
            }
            5 => {
                self.current_tick += 1;
                if self.buffer.down {
                    1
                } else {
                    0
                }
            }
            6 => {
                self.current_tick += 1;
                if self.buffer.left {
                    1
                } else {
                    0
                }
            }
            7 => {
                self.current_tick += 1;
                if self.buffer.right {
                    1
                } else {
                    0
                }
            }
            8 => 0,
            _ => unreachable!(),
        }
    }
}
