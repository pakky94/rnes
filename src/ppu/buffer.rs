pub struct Buffer {
    data: Box<[u8; 4 * 256 * 240]>,
}

impl Buffer {
    pub(crate) fn empty() -> Self {
        let data = [255; 4 * 256 * 240];
        Self {
            data: Box::new(data),
        }
    }

    pub fn get_data(&mut self) -> &mut [u8] {
        &mut self.data[..]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        self.data[4 * (x + 256 * y)] = 255;
        self.data[4 * (x + 256 * y) + 1] = b;
        self.data[4 * (x + 256 * y) + 2] = g;
        self.data[4 * (x + 256 * y) + 3] = r;
    }
}
