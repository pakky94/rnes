mod nrom;
pub(crate) use nrom::NROM;

mod mmc1;
pub(crate) use mmc1::MMC1;

pub(crate) trait Mapper {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn ppu_read(&self, address: u16, internal_vram: &[u8; 0x800]) -> u8;
    fn ppu_write(&mut self, address: u16, value: u8, internal_vram: &mut [u8; 0x800]);
    fn tick(&mut self);
}

pub(crate) enum Mirroring {
    Vertical,
    Horizontal,
    OneScreenLowerBank,
    OneScreenUpperBank,
}

pub(crate) struct Empty;
impl Mapper for Empty {
    fn read(&self, _address: u16) -> u8 {
        0
    }
    fn write(&mut self, _address: u16, _value: u8) {}
    fn tick(&mut self) {}

    fn ppu_read(&self, _address: u16, _internal_vram: &[u8; 0x800]) -> u8 {
        todo!()
    }

    fn ppu_write(&mut self, _address: u16, _value: u8, _internal_vram: &mut [u8; 0x800]) {
        todo!()
    }
}

pub(crate) struct FromVec(Vec<u8>);
impl Mapper for FromVec {
    fn read(&self, address: u16) -> u8 {
        self.0[address as usize]
    }

    fn write(&mut self, _address: u16, _value: u8) {}

    fn tick(&mut self) {}

    fn ppu_read(&self, _address: u16, _internal_vram: &[u8; 0x800]) -> u8 {
        todo!()
    }

    fn ppu_write(&mut self, _address: u16, _value: u8, _internal_vram: &mut [u8; 0x800]) {
        todo!()
    }
}

impl FromVec {
    pub(crate) fn new(memory: Vec<u8>) -> Self {
        Self(memory)
    }
}
