pub(crate) struct SpriteUnit {
    high_color: u8,
    low_color: u8,
    pub attributes: u8,
    pub position: u8,
    pub tile_number: u8,
    pub y: u8,
}

impl SpriteUnit {
    pub(crate) fn new() -> Self {
        Self {
            high_color: 0,
            low_color: 0,
            attributes: 0,
            position: 0,
            tile_number: 0,
            y: 0,
        }
    }

    pub(crate) fn units() -> [Self; 8] {
        [
            Self::new(),
            Self::new(),
            Self::new(),
            Self::new(),
            Self::new(),
            Self::new(),
            Self::new(),
            Self::new(),
        ]
    }

    pub(crate) fn set_transparent(&mut self) {
        self.high_color = 0;
        self.low_color = 0;
    }

    pub(crate) fn get_low_data_address(
        &self,
        screen_y: usize,
        pattern_table: u16,
        is_8by8: bool,
    ) -> u16 {
        if is_8by8 {
            let offset = self.get_y_on_current_line(screen_y) as u16;
            let tile_address = (self.tile_number as u16) << 4;
            pattern_table | tile_address | offset
        } else {
            todo!()
        }
    }

    pub(crate) fn get_high_data_address(
        &self,
        screen_y: usize,
        pattern_table: u16,
        is_8by8: bool,
    ) -> u16 {
        self.get_low_data_address(screen_y, pattern_table, is_8by8) + 8
    }

    pub(crate) fn set_low_color(&mut self, color: u8) {
        if self.is_flipped_hori() {
            self.low_color = flip_byte(color);
        } else {
            self.low_color = color;
        }
    }

    pub(crate) fn set_high_color(&mut self, color: u8) {
        if self.is_flipped_hori() {
            self.high_color = flip_byte(color);
        } else {
            self.high_color = color;
        }
    }

    pub(crate) fn shift_left(&mut self) {
        if self.position == 0 {
            self.low_color = self.low_color << 1;
            self.high_color = self.high_color << 1;
        } else {
            self.position = self.position.wrapping_sub(1);
        }
    }

    pub(crate) fn get_color(&self) -> u8 {
        let palette = self.attributes & 0b00000011;
        let palette = palette << 2;
        if self.position == 0 {
            palette | ((self.low_color & 128) >> 7) | ((self.high_color & 128) >> 6)
        } else {
            0
        }
    }

    pub(crate) fn is_foreground(&self) -> bool {
        self.attributes >> 5 & 1 == 0
    }

    fn is_flipped_hori(&self) -> bool {
        self.attributes >> 6 & 1 == 1
    }

    fn is_flipped_vert(&self) -> bool {
        self.attributes >> 7 & 1 == 1
    }

    fn get_y_on_current_line(&self, screen_y: usize) -> u8 {
        if self.is_flipped_vert() {
            8 - ((screen_y as u8) - self.y)
        } else {
            (screen_y as u8) - self.y
        }
    }
}

fn flip_byte(b: u8) -> u8 {
    let mut b = b;
    b = (b & 0xF0) >> 4 | (b & 0x0F) << 4;
    b = (b & 0xCC) >> 2 | (b & 0x33) << 2;
    b = (b & 0xAA) >> 1 | (b & 0x55) << 1;
    b
}
