use std::fs::File;
use std::io::Read;

use crate::Cartridge;

pub fn read_rom(filename: &str) -> Result<Cartridge, String> {
    let mut header = [0u8; 16];

    let mut f = File::open(filename).map_err(|e| e.to_string())?;
    f.read_exact(&mut header).map_err(|e| e.to_string())?;

    //let mapper_low = (header[6] & 248) >> 4;
    //let mapper_high = header[7] & 248; // 4 highest bits
    //let mapper = mapper_high | mapper_low;

    let mut data = Vec::new();
    f.read_to_end(&mut data).map_err(|e| e.to_string())?;

    Ok(Cartridge::new(header, data))

    //
    //println!("mapper found: {}", mapper);
    //println!("banks: {}", banks.len());
    //match mapper {
    //0 => Cartridge::new(Mapper::NROM(banks.len() == 1), banks),
    //1 => Cartridge::new(Mapper::MMC1(0, banks.len() - 1), banks),
    //_ => unimplemented!("Unimplemented mapper {}", mapper),
    //}
}
