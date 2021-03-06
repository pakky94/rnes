mod mappers;

use mappers::Mapper;

const BANK_1_OFFSET: u16 = 0x8000;
const BANK_2_OFFSET: u16 = 0xC000;

pub struct Cartridge {
    header: [u8; 16],
    data: Vec<u8>,
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub(crate) fn new(header: [u8; 16], data: Vec<u8>) -> Self {
        let header_bac = header.clone();
        let data_bac = data.clone();
        let mapper_low = (header[6] & 248) >> 4;
        let mapper_high = header[7] & 248; // 4 highest bits
        let mapper = mapper_high | mapper_low;

        let mirroring_bit = header[6] & 1;
        let mirroring = if mirroring_bit == 0 {
            mappers::Mirroring::Horizontal
        } else {
            mappers::Mirroring::Vertical
        };

        let prg_rom_banks = header[4];
        let chr_rom_banks = header[5] * 2; // Treat banks as 4k banks

        let mut prg_banks = Vec::new();
        let mut chr_banks = Vec::new();

        let mut data_iter = data.into_iter();

        let mut buffer = [0u8; 0x4000];
        for _ in 0..prg_rom_banks {
            for i in 0..0x4000 {
                let b = data_iter.next().unwrap();
                buffer[i] = b;
            }
            prg_banks.push(buffer.clone());
        }

        let mut buffer = [0u8; 0x1000];
        for _ in 0..chr_rom_banks {
            for i in 0..0x1000 {
                let b = data_iter.next().unwrap();
                buffer[i] = b;
            }
            chr_banks.push(buffer.clone());
        }

        //let mut i = 0;
        //data.into_iter().for_each(|b| {
        //buffer[i] = b;
        //i += 1;
        //if i == 0x4000 {
        //i = 0;
        //prg_banks.push(buffer.clone());
        //}
        //});
        //if i > 0 {
        //prg_banks.push(buffer);
        //}

        match mapper {
            0 => {
                let mut chr_bank = [0u8; 0x2000];
                let mut i = 0;
                for j in 0..0x1000 {
                    chr_bank[i] = chr_banks[0][j];
                    i += 1;
                }
                for j in 0..0x1000 {
                    chr_bank[i] = chr_banks[1][j];
                    i += 1;
                }
                
                if prg_banks.len() == 1 {
                    Cartridge {
                        header: header_bac,
                        data: data_bac,
                        mapper: Box::new(mappers::NROM::new(
                            true,
                            [prg_banks[0], [0u8; 0x4000]],
                            if chr_rom_banks > 0 {
                                chr_bank
                            } else {
                                [0u8; 0x2000]
                            },
                            mirroring,
                        )),
                    }
                } else {
                    Cartridge {
                        header: header_bac,
                        data: data_bac,
                        mapper: Box::new(mappers::NROM::new(
                            false,
                            [prg_banks[0], prg_banks[1]],
                            if chr_rom_banks > 0 {
                                chr_bank
                            } else {
                                [0u8; 0x2000]
                            },
                            mirroring,
                        )),
                    }
                }
            }
            1 => {
                
                if chr_rom_banks == 0 {
                    chr_banks = vec![[0u8; 0x1000]; 32];
                    Cartridge {
                        header: header_bac,
                        data: data_bac,
                        mapper: Box::new(mappers::MMC1::new(prg_banks, chr_banks)),
                    }
                } else {
                    Cartridge {
                        header: header_bac,
                        data: data_bac,
                        mapper: Box::new(mappers::MMC1::new(prg_banks, chr_banks)),
                    }
                }
            }
            _ => unimplemented!("Unimplemented mapper {}", mapper),
        }
    }

    pub fn from_vec(memory: Vec<u8>) -> Self {
        Self {
            header: [0; 16],
            data: memory.clone(),
            mapper: Box::new(mappers::FromVec::new(memory)),
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            header: [1; 16],
            data: Vec::new(),
            mapper: Box::new(mappers::Empty {}),
        }
    }

    pub(crate) fn cpu_read(&self, address: u16) -> u8 {
        self.mapper.read(address)
    }
    pub(crate) fn cpu_write(&mut self, address: u16, value: u8) {
        self.mapper.write(address, value)
    }
    pub(crate) fn ppu_read(&self, address: u16, internal_vram: &[u8; 0x800]) -> u8 {
        self.mapper.ppu_read(address, internal_vram)
    }
    pub(crate) fn ppu_write(&mut self, address: u16, value: u8, internal_vram: &mut [u8; 0x800]) {
        self.mapper.ppu_write(address, value, internal_vram)
    }
    pub(crate) fn tick(&mut self) {
        self.mapper.tick()
    }
    pub(crate) fn get_save_data(&self) -> Vec<u8> {
        self.mapper.as_ref().get_save_ram()
    }
    pub(crate) fn set_save_data(&mut self, data: Vec<u8>) {
        self.mapper.as_mut().set_save_ram(data)
    }
    pub(crate) fn reset(&mut self) {
        match self.header[0] {
            0 => *self = Self::from_vec(self.data.clone()),
            1 => *self = Self::empty(),
            _ => *self = Self::new(self.header.clone(), self.data.clone()),
        }
    }
}
