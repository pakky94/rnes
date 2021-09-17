use super::{
    super::{BANK_1_OFFSET, BANK_2_OFFSET},
    Mapper,
};

pub(crate) struct NROM {
    one_bank: bool,
    prg_banks: [[u8; 0x4000]; 2],
    chr_bank: [u8; 0x2000],
}

impl Mapper for NROM {
    fn read(&self, address: u16) -> u8 {
        if address >= BANK_2_OFFSET {
            if self.one_bank {
                self.prg_banks[0][(address - BANK_2_OFFSET) as usize]
            } else {
                self.prg_banks[1][(address - BANK_2_OFFSET) as usize]
            }
        } else if address >= BANK_1_OFFSET {
            self.prg_banks[0][(address - BANK_1_OFFSET) as usize]
        } else {
            unreachable!()
        }
    }

    fn write(&mut self, _address: u16, _value: u8) {}
    fn tick(&mut self) {}
}

impl NROM {
    pub(crate) fn new(one_bank: bool, prg_banks: [[u8; 0x4000]; 2], chr_bank: [u8; 0x2000]) -> Self {
        Self {
            one_bank,
            prg_banks,
            chr_bank,
        }
    }
}
