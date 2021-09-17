use rnes::cartridge::Cartridge;
use rnes::cpu::Cpu;

pub fn cpu_from_vec(instr: Vec<u8>) -> Cpu {
    let mut cpu = Cpu::new();
    cpu.load_cartridge(cartridge_from_vec(instr));
    cpu.init();
    cpu
}

pub fn cartridge_from_vec(instr: Vec<u8>) -> Cartridge {
    let mut memory = vec![0; 0x10000];
    memory[0xFFFC] = 0x00;
    memory[0xFFFD] = 0xC0;

    for (i, byte) in instr.into_iter().enumerate() {
        memory[0xC000 + i] = byte
    }
    
    Cartridge::from_vec(memory)
}
