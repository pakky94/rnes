pub struct Buffer {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

impl Buffer {
    pub(crate) fn empty(width: usize, height: usize) -> Self {
        let data = vec![255; 4 * width * height];
        Self {
            width,
            height,
            data,
        }
    }

    pub fn get_data(&mut self) -> &mut [u8] {
        &mut self.data[..]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        self.data[4 * (x + self.width * y)] = 255;
        self.data[4 * (x + self.width * y) + 1] = b;
        self.data[4 * (x + self.width * y) + 2] = g;
        self.data[4 * (x + self.width * y) + 3] = r;
    }
}
