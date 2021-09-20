use super::{super::{BANK_1_OFFSET, BANK_2_OFFSET}, Mapper, Mirroring};

pub(crate) struct NROM {
    one_bank: bool,
    prg_banks: [[u8; 0x4000]; 2],
    chr_bank: [u8; 0x2000],
    mirroring: Mirroring,
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

    fn ppu_read(&self, address: u16, internal_vram: &[u8; 0x800]) -> u8 {
        if address < 0x2000 {
            self.chr_bank[address as usize]
        } else if address < 0x3F00 {
            let address = self.map_nametable_address(address);
            internal_vram[address as usize]
        } else {
            unreachable!();
        }
    }

    fn ppu_write(&mut self, address: u16, value: u8, internal_vram: &mut [u8; 0x800]) {
        if address < 0x2000 {
            self.chr_bank[address as usize] = value; // treat CHR-ROM as RAM
        } else if address < 0x3F00 {
            //let address = address % 0x800;
            let address = self.map_nametable_address(address);
            internal_vram[address as usize] = value;
        } else {
            unreachable!();
        }
    }
}

impl NROM {
    pub(crate) fn new(one_bank: bool, prg_banks: [[u8; 0x4000]; 2], chr_bank: [u8; 0x2000], mirroring: Mirroring) -> Self {
        Self {
            one_bank,
            prg_banks,
            chr_bank,
            mirroring,
        }
    }

    fn map_nametable_address(&self, address: u16) -> usize {
        if address < 0x2000 {
            address as usize
        } else if address < 0x3F00 {
            match self.mirroring {
                Mirroring::Vertical => {
                    let high_addr = address & 0xC00;
                    let low_addr = address % 0x400;
                    match high_addr {
                        0x000 => {
                            (0x000 | low_addr) as usize
                        }
                        0x400 => {
                            (0x400 | low_addr) as usize
                        }
                        0x800 => {
                            (0x000 | low_addr) as usize
                        }
                        0xC00 => {
                            (0x400 | low_addr) as usize
                        }
                        _ => unreachable!(),
                    }
                }
                Mirroring::Horizontal => {
                    let high_addr = address & 0xC00;
                    let low_addr = address % 0x400;
                    match high_addr {
                        0x000 => {
                            (0x000 | low_addr) as usize
                        }
                        0x400 => {
                            (0x000 | low_addr) as usize
                        }
                        0x800 => {
                            (0x400 | low_addr) as usize
                        }
                        0xC00 => {
                            (0x400 | low_addr) as usize
                        }
                        _ => unreachable!(),
                    }
                }

            }
        } else {
            unreachable!();
        }
    }
}
