use crate::cpu::CpuStatus;
use std::{collections::VecDeque, io::Write};

#[derive(Clone)]
struct LoggedInstr {
    address: u16,
    cpu_status: Option<CpuStatus>,
    data: Vec<u8>,
    target_address: Option<(u16, u8)>,
    cycle: usize,
}

#[derive(Clone)]
enum LoggedEvent {
    LoggedInstr(LoggedInstr),
    NMI,
}

pub struct Logger {
    instructions: VecDeque<LoggedEvent>,
    is_logging: bool,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            instructions: VecDeque::new(),
            is_logging: false,
        }
    }

    pub fn log_nmi(&mut self) {
        self.instructions.push_back(LoggedEvent::NMI);
    }

    pub fn start_new_instr(&mut self, address: u16, opcode: u8, cycle: usize) {
        self.instructions
            .push_back(LoggedEvent::LoggedInstr(LoggedInstr {
                address,
                cpu_status: None,
                data: vec![opcode],
                target_address: None,
                cycle,
            }))
    }

    pub fn set_proc_status(&mut self, status: CpuStatus) {
        if let Some(LoggedEvent::LoggedInstr(ref mut last_instr)) = self.instructions.back_mut() {
            last_instr.cpu_status = Some(status);
        }
    }

    pub fn add_data_to_last_instr(&mut self, data: u8) {
        if let Some(LoggedEvent::LoggedInstr(ref mut last_instr)) = self.instructions.back_mut() {
            last_instr.data.push(data);
        }
    }

    pub fn set_last_target_address(&mut self, address: u16, value: u8) {
        if let Some(LoggedEvent::LoggedInstr(ref mut last_instr)) = self.instructions.back_mut() {
            last_instr.target_address = Some((address, value));
        }
    }

    pub fn enable_logging(&mut self) {
        self.is_logging = true;
    }

    pub fn disable_logging(&mut self) {
        self.is_logging = false;
    }

    pub fn is_logging(&self) -> bool {
        self.is_logging
    }

    pub fn get_log(&self) -> String {
        let mut complete = String::new();
        for event in self.instructions.iter() {
            match event {
                LoggedEvent::LoggedInstr(instr) => {
                    complete.push_str(&Self::print_instruction(instr));
                    complete.push('\n');
                }
                LoggedEvent::NMI => complete.push_str("------ NMI ------\n"),
            }
        }

        complete
    }

    fn print_instruction(instr: &LoggedInstr) -> String {
        let mut out = String::with_capacity(90);
        out.push_str(&format!("{:04X}  ", instr.address));
        for byte in instr.data.iter() {
            out.push_str(&format!("{:02X} ", byte));
        }
        //out.push_str(&format!("{:02X} ", instr.data[0]));

        while out.len() < 16 {
            out.push(' ');
        }
        out.push_str(opcode_to_mnemonic(instr.data[0]));
        out.push(' ');

        if let Some((addr, _val)) = instr.target_address.as_ref() {
            out.push_str(&format!("${:04X}", addr,));
        }

        while out.len() < 48 {
            out.push(' ');
        }
        if let Some(cpu_status) = instr.cpu_status.as_ref() {
            out.push_str(&format!(
                "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
                cpu_status.accumulator,
                cpu_status.x,
                cpu_status.y,
                cpu_status.p,
                cpu_status.stack_pointer,
            ));
        }

        out.push_str(&format!("  cycle: {}", instr.cycle));

        out
    }

    pub fn write_log(&self, filename: &str) {
        let out = self.get_log();
        let mut file = std::fs::File::create(filename).expect("create failed");
        file.write_all(out.as_bytes()).expect("write failed");
    }

    pub fn clear(&mut self) {
        self.instructions.clear();
    }

    pub fn print_log(&self) {
        println!("{}", self.get_log())
    }
}

fn opcode_to_mnemonic(opcode: u8) -> &'static str {
    match opcode {
        0x0B | 0x2B => "AAC",
        0x87 | 0x97 | 0x83 | 0x8F => "AAX",
        0x61 | 0x65 | 0x69 | 0x6D | 0x71 | 0x75 | 0x79 | 0x7D => "ADC",
        0x4B => "ALR",
        0x21 | 0x25 | 0x29 | 0x2D | 0x31 | 0x35 | 0x39 | 0x3D => "AND",
        0x6B => "ARR",
        0x06 | 0x0A | 0x0E | 0x16 | 0x1E => "ASL",
        0xCB => "AXS",
        0x24 | 0x2C => "BIT",
        0x90 => "BCC",
        0xB0 => "BCS",
        0xF0 => "BEQ",
        0x30 => "BMI",
        0xD0 => "BNE",
        0x10 => "BPL",
        0x50 => "BVC",
        0x70 => "BVS",
        0x00 => "BRK",
        0x18 => "CLC",
        0xD8 => "CLD",
        0x58 => "CLI",
        0xB8 => "CLV",
        0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => "CMP",
        0xE0 | 0xE4 | 0xEC => "CPX",
        0xC0 | 0xC4 | 0xCC => "CPY",
        0xC7 | 0xD7 | 0xCF | 0xDF | 0xDB | 0xC3 | 0xD3 => "DCP",
        0xC6 | 0xD6 | 0xCE | 0xDE => "DEC",
        0xCA => "DEX",
        0x88 => "DEY",
        0x04 | 0x14 | 0x34 | 0x44 | 0x54 | 0x64 | 0x74 | 0x80 | 0x82 | 0x89 | 0xC2 | 0xD4
        | 0xE2 | 0xF4 => "DOP",
        0x41 | 0x45 | 0x49 | 0x4D | 0x51 | 0x55 | 0x59 | 0x5D => "EOR",
        0xE6 | 0xF6 | 0xEE | 0xFE => "INC",
        0xE8 => "INX",
        0xC8 => "INY",
        0xE7 | 0xF7 | 0xEF | 0xFF | 0xFB | 0xE3 | 0xF3 => "ISC",
        0xA7 | 0xB7 | 0xAF | 0xBF | 0xA3 | 0xB3 | 0xAB => "LAX",
        0xA1 | 0xA5 | 0xA9 | 0xAD | 0xB1 | 0xB5 | 0xB9 | 0xBD => "LDA",
        0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => "LDX",
        0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => "LDY",
        0x46 | 0x4A | 0x4E | 0x56 | 0x5E => "LSR",
        0x4C | 0x6C => "JMP",
        0x20 => "JSR",
        0xEA | 0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => "NOP",
        0x01 | 0x05 | 0x09 | 0x0D | 0x11 | 0x15 | 0x19 | 0x1D => "ORA",
        0x48 => "PHA",
        0x08 => "PHP",
        0x68 => "PLA",
        0x28 => "PLP",
        0x27 | 0x37 | 0x2F | 0x3F | 0x3B | 0x23 | 0x33 => "RLA",
        0x67 | 0x77 | 0x6F | 0x7F | 0x7B | 0x63 | 0x73 => "RRA",
        0x26 | 0x2A | 0x2E | 0x36 | 0x3E => "ROL",
        0x66 | 0x6A | 0x6E | 0x76 | 0x7E => "ROR",
        0x40 => "RTI",
        0x60 => "RTS",
        0xE1 | 0xE5 | 0xE9 | 0xEB | 0xED | 0xF1 | 0xF5 | 0xF9 | 0xFD => "SBC",
        0x38 => "SEC",
        0xF8 => "SED",
        0x78 => "SEI",
        0x07 | 0x17 | 0x0F | 0x1F | 0x1B | 0x03 | 0x13 => "SLO",
        0x47 | 0x57 | 0x4F | 0x5F | 0x5B | 0x43 | 0x53 => "SRE",
        0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => "STA",
        0x86 | 0x96 | 0x8E => "STX",
        0x84 | 0x94 | 0x8C => "STY",
        0xAA => "TAX",
        0xA8 => "TAY",
        0xBA => "TSX",
        0x8A => "TXA",
        0x9A => "TXS",
        0x98 => "TYA",
        0x0C | 0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => "TOP",
        _ => "UNIMPLEMENTED",
    }
}
