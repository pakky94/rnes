use super::{
    super::{BANK_1_OFFSET, BANK_2_OFFSET},
    Mapper,
};

pub(crate) struct MMC1 {
    lower_bank: usize,
    higher_bank: usize,
    banks: Vec<[u8; 0x4000]>,
    load_register: u8,
    written_last_cycle: usize,
    prg_rom_switch_mode: PrgRomSwitchMode,
}

enum PrgRomSwitchMode {
    Switch32k,
    FirstFixed,
    LastFixed,
}

impl Mapper for MMC1 {
    fn read(&self, address: u16) -> u8 {
        if address >= BANK_2_OFFSET {
            self.banks[self.higher_bank][(address - BANK_2_OFFSET) as usize]
        } else if address >= BANK_1_OFFSET {
            self.banks[self.lower_bank][(address - BANK_1_OFFSET) as usize]
        } else {
            unreachable!()
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if self.written_last_cycle > 0 {
            return;
        }
        self.written_last_cycle = 1;

        if value >= 128 {
            self.clear_load_register();
        } else {
            let low_bit = value & 1;
            self.load_register = (self.load_register << 1) | low_bit;
            if self.load_register > 0b00100000 {
                // load_register full
                let load_register = self.load_register & 0b00011111;
                //println!("address:      {:#18b}", address);
                //println!("mask:         {:#18b}", 0x6000);
                let write_address = address & 0x6000; // get only bit 13 and 14 of the address
                //println!("write_a:      {:#18b}", write_address);
                let control_address = (write_address >> 13) as u8;
                //println!("shifted mask: {:#18b}", 0x6000 >> 13);
                //println!("shifted addr: {:#18b}", control_address);

                self.write_control(control_address, load_register);

                //panic!();

                self.clear_load_register();
            }
        }
    }

    fn tick(&mut self) {
        if self.written_last_cycle == 1 {
            self.written_last_cycle += 1;
        } else if self.written_last_cycle > 1 {
            self.written_last_cycle = 0;
        }
    }

    fn ppu_read(&self, _address: u16, _internal_vram: &[u8; 0x800]) -> u8 {
        todo!()
    }

    fn ppu_write(&mut self, _address: u16, _value: u8, _internal_vram: &mut [u8; 0x800]) {
        todo!()
    }
}

impl MMC1 {
    pub(crate) fn new(banks: Vec<[u8; 0x4000]>) -> Self {
        Self {
            lower_bank: 0,
            higher_bank: 1,
            banks,
            load_register: 1,
            written_last_cycle: 0,
            prg_rom_switch_mode: PrgRomSwitchMode::Switch32k,
        }
    }

    fn write_control(&mut self, address: u8, value: u8) {
        println!("addr: {:#010b}, val: {:#010b}", address, value);
        match address {
            0 => {
                let prg_rom_bank_mode = (value & 12) >> 2;
                match prg_rom_bank_mode {
                    0 | 1 => {
                        self.prg_rom_switch_mode = PrgRomSwitchMode::Switch32k;
                        println!("switched to 32k PRG ROM banks");
                    }
                    2 => {
                        self.prg_rom_switch_mode = PrgRomSwitchMode::FirstFixed;
                        println!("switched to first fixed PRG ROM bank");
                    }
                    3 => {
                        self.prg_rom_switch_mode = PrgRomSwitchMode::LastFixed;
                        println!("switched to last fixed PRG ROM bank");
                    }
                    _ => unreachable!(),
                }
            }
            1 => {
                // TODO:
            }
            2 => {
                // TODO:
            }
            3 => {
                let bank = value & 15;
                match self.prg_rom_switch_mode {
                    PrgRomSwitchMode::Switch32k => {
                        let low_bank = bank & 14;
                        self.lower_bank = low_bank as usize;
                        self.higher_bank = (low_bank + 1) as usize;
                        println!("switched to banks {} and {}", self.lower_bank, self.higher_bank);
                    }
                    PrgRomSwitchMode::FirstFixed => {
                        self.higher_bank = (bank as usize) % self.banks.len();
                        println!("switched higher bank to bank {}", self.higher_bank);
                    }
                    PrgRomSwitchMode::LastFixed => {
                        self.lower_bank = (bank as usize) % self.banks.len();
                        println!("switched lower bank to bank {}", self.lower_bank);
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn clear_load_register(&mut self) {
        //println!("cleared MMC1 load register");
        self.load_register = 1;
    }
}
