use super::{
    super::{BANK_1_OFFSET, BANK_2_OFFSET},
    Mapper, Mirroring,
};
use crate::cpu::addresses::{EXPANSION_ROM, PRG_ROM_LOWER, SAVE_RAM};

pub(crate) struct MMC1 {
    save_ram: [u8; 0x2000],
    lower_bank: usize,
    higher_bank: usize,
    banks: Vec<[u8; 0x4000]>,
    lower_chr: usize,
    higher_chr: usize,
    chr_banks: Vec<[u8; 0x1000]>,
    load_register: u8,
    written_last_cycle: usize,
    prg_rom_switch_mode: PrgRomSwitchMode,
    chr_rom_switch_mode: ChrRomSwitchMode,
    mirroring: Mirroring,
}

enum PrgRomSwitchMode {
    Switch32k,
    FirstFixed,
    LastFixed,
}

enum ChrRomSwitchMode {
    Switch8k,
    SwitchSeparate,
}

impl Mapper for MMC1 {
    fn read(&self, address: u16) -> u8 {
        if address >= BANK_2_OFFSET {
            self.banks[self.higher_bank][(address - BANK_2_OFFSET) as usize]
        } else if address >= BANK_1_OFFSET {
            self.banks[self.lower_bank][(address - BANK_1_OFFSET) as usize]
        } else if address >= SAVE_RAM {
            // PRG RAM
            self.save_ram[(address - SAVE_RAM) as usize]
        } else {
            unreachable!()
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if address >= BANK_1_OFFSET {
            if self.written_last_cycle > 0 {
                return;
            }
            self.written_last_cycle = 1;

            if value >= 128 {
                self.clear_load_register();
            } else {
                let low_bit = value & 1;
                self.load_register = (self.load_register >> 1) | (low_bit << 5);
                if self.load_register % 2 == 1 {
                    // load_register full
                    let load_register = self.load_register & 0b00111110;
                    let load_register = load_register >> 1;
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
        } else {
            // PRG RAM
            self.save_ram[(address - SAVE_RAM) as usize] = value;
        }
    }

    fn tick(&mut self) {
        if self.written_last_cycle == 1 {
            self.written_last_cycle += 1;
        } else if self.written_last_cycle > 1 {
            self.written_last_cycle = 0;
        }
    }

    fn ppu_read(&self, address: u16, internal_vram: &[u8; 0x800]) -> u8 {
        let address = address % 0x4000;
        if address < 0x1000 {
            let val = self.chr_banks[self.lower_chr][address as usize];
            //println!("reading: {:#06x}, {}", address, val);
            val
        } else if address < 0x2000 {
            self.chr_banks[self.higher_chr][(address as usize) & !0x1000]
        } else if address < 0x3F00 {
            let address = self.map_nametable_address(address);
            internal_vram[address as usize]
        } else {
            unreachable!();
        }
    }

    fn ppu_write(&mut self, address: u16, value: u8, internal_vram: &mut [u8; 0x800]) {
        let address = address % 0x4000;
        if address < 0x1000 {
            self.chr_banks[self.lower_chr][address as usize] = value; // treat CHR-ROM as RAM
                                                                      //println!("writing: {:#06x}, {}", address, value);
        } else if address < 0x2000 {
            self.chr_banks[self.higher_chr][(address as usize) & !0x1000] = value;
        // treat CHR-ROM as RAM
        } else if address < 0x3F00 {
            let address_mapped = self.map_nametable_address(address);
            println!(
                "writing {:#04X} to nametable from {:#06X} to {:#06X}",
                value, address, address_mapped
            );
            internal_vram[address_mapped as usize] = value;
        } else {
            unreachable!();
        }
    }

    fn get_save_ram(&self) -> Vec<u8> {
        self.save_ram.iter().cloned().collect()
    }

    fn set_save_ram(&mut self, data: Vec<u8>) {
        for (i, x) in data.into_iter().take(0x2000).enumerate() {
            self.save_ram[i] = x;
        }
    }
}

impl MMC1 {
    pub(crate) fn new(banks: Vec<[u8; 0x4000]>, chr_banks: Vec<[u8; 0x1000]>) -> Self {
        let last_bank = banks.len() - 1;
        Self {
            lower_bank: 0,
            higher_bank: last_bank,
            banks,
            lower_chr: 0,
            higher_chr: 1,
            chr_banks,
            load_register: 1,
            written_last_cycle: 0,
            //prg_rom_switch_mode: PrgRomSwitchMode::Switch32k,
            prg_rom_switch_mode: PrgRomSwitchMode::LastFixed,
            mirroring: Mirroring::Vertical,
            chr_rom_switch_mode: ChrRomSwitchMode::Switch8k,
            save_ram: [0; 0x2000]
        }
    }

    fn write_control(&mut self, address: u8, value: u8) {
        println!("addr: {:#010b}, val: {:#010b}", address, value);
        match address {
            0 => {
                let prg_rom_bank_mode = (value & 12) >> 2;
                match value & 0b11 {
                    0 => {
                        self.mirroring = Mirroring::OneScreenLowerBank;
                        println!("switched to one-screen lower bank mirroring");
                    }
                    1 => {
                        self.mirroring = Mirroring::OneScreenUpperBank;
                        println!("switched to one-screen upper bank mirroring");
                    }
                    2 => {
                        self.mirroring = Mirroring::Vertical;
                        println!("switched to vertical mirroring");
                    }
                    3 => {
                        self.mirroring = Mirroring::Horizontal;
                        println!("switched to horizontal mirroring");
                    }
                    _ => unreachable!(),
                }
                match prg_rom_bank_mode {
                    0 | 1 => {
                        self.prg_rom_switch_mode = PrgRomSwitchMode::Switch32k;
                        //self.lower_bank = 0;
                        //self.higher_bank = 1;
                        println!("switched to 32k PRG ROM banks");
                    }
                    2 => {
                        self.prg_rom_switch_mode = PrgRomSwitchMode::FirstFixed;
                        self.lower_bank = 0;
                        println!("switched to first fixed PRG ROM bank");
                    }
                    3 => {
                        self.prg_rom_switch_mode = PrgRomSwitchMode::LastFixed;
                        self.higher_bank = self.banks.len() - 1;
                        println!("switched to last fixed PRG ROM bank");
                    }
                    _ => unreachable!(),
                }
                match value & 0x10 {
                    0x00 => {
                        self.chr_rom_switch_mode = ChrRomSwitchMode::Switch8k;
                        println!("switched to 8k CHR ROM banks");
                    }
                    0x10 => {
                        self.chr_rom_switch_mode = ChrRomSwitchMode::SwitchSeparate;
                        println!("switched to separate CHR ROM banks");
                    }
                    _ => unreachable!(),
                }
            }
            1 => match self.chr_rom_switch_mode {
                ChrRomSwitchMode::Switch8k => {
                    let lower_bank = value & 0x1E;
                    let higher_bank = lower_bank | 1;
                    self.lower_chr = lower_bank as usize % self.chr_banks.len();
                    self.higher_chr = higher_bank as usize % self.chr_banks.len();
                }
                ChrRomSwitchMode::SwitchSeparate => {
                    self.lower_chr = (value as usize) % self.chr_banks.len();
                    println!("switched lower CHR bank to bank {}", self.higher_bank);
                }
            },
            2 => match self.chr_rom_switch_mode {
                ChrRomSwitchMode::Switch8k => {
                    println!("Ignored switch of higher CHR bank");
                }
                ChrRomSwitchMode::SwitchSeparate => {
                    self.higher_chr = (value as usize) % self.chr_banks.len();
                    println!("switched higher CHR bank to bank {}", self.higher_bank);
                }
            },
            3 => {
                let bank = value & 15;
                match self.prg_rom_switch_mode {
                    PrgRomSwitchMode::Switch32k => {
                        let low_bank = (bank & 14) % (self.banks.len() as u8);
                        self.lower_bank = low_bank as usize;
                        self.higher_bank = (low_bank + 1) as usize;
                        println!(
                            "switched to banks {} and {}",
                            self.lower_bank, self.higher_bank
                        );
                    }
                    PrgRomSwitchMode::FirstFixed => {
                        self.higher_bank = (bank as usize) % self.banks.len();
                        println!("switched higher bank to bank {}", self.higher_bank);
                    }
                    PrgRomSwitchMode::LastFixed => {
                        self.lower_bank = (bank as usize) % self.banks.len();
                        println!(
                            "switched lower bank to bank {} out of {} banks",
                            self.lower_bank,
                            self.banks.len()
                        );
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn clear_load_register(&mut self) {
        println!("cleared MMC1 load register");
        self.load_register = 0b100000;
    }

    fn map_nametable_address(&self, address: u16) -> usize {
        if address < 0x2000 {
            //address as usize
            unreachable!()
        } else if address < 0x3F00 {
            match self.mirroring {
                Mirroring::Vertical => {
                    let high_addr = address & 0xC00;
                    let low_addr = address % 0x400;
                    match high_addr {
                        0x000 => (0x000 | low_addr) as usize,
                        0x400 => (0x400 | low_addr) as usize,
                        0x800 => (0x000 | low_addr) as usize,
                        0xC00 => (0x400 | low_addr) as usize,
                        _ => unreachable!(),
                    }
                }
                Mirroring::Horizontal => {
                    let high_addr = address & 0xC00;
                    let low_addr = address % 0x400;
                    match high_addr {
                        0x000 => (0x000 | low_addr) as usize,
                        0x400 => (0x000 | low_addr) as usize,
                        0x800 => (0x400 | low_addr) as usize,
                        0xC00 => (0x400 | low_addr) as usize,
                        _ => unreachable!(),
                    }
                }
                Mirroring::OneScreenLowerBank => (address % 0x400) as usize,
                Mirroring::OneScreenUpperBank => ((address % 0x400) | 0x400) as usize,
            }
        } else {
            unreachable!();
        }
    }
}
