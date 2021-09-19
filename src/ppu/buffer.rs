pub struct Buffer {
    data: Box<[u8; 4 * 256 * 240]>,
}

impl Buffer {
    pub(crate) fn empty() -> Self {
        let data = [255; 4 * 256 * 240];
        Self {
            data: Box::new(data)
        }
    }

    pub fn get_data(&mut self) -> &mut [u8] {
        &mut self.data[..]
    }
}
