use super::{
    AddressingMode, AddressingResult,
    AddressingResult::{Address, Done, Implied, Relative, Value, ValueAddress},
    Cpu,
};

use crate::{
    cpu,
    utils::{
        self, add_with_carry, compare_u8, merge_u16, overflowing_add_u8_i8, rotate_left,
        rotate_right, shift_left, shift_right, split_u16, subtract_with_carry,
    },
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Instruction {
    pub(crate) base_instruction: BaseInstruction,
    pub(crate) addressing_mode: AddressingMode,
    addressing_done: bool,
}

impl Instruction {
    fn new(base_instruction: BaseInstruction, addressing_mode: AddressingMode) -> Self {
        Self {
            base_instruction,
            addressing_mode,
            addressing_done: false,
        }
    }

    pub(crate) fn no_op() -> Self {
        Self::new(BaseInstruction::NOP, AddressingMode::Implied)
    }

    pub(crate) fn from_opcode(opcode: u8) -> Self {
        match opcode {
            // AAC
            0x0B => Self::new(BaseInstruction::AAC, AddressingMode::Immediate),
            0x2B => Self::new(BaseInstruction::AAC, AddressingMode::Immediate),

            // AAX
            0x87 => Self::new(BaseInstruction::AAX, AddressingMode::zero_page_addr()),
            0x97 => Self::new(BaseInstruction::AAX, AddressingMode::zero_page_y_addr()),
            0x83 => Self::new(
                BaseInstruction::AAX,
                AddressingMode::indexed_indirect_addr(),
            ),
            0x8F => Self::new(BaseInstruction::AAX, AddressingMode::absolute_addr()),

            // ADC
            0x61 => Self::new(BaseInstruction::ADC, AddressingMode::indexed_indirect_val()),
            0x65 => Self::new(BaseInstruction::ADC, AddressingMode::zero_page_val()),
            0x69 => Self::new(BaseInstruction::ADC, AddressingMode::Immediate),
            0x6D => Self::new(BaseInstruction::ADC, AddressingMode::absolute_val()),
            0x71 => Self::new(BaseInstruction::ADC, AddressingMode::indirect_indexed_val()),
            0x75 => Self::new(BaseInstruction::ADC, AddressingMode::zero_page_x_val()),
            0x79 => Self::new(BaseInstruction::ADC, AddressingMode::absolute_y_val()),
            0x7D => Self::new(BaseInstruction::ADC, AddressingMode::absolute_x_val()),

            // ALR
            0x4B => Self::new(BaseInstruction::ALR, AddressingMode::Immediate),

            // AND
            0x21 => Self::new(BaseInstruction::AND, AddressingMode::indexed_indirect_val()),
            0x25 => Self::new(BaseInstruction::AND, AddressingMode::zero_page_val()),
            0x29 => Self::new(BaseInstruction::AND, AddressingMode::Immediate),
            0x2D => Self::new(BaseInstruction::AND, AddressingMode::absolute_val()),
            0x31 => Self::new(BaseInstruction::AND, AddressingMode::indirect_indexed_val()),
            0x35 => Self::new(BaseInstruction::AND, AddressingMode::zero_page_x_val()),
            0x39 => Self::new(BaseInstruction::AND, AddressingMode::absolute_y_val()),
            0x3D => Self::new(BaseInstruction::AND, AddressingMode::absolute_x_val()),

            // ARR
            0x6B => Self::new(BaseInstruction::ARR, AddressingMode::Immediate),

            // ASL
            0x06 => Self::new(BaseInstruction::ASL, AddressingMode::zero_page_val_addr()),
            0x0A => Self::new(BaseInstruction::ASL, AddressingMode::Accumulator),
            0x0E => Self::new(BaseInstruction::ASL, AddressingMode::absolute_val_addr()),
            0x16 => Self::new(BaseInstruction::ASL, AddressingMode::zero_page_x_val_addr()),
            0x1E => Self::new(BaseInstruction::ASL, AddressingMode::absolute_x_val_addr()),

            // ATX
            0xAB => Self::new(BaseInstruction::LAX, AddressingMode::Immediate),

            // AXS
            0xCB => Self::new(BaseInstruction::AXS, AddressingMode::Immediate),

            // BIT
            0x24 => Self::new(BaseInstruction::BIT, AddressingMode::zero_page_val()),
            0x2C => Self::new(BaseInstruction::BIT, AddressingMode::absolute_val()),

            // Branches
            0x90 => Self::new(BaseInstruction::BCC(0), AddressingMode::Relative),
            0xB0 => Self::new(BaseInstruction::BCS(0), AddressingMode::Relative),
            0xF0 => Self::new(BaseInstruction::BEQ(0), AddressingMode::Relative),
            0x30 => Self::new(BaseInstruction::BMI(0), AddressingMode::Relative),
            0xD0 => Self::new(BaseInstruction::BNE(0), AddressingMode::Relative),
            0x10 => Self::new(BaseInstruction::BPL(0), AddressingMode::Relative),
            0x50 => Self::new(BaseInstruction::BVC(0), AddressingMode::Relative),
            0x70 => Self::new(BaseInstruction::BVS(0), AddressingMode::Relative),

            // BRK
            0x00 => Self::new(
                BaseInstruction::BRK(BrkTarget::BRK, 0),
                AddressingMode::Implied,
            ),

            // Clears
            0x18 => Self::new(BaseInstruction::CLC, AddressingMode::Implied),
            0xD8 => Self::new(BaseInstruction::CLD, AddressingMode::Implied),
            0x58 => Self::new(BaseInstruction::CLI, AddressingMode::Implied),
            0xB8 => Self::new(BaseInstruction::CLV, AddressingMode::Implied),

            // CMP
            0xC9 => Self::new(BaseInstruction::CMP, AddressingMode::Immediate),
            0xC5 => Self::new(BaseInstruction::CMP, AddressingMode::zero_page_val()),
            0xD5 => Self::new(BaseInstruction::CMP, AddressingMode::zero_page_x_val()),
            0xCD => Self::new(BaseInstruction::CMP, AddressingMode::absolute_val()),
            0xDD => Self::new(BaseInstruction::CMP, AddressingMode::absolute_x_val()),
            0xD9 => Self::new(BaseInstruction::CMP, AddressingMode::absolute_y_val()),
            0xC1 => Self::new(BaseInstruction::CMP, AddressingMode::indexed_indirect_val()),
            0xD1 => Self::new(BaseInstruction::CMP, AddressingMode::indirect_indexed_val()),

            // CPX
            0xE0 => Self::new(BaseInstruction::CPX, AddressingMode::Immediate),
            0xE4 => Self::new(BaseInstruction::CPX, AddressingMode::zero_page_val()),
            0xEC => Self::new(BaseInstruction::CPX, AddressingMode::absolute_val()),

            // CPY
            0xC0 => Self::new(BaseInstruction::CPY, AddressingMode::Immediate),
            0xC4 => Self::new(BaseInstruction::CPY, AddressingMode::zero_page_val()),
            0xCC => Self::new(BaseInstruction::CPY, AddressingMode::absolute_val()),

            // DCP
            0xC7 => Self::new(BaseInstruction::DCP, AddressingMode::zero_page_val_addr()),
            0xD7 => Self::new(BaseInstruction::DCP, AddressingMode::zero_page_x_val_addr()),
            0xCF => Self::new(BaseInstruction::DCP, AddressingMode::absolute_val_addr()),
            0xDF => Self::new(BaseInstruction::DCP, AddressingMode::absolute_x_val_addr()),
            0xDB => Self::new(BaseInstruction::DCP, AddressingMode::absolute_y_val_addr()),
            0xC3 => Self::new(
                BaseInstruction::DCP,
                AddressingMode::indexed_indirect_val_addr(),
            ),
            0xD3 => Self::new(
                BaseInstruction::DCP,
                AddressingMode::indirect_indexed_val_addr(),
            ),

            // Decrement
            0xC6 => Self::new(BaseInstruction::DEC, AddressingMode::zero_page_val_addr()),
            0xD6 => Self::new(BaseInstruction::DEC, AddressingMode::zero_page_x_val_addr()),
            0xCE => Self::new(BaseInstruction::DEC, AddressingMode::absolute_val_addr()),
            0xDE => Self::new(BaseInstruction::DEC, AddressingMode::absolute_x_val_addr()),
            0xCA => Self::new(BaseInstruction::DEX, AddressingMode::Implied),
            0x88 => Self::new(BaseInstruction::DEY, AddressingMode::Implied),

            // DOP
            0x04 => Self::new(BaseInstruction::DOP, AddressingMode::zero_page_val()),
            0x14 => Self::new(BaseInstruction::DOP, AddressingMode::zero_page_x_val()),
            0x34 => Self::new(BaseInstruction::DOP, AddressingMode::zero_page_x_val()),
            0x44 => Self::new(BaseInstruction::DOP, AddressingMode::zero_page_val()),
            0x54 => Self::new(BaseInstruction::DOP, AddressingMode::zero_page_x_val()),
            0x64 => Self::new(BaseInstruction::DOP, AddressingMode::zero_page_val()),
            0x74 => Self::new(BaseInstruction::DOP, AddressingMode::zero_page_x_val()),
            0x80 => Self::new(BaseInstruction::DOP, AddressingMode::Immediate),
            0x82 => Self::new(BaseInstruction::DOP, AddressingMode::Immediate),
            0x89 => Self::new(BaseInstruction::DOP, AddressingMode::Immediate),
            0xC2 => Self::new(BaseInstruction::DOP, AddressingMode::Immediate),
            0xD4 => Self::new(BaseInstruction::DOP, AddressingMode::zero_page_x_val()),
            0xE2 => Self::new(BaseInstruction::DOP, AddressingMode::Immediate),
            0xF4 => Self::new(BaseInstruction::DOP, AddressingMode::zero_page_x_val()),

            // EOR
            0x41 => Self::new(BaseInstruction::EOR, AddressingMode::indexed_indirect_val()),
            0x45 => Self::new(BaseInstruction::EOR, AddressingMode::zero_page_val()),
            0x49 => Self::new(BaseInstruction::EOR, AddressingMode::Immediate),
            0x4D => Self::new(BaseInstruction::EOR, AddressingMode::absolute_val()),
            0x51 => Self::new(BaseInstruction::EOR, AddressingMode::indirect_indexed_val()),
            0x55 => Self::new(BaseInstruction::EOR, AddressingMode::zero_page_x_val()),
            0x59 => Self::new(BaseInstruction::EOR, AddressingMode::absolute_y_val()),
            0x5D => Self::new(BaseInstruction::EOR, AddressingMode::absolute_x_val()),

            // Increment
            0xE6 => Self::new(BaseInstruction::INC, AddressingMode::zero_page_val_addr()),
            0xF6 => Self::new(BaseInstruction::INC, AddressingMode::zero_page_x_val_addr()),
            0xEE => Self::new(BaseInstruction::INC, AddressingMode::absolute_val_addr()),
            0xFE => Self::new(BaseInstruction::INC, AddressingMode::absolute_x_val_addr()),
            0xE8 => Self::new(BaseInstruction::INX, AddressingMode::Implied),
            0xC8 => Self::new(BaseInstruction::INY, AddressingMode::Implied),

            // LAX
            0xA7 => Self::new(BaseInstruction::LAX, AddressingMode::zero_page_val()),
            0xB7 => Self::new(BaseInstruction::LAX, AddressingMode::zero_page_y_val()),
            0xAF => Self::new(BaseInstruction::LAX, AddressingMode::absolute_val()),
            0xBF => Self::new(BaseInstruction::LAX, AddressingMode::absolute_y_val()),
            0xA3 => Self::new(BaseInstruction::LAX, AddressingMode::indexed_indirect_val()),
            0xB3 => Self::new(BaseInstruction::LAX, AddressingMode::indirect_indexed_val()),

            // ISC
            0xE7 => Self::new(BaseInstruction::ISC, AddressingMode::zero_page_val_addr()),
            0xF7 => Self::new(BaseInstruction::ISC, AddressingMode::zero_page_x_val_addr()),
            0xEF => Self::new(BaseInstruction::ISC, AddressingMode::absolute_val_addr()),
            0xFF => Self::new(BaseInstruction::ISC, AddressingMode::absolute_x_val_addr()),
            0xFB => Self::new(BaseInstruction::ISC, AddressingMode::absolute_y_val_addr()),
            0xE3 => Self::new(
                BaseInstruction::ISC,
                AddressingMode::indexed_indirect_val_addr(),
            ),
            0xF3 => Self::new(
                BaseInstruction::ISC,
                AddressingMode::indirect_indexed_val_addr(),
            ),

            // LDA
            0xA1 => Self::new(BaseInstruction::LDA, AddressingMode::indexed_indirect_val()),
            0xA5 => Self::new(BaseInstruction::LDA, AddressingMode::zero_page_val()),
            0xA9 => Self::new(BaseInstruction::LDA, AddressingMode::Immediate),
            0xAD => Self::new(BaseInstruction::LDA, AddressingMode::absolute_val()),
            0xB1 => Self::new(BaseInstruction::LDA, AddressingMode::indirect_indexed_val()),
            0xB5 => Self::new(BaseInstruction::LDA, AddressingMode::zero_page_x_val()),
            0xB9 => Self::new(BaseInstruction::LDA, AddressingMode::absolute_y_val()),
            0xBD => Self::new(BaseInstruction::LDA, AddressingMode::absolute_x_val()),

            // LDX
            0xA2 => Self::new(BaseInstruction::LDX, AddressingMode::Immediate),
            0xA6 => Self::new(BaseInstruction::LDX, AddressingMode::zero_page_val()),
            0xB6 => Self::new(BaseInstruction::LDX, AddressingMode::zero_page_y_val()),
            0xAE => Self::new(BaseInstruction::LDX, AddressingMode::absolute_val()),
            0xBE => Self::new(BaseInstruction::LDX, AddressingMode::absolute_y_val()),

            // LDY
            0xA0 => Self::new(BaseInstruction::LDY, AddressingMode::Immediate),
            0xA4 => Self::new(BaseInstruction::LDY, AddressingMode::zero_page_val()),
            0xB4 => Self::new(BaseInstruction::LDY, AddressingMode::zero_page_x_val()),
            0xAC => Self::new(BaseInstruction::LDY, AddressingMode::absolute_val()),
            0xBC => Self::new(BaseInstruction::LDY, AddressingMode::absolute_x_val()),

            // LSR
            0x46 => Self::new(BaseInstruction::LSR, AddressingMode::zero_page_val_addr()),
            0x4A => Self::new(BaseInstruction::LSR, AddressingMode::Accumulator),
            0x4E => Self::new(BaseInstruction::LSR, AddressingMode::absolute_val_addr()),
            0x56 => Self::new(BaseInstruction::LSR, AddressingMode::zero_page_x_val_addr()),
            0x5E => Self::new(BaseInstruction::LSR, AddressingMode::absolute_x_val_addr()),

            // JMP
            0x4C => Self::new(BaseInstruction::JMP, AddressingMode::absolute_jmp()),
            0x6C => Self::new(BaseInstruction::JMP, AddressingMode::indirect()),

            // JSR addressing mode should be "Absolute" but this makes it easyer to implement correctly
            0x20 => Self::new(BaseInstruction::JSR(0), AddressingMode::Implied),

            // NOP
            0xEA => Self::new(BaseInstruction::NOP, AddressingMode::Implied),
            0x1A => Self::new(BaseInstruction::NOP, AddressingMode::Implied),
            0x3A => Self::new(BaseInstruction::NOP, AddressingMode::Implied),
            0x5A => Self::new(BaseInstruction::NOP, AddressingMode::Implied),
            0x7A => Self::new(BaseInstruction::NOP, AddressingMode::Implied),
            0xDA => Self::new(BaseInstruction::NOP, AddressingMode::Implied),
            0xFA => Self::new(BaseInstruction::NOP, AddressingMode::Implied),

            // ORA
            0x01 => Self::new(BaseInstruction::ORA, AddressingMode::indexed_indirect_val()),
            0x05 => Self::new(BaseInstruction::ORA, AddressingMode::zero_page_val()),
            0x09 => Self::new(BaseInstruction::ORA, AddressingMode::Immediate),
            0x0D => Self::new(BaseInstruction::ORA, AddressingMode::absolute_val()),
            0x11 => Self::new(BaseInstruction::ORA, AddressingMode::indirect_indexed_val()),
            0x15 => Self::new(BaseInstruction::ORA, AddressingMode::zero_page_x_val()),
            0x19 => Self::new(BaseInstruction::ORA, AddressingMode::absolute_y_val()),
            0x1D => Self::new(BaseInstruction::ORA, AddressingMode::absolute_x_val()),

            // Push
            0x48 => Self::new(BaseInstruction::PHA, AddressingMode::Implied),
            0x08 => Self::new(BaseInstruction::PHP, AddressingMode::Implied),

            // Pull
            0x68 => Self::new(BaseInstruction::PLA, AddressingMode::Implied),
            0x28 => Self::new(BaseInstruction::PLP, AddressingMode::Implied),

            // RLA
            0x27 => Self::new(BaseInstruction::RLA, AddressingMode::zero_page_val_addr()),
            0x37 => Self::new(BaseInstruction::RLA, AddressingMode::zero_page_x_val_addr()),
            0x2F => Self::new(BaseInstruction::RLA, AddressingMode::absolute_val_addr()),
            0x3F => Self::new(BaseInstruction::RLA, AddressingMode::absolute_x_val_addr()),
            0x3B => Self::new(BaseInstruction::RLA, AddressingMode::absolute_y_val_addr()),
            0x23 => Self::new(
                BaseInstruction::RLA,
                AddressingMode::indexed_indirect_val_addr(),
            ),
            0x33 => Self::new(
                BaseInstruction::RLA,
                AddressingMode::indirect_indexed_val_addr(),
            ),

            // RRA
            0x67 => Self::new(BaseInstruction::RRA, AddressingMode::zero_page_val_addr()),
            0x77 => Self::new(BaseInstruction::RRA, AddressingMode::zero_page_x_val_addr()),
            0x6F => Self::new(BaseInstruction::RRA, AddressingMode::absolute_val_addr()),
            0x7F => Self::new(BaseInstruction::RRA, AddressingMode::absolute_x_val_addr()),
            0x7B => Self::new(BaseInstruction::RRA, AddressingMode::absolute_y_val_addr()),
            0x63 => Self::new(
                BaseInstruction::RRA,
                AddressingMode::indexed_indirect_val_addr(),
            ),
            0x73 => Self::new(
                BaseInstruction::RRA,
                AddressingMode::indirect_indexed_val_addr(),
            ),

            // ROL
            0x26 => Self::new(BaseInstruction::ROL, AddressingMode::zero_page_val_addr()),
            0x2A => Self::new(BaseInstruction::ROL, AddressingMode::Accumulator),
            0x2E => Self::new(BaseInstruction::ROL, AddressingMode::absolute_val_addr()),
            0x36 => Self::new(BaseInstruction::ROL, AddressingMode::zero_page_x_val_addr()),
            0x3E => Self::new(BaseInstruction::ROL, AddressingMode::absolute_x_val_addr()),

            // ROR
            0x66 => Self::new(BaseInstruction::ROR, AddressingMode::zero_page_val_addr()),
            0x6A => Self::new(BaseInstruction::ROR, AddressingMode::Accumulator),
            0x6E => Self::new(BaseInstruction::ROR, AddressingMode::absolute_val_addr()),
            0x76 => Self::new(BaseInstruction::ROR, AddressingMode::zero_page_x_val_addr()),
            0x7E => Self::new(BaseInstruction::ROR, AddressingMode::absolute_x_val_addr()),

            // RTI
            0x40 => Self::new(BaseInstruction::RTI(0), AddressingMode::Implied),

            // RTS
            0x60 => Self::new(BaseInstruction::RTS(0), AddressingMode::Implied),

            // SBC
            0xE1 => Self::new(BaseInstruction::SBC, AddressingMode::indexed_indirect_val()),
            0xE5 => Self::new(BaseInstruction::SBC, AddressingMode::zero_page_val()),
            0xE9 => Self::new(BaseInstruction::SBC, AddressingMode::Immediate),
            0xEB => Self::new(BaseInstruction::SBC, AddressingMode::Immediate), // Unofficial
            0xED => Self::new(BaseInstruction::SBC, AddressingMode::absolute_val()),
            0xF1 => Self::new(BaseInstruction::SBC, AddressingMode::indirect_indexed_val()),
            0xF5 => Self::new(BaseInstruction::SBC, AddressingMode::zero_page_x_val()),
            0xF9 => Self::new(BaseInstruction::SBC, AddressingMode::absolute_y_val()),
            0xFD => Self::new(BaseInstruction::SBC, AddressingMode::absolute_x_val()),

            // Sets
            0x38 => Self::new(BaseInstruction::SEC, AddressingMode::Implied),
            0xF8 => Self::new(BaseInstruction::SED, AddressingMode::Implied),
            0x78 => Self::new(BaseInstruction::SEI, AddressingMode::Implied),

            // SLO
            0x07 => Self::new(BaseInstruction::SLO, AddressingMode::zero_page_val_addr()),
            0x17 => Self::new(BaseInstruction::SLO, AddressingMode::zero_page_x_val_addr()),
            0x0F => Self::new(BaseInstruction::SLO, AddressingMode::absolute_val_addr()),
            0x1F => Self::new(BaseInstruction::SLO, AddressingMode::absolute_x_val_addr()),
            0x1B => Self::new(BaseInstruction::SLO, AddressingMode::absolute_y_val_addr()),
            0x03 => Self::new(
                BaseInstruction::SLO,
                AddressingMode::indexed_indirect_val_addr(),
            ),
            0x13 => Self::new(
                BaseInstruction::SLO,
                AddressingMode::indirect_indexed_val_addr(),
            ),

            // SRE
            0x47 => Self::new(BaseInstruction::SRE, AddressingMode::zero_page_val_addr()),
            0x57 => Self::new(BaseInstruction::SRE, AddressingMode::zero_page_x_val_addr()),
            0x4F => Self::new(BaseInstruction::SRE, AddressingMode::absolute_val_addr()),
            0x5F => Self::new(BaseInstruction::SRE, AddressingMode::absolute_x_val_addr()),
            0x5B => Self::new(BaseInstruction::SRE, AddressingMode::absolute_y_val_addr()),
            0x43 => Self::new(
                BaseInstruction::SRE,
                AddressingMode::indexed_indirect_val_addr(),
            ),
            0x53 => Self::new(
                BaseInstruction::SRE,
                AddressingMode::indirect_indexed_val_addr(),
            ),

            // STA
            0x85 => Self::new(BaseInstruction::STA, AddressingMode::zero_page_addr()),
            0x95 => Self::new(BaseInstruction::STA, AddressingMode::zero_page_x_addr()),
            0x8D => Self::new(BaseInstruction::STA, AddressingMode::absolute_addr()),
            0x9D => Self::new(BaseInstruction::STA, AddressingMode::absolute_x_addr()),
            0x99 => Self::new(BaseInstruction::STA, AddressingMode::absolute_y_addr()),
            0x81 => Self::new(
                BaseInstruction::STA,
                AddressingMode::indexed_indirect_addr(),
            ),
            0x91 => Self::new(
                BaseInstruction::STA,
                AddressingMode::indirect_indexed_addr(),
            ),

            // STX
            0x86 => Self::new(BaseInstruction::STX, AddressingMode::zero_page_addr()),
            0x96 => Self::new(BaseInstruction::STX, AddressingMode::zero_page_y_addr()),
            0x8E => Self::new(BaseInstruction::STX, AddressingMode::absolute_addr()),

            // STY
            0x84 => Self::new(BaseInstruction::STY, AddressingMode::zero_page_addr()),
            0x94 => Self::new(BaseInstruction::STY, AddressingMode::zero_page_x_addr()),
            0x8C => Self::new(BaseInstruction::STY, AddressingMode::absolute_addr()),

            // Transfers
            0xAA => Self::new(BaseInstruction::TAX, AddressingMode::Implied),
            0xA8 => Self::new(BaseInstruction::TAY, AddressingMode::Implied),
            0xBA => Self::new(BaseInstruction::TSX, AddressingMode::Implied),
            0x8A => Self::new(BaseInstruction::TXA, AddressingMode::Implied),
            0x9A => Self::new(BaseInstruction::TXS, AddressingMode::Implied),
            0x98 => Self::new(BaseInstruction::TYA, AddressingMode::Implied),

            // Triple NOP
            0x0C => Self::new(BaseInstruction::TOP, AddressingMode::absolute_val()),
            0x1C => Self::new(BaseInstruction::TOP, AddressingMode::absolute_x_val()),
            0x3C => Self::new(BaseInstruction::TOP, AddressingMode::absolute_x_val()),
            0x5C => Self::new(BaseInstruction::TOP, AddressingMode::absolute_x_val()),
            0x7C => Self::new(BaseInstruction::TOP, AddressingMode::absolute_x_val()),
            0xDC => Self::new(BaseInstruction::TOP, AddressingMode::absolute_x_val()),
            0xFC => Self::new(BaseInstruction::TOP, AddressingMode::absolute_x_val()),

            //_ => Self::new(BaseInstruction::NOP, AddressingMode::Implied),
            byte => Self::new(BaseInstruction::Unknown(byte), AddressingMode::Implied),
            //_ => unimplemented!("Unimplemented opcode: '{:#04x}'", opcode),
        }
    }

    pub(crate) fn tick(cpu: &mut Cpu) {
        #[cfg(debug_assertions)]
        println!(
            "instr: {:?}, cycle: {}",
            cpu.current_instr.base_instruction, cpu.instr_cycle
        );
        //let self_i = cpu.current_instr;
        if !cpu.current_instr.addressing_done {
            let res = AddressingMode::tick(cpu);
            if let AddressingResult::NotDone = res {
            } else {
                cpu.current_instr.addressing_done = true;
                BaseInstruction::execute(res, cpu);
            };
            if cpu.instr_cycle != 0 {
                cpu.instr_cycle += 1;
            } else {
                cpu.pool_interrupts();
            }
        } else {
            BaseInstruction::execute(AddressingResult::Done, cpu);
            if cpu.instr_cycle != 0 {
                cpu.instr_cycle += 1;
            } else {
                cpu.pool_interrupts();
            }
        }
    }
}

macro_rules! branch_if_condition {
    ($condition:expr, $cpu:ident, $delta:ident, $addr_result:ident) => {
        match $addr_result {
            Relative(d) => {
                assert_eq!($cpu.instr_cycle, 2);
                if $condition {
                    *$delta = d;
                } else {
                    // terminate instruction without branching
                    $cpu.instr_cycle = 0;
                }
            }
            Done => match $cpu.instr_cycle {
                3 => {
                    // We didn't branch but fetch opcode of next instruction and throw it away
                    let _ = $cpu.memory.read_u8($cpu.program_counter);

                        // We are branching, check first if the PC will overflow
                    let (res, overflow) =
                        overflowing_add_u8_i8($cpu.program_counter as u8, *$delta);
                        //($cpu.program_counter as u8).overflowing_add(*$delta as u8);

                    if overflow {
                        // we will branch at the next cycle, after fixing the high byte of the address
                    } else {
                        // branch now
                        let (_, high) = split_u16($cpu.program_counter);
                        $cpu.program_counter = merge_u16(res, high);
                        $cpu.instr_cycle = 0;
                    }
                }
                4 => {
                    let (low, high) = split_u16($cpu.program_counter);
                    let low = low.wrapping_add(*$delta as u8);

                    // fix high byte
                    let high = if *$delta >= 0 { high + 1 } else { high - 1 };

                    $cpu.program_counter = merge_u16(low, high);
                    $cpu.instr_cycle = 0;
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum BrkTarget {
    BRK,
    IRQ,
    NMI,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum BaseInstruction {
    AAC, // Unofficial
    AAX,
    ADC,
    ALR,
    AND,
    ARR,
    ASL,
    //ATX,
    AXS,
    BCC(i8),
    BCS(i8),
    BEQ(i8),
    BIT,
    BMI(i8),
    BNE(i8),
    BPL(i8),
    BRK(BrkTarget, u8),
    BVC(i8),
    BVS(i8),
    CLC,
    CLD,
    CLI,
    CLV,

    CMP,
    CPX,
    CPY,

    DCP,
    DEC,
    DEX,
    DEY,
    DOP, // Double NOP
    EOR,
    INC,
    INX,
    INY,

    ISC, // Increment then SBC value

    JMP,
    JSR(u8),

    LAX,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,

    RLA,
    RRA,

    ROL,
    ROR,
    RTI(u8),
    RTS(u8),

    SBC,
    SEC,
    SED,
    SEI,

    SLO,
    SRE,

    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    TOP, // Triple NOP
    Unknown(u8),
}

impl BaseInstruction {
    fn execute(
        //&self,
        addr_result: AddressingResult,
        //_addressing_mode: AddressingMode,
        cpu: &mut Cpu,
    ) {
        match &mut cpu.current_instr.base_instruction {
            BaseInstruction::AAC => {
                if let Value(m) = addr_result {
                    let acc = cpu.accumulator & m;

                    cpu.accumulator = acc;
                    cpu.zero_flag = acc == 0;
                    cpu.negative_flag = acc >= 128u8;
                    cpu.carry_flag = cpu.negative_flag;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::AAX => {
                if let Address(address) = addr_result {
                    let value = cpu.accumulator & cpu.x;
                    cpu.memory.write_u8(address, value);

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!("{:?}", addr_result);
                }
            }
            BaseInstruction::ADC => {
                if let Value(m) = addr_result {
                    //let old_acc_sign = cpu.accumulator & 128u8;
                    //
                    //let (mut acc, mut carry) = cpu.accumulator.overflowing_add(m);
                    //if cpu.carry_flag {
                    //let (acc2, carry2) = acc.overflowing_add(1);
                    //acc = acc2;
                    //carry = carry || carry2;
                    //}
                    //
                    //cpu.accumulator = acc;
                    //cpu.carry_flag = carry;
                    //cpu.zero_flag = acc == 0;
                    //
                    //// TODO: check if set overflow flag correctly
                    //let new_acc_sign = cpu.accumulator & 128u8;
                    //cpu.overflow_flag = old_acc_sign != new_acc_sign;
                    //
                    //cpu.negative_flag = acc >= 128u8;
                    let res = add_with_carry(cpu.accumulator, m, cpu.carry_flag);

                    cpu.accumulator = res.result;
                    cpu.carry_flag = res.carry_flag;
                    cpu.overflow_flag = res.overflow_flag;
                    cpu.zero_flag = res.zero_flag;
                    cpu.negative_flag = res.negative_flag;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!("{:?}", addr_result);
                }
            }
            BaseInstruction::AND => {
                if let Value(m) = addr_result {
                    let acc = cpu.accumulator & m;

                    cpu.accumulator = acc;
                    cpu.zero_flag = acc == 0;
                    cpu.negative_flag = acc >= 128u8;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::ALR => {
                if let Value(m) = addr_result {
                    let and_res = cpu.accumulator & m;

                    let res = shift_right(and_res);
                    cpu.accumulator = res.result;
                    cpu.carry_flag = res.carry_flag;
                    cpu.zero_flag = res.zero_flag;
                    cpu.negative_flag = res.negative_flag;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::ARR => {
                if let Value(m) = addr_result {
                    let value = cpu.accumulator & m;

                    let res = rotate_right(value, cpu.carry_flag);
                    cpu.accumulator = res.result;
                    cpu.zero_flag = res.zero_flag;
                    cpu.negative_flag = res.negative_flag;

                    let bit6 = (res.result & 0b01000000) != 0;
                    let bit5 = (res.result & 0b00100000) != 0;
                    cpu.carry_flag = bit6;
                    cpu.overflow_flag = bit6 ^ bit5;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::ASL => {
                let res = match addr_result {
                    Value(m) => {
                        let res = shift_left(m);
                        cpu.accumulator = res.result;
                        res
                    }
                    ValueAddress(val, addr) => {
                        let res = shift_left(val);
                        cpu.memory.write_u8(addr, res.result);
                        res
                    }
                    _ => unreachable!(),
                };
                cpu.carry_flag = res.carry_flag;
                cpu.zero_flag = res.zero_flag;
                cpu.negative_flag = res.negative_flag;

                cpu.instr_cycle = 0;
            }
            //BaseInstruction::ATX => {
            //if let Value(m) = addr_result {
            //let value = cpu.accumulator & m;
            //
            //cpu.accumulator = value;
            //cpu.x = value;
            //cpu.zero_flag = value == 0;
            //cpu.negative_flag = value >= 128;
            //
            //cpu.instr_cycle = 0;
            //} else {
            //unreachable!();
            //}
            //}
            BaseInstruction::AXS => {
                if let Value(m) = addr_result {
                    let value = cpu.accumulator & cpu.x;

                    let res = subtract_with_carry(value, m, true);
                    cpu.x = res.result;
                    cpu.zero_flag = res.zero_flag;
                    cpu.negative_flag = res.negative_flag;
                    cpu.carry_flag = res.carry_flag;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::BCC(delta) => {
                branch_if_condition!(cpu.carry_flag == false, cpu, delta, addr_result)
            }
            BaseInstruction::BCS(delta) => {
                branch_if_condition!(cpu.carry_flag == true, cpu, delta, addr_result)
            }
            BaseInstruction::BEQ(delta) => {
                branch_if_condition!(cpu.zero_flag == true, cpu, delta, addr_result)
            }
            BaseInstruction::BIT => {
                if let Value(m) = addr_result {
                    let res = cpu.accumulator & m;

                    cpu.zero_flag = res == 0;
                    cpu.overflow_flag = (m & 64u8) > 0;
                    cpu.negative_flag = m >= 128u8;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::BMI(delta) => {
                branch_if_condition!(cpu.negative_flag == true, cpu, delta, addr_result)
            }
            BaseInstruction::BNE(delta) => {
                //let _ = std::io::Read::read(&mut std::io::stdin(), &mut [0u8; 15]);
                branch_if_condition!(cpu.zero_flag == false, cpu, delta, addr_result)
            }
            BaseInstruction::BPL(delta) => {
                branch_if_condition!(cpu.negative_flag == false, cpu, delta, addr_result)
            }
            BaseInstruction::BRK(target, pcl) => match cpu.instr_cycle {
                2 => match addr_result {
                    Implied(_) => {}
                    _ => unreachable!(),
                },
                3 => {
                    let (_, high) = split_u16(cpu.program_counter);
                    cpu.memory.write_u8(cpu.stack_address(), high);
                    cpu.decrement_stack_pointer();
                }
                4 => {
                    let (low, _) = split_u16(cpu.program_counter);
                    cpu.memory.write_u8(cpu.stack_address(), low);
                    cpu.decrement_stack_pointer();
                }
                5 => {
                    // TODO: Handle IRQ interrupts here
                    *target = if cpu.nmi {
                        BrkTarget::NMI
                    } else {
                        BrkTarget::BRK
                    };
                    let target = *target;
                    let mut status = cpu.get_processor_status();
                    match target {
                        BrkTarget::BRK => {
                            status = status | 0b00010000; // set break flag
                        }
                        BrkTarget::IRQ => {
                            status = status & 0b11101111; // clear break flag
                        }
                        BrkTarget::NMI => {
                            status = status & 0b11101111; // clear break flag
                            cpu.nmi = false;
                        }
                    }
                    cpu.memory.write_u8(cpu.stack_address(), status);
                    cpu.decrement_stack_pointer();
                }
                6 => {
                    match *target {
                        BrkTarget::BRK | BrkTarget::IRQ => {
                            *pcl = cpu.memory.read_u8(cpu::addresses::IRQ_BRK_VECTOR);
                        }
                        BrkTarget::NMI => {
                            *pcl = cpu.memory.read_u8(cpu::addresses::NMI_VECTOR);
                        }
                    }
                    cpu.interrupt_disable = true;
                }
                7 => {
                    let pch = match *target {
                        BrkTarget::BRK | BrkTarget::IRQ => {
                            cpu.memory.read_u8(cpu::addresses::IRQ_BRK_VECTOR + 1)
                        }
                        BrkTarget::NMI => cpu.memory.read_u8(cpu::addresses::NMI_VECTOR + 1),
                    };
                    //cpu.program_counter = (pch as u16) << 8 | (*pcl as u16);
                    cpu.program_counter = merge_u16(*pcl, pch);
                    cpu.instr_cycle = 0;
                }
                _ => unimplemented!("BRK instruction at cycle {}", cpu.instr_cycle),
            },
            BaseInstruction::BVC(delta) => {
                branch_if_condition!(cpu.overflow_flag == false, cpu, delta, addr_result)
            }
            BaseInstruction::BVS(delta) => {
                branch_if_condition!(cpu.overflow_flag == true, cpu, delta, addr_result)
            }
            BaseInstruction::CLC => {
                match addr_result {
                    Implied(_) => {}
                    _ => unreachable!(),
                }
                cpu.carry_flag = false;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::CLD => {
                match addr_result {
                    Implied(_) => {}
                    _ => unreachable!(),
                }
                cpu.decimal_mode = false;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::CLI => {
                match addr_result {
                    Implied(_) => {}
                    _ => unreachable!(),
                }
                cpu.interrupt_disable = false;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::CLV => {
                match addr_result {
                    Implied(_) => {}
                    _ => unreachable!(),
                }
                cpu.overflow_flag = false;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::CMP => {
                if let Value(m) = addr_result {
                    let res = utils::compare_u8(cpu.accumulator, m);

                    cpu.carry_flag = res.carry;
                    cpu.zero_flag = res.zero;
                    cpu.negative_flag = res.neg;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::CPX => {
                if let Value(m) = addr_result {
                    let res = utils::compare_u8(cpu.x, m);

                    cpu.carry_flag = res.carry;
                    cpu.zero_flag = res.zero;
                    cpu.negative_flag = res.neg;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::CPY => {
                if let Value(m) = addr_result {
                    let res = utils::compare_u8(cpu.y, m);

                    cpu.carry_flag = res.carry;
                    cpu.zero_flag = res.zero;
                    cpu.negative_flag = res.neg;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::DCP => {
                if let ValueAddress(value, address) = addr_result {
                    // TODO: check this
                    let res = subtract_with_carry(value, 1, true);
                    cpu.memory.write_u8(address, res.result);

                    //cpu.carry_flag = res.carry_flag;
                    let cmp_res = compare_u8(cpu.accumulator, res.result);
                    cpu.carry_flag = cmp_res.carry;
                    cpu.negative_flag = cmp_res.neg;
                    cpu.zero_flag = cmp_res.zero;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::DEC => {
                if let ValueAddress(value, address) = addr_result {
                    let value = value.wrapping_sub(1);
                    cpu.memory.write_u8(address, value);

                    cpu.zero_flag = value == 0u8;
                    cpu.negative_flag = value >= 128u8;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::DEX => {
                match addr_result {
                    Implied(_) => {}
                    _ => unreachable!(),
                }
                cpu.x = cpu.x.wrapping_sub(1);

                cpu.zero_flag = cpu.x == 0u8;
                cpu.negative_flag = cpu.x >= 128u8;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::DEY => {
                match addr_result {
                    Implied(_) => {}
                    _ => unreachable!(),
                }
                cpu.y = cpu.y.wrapping_sub(1);

                cpu.zero_flag = cpu.y == 0u8;
                cpu.negative_flag = cpu.y >= 128u8;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::DOP => cpu.instr_cycle = 0,
            BaseInstruction::EOR => {
                if let Value(m) = addr_result {
                    let acc = cpu.accumulator ^ m;

                    cpu.accumulator = acc;
                    cpu.zero_flag = acc == 0;
                    cpu.negative_flag = acc >= 128u8;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::INC => {
                if let ValueAddress(value, address) = addr_result {
                    let value = value.wrapping_add(1);
                    cpu.memory.write_u8(address, value);

                    cpu.zero_flag = value == 0u8;
                    cpu.negative_flag = value >= 128u8;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::INX => {
                match addr_result {
                    Implied(_) => {}
                    _ => unreachable!(),
                }
                cpu.x = cpu.x.wrapping_add(1);

                cpu.zero_flag = cpu.x == 0u8;
                cpu.negative_flag = cpu.x >= 128u8;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::INY => {
                match addr_result {
                    Implied(_) => {}
                    _ => unreachable!(),
                }
                cpu.y = cpu.y.wrapping_add(1);

                cpu.zero_flag = cpu.y == 0u8;
                cpu.negative_flag = cpu.y >= 128u8;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::ISC => {
                if let ValueAddress(value, address) = addr_result {
                    let value = value.wrapping_add(1);
                    cpu.memory.write_u8(address, value);

                    let res = subtract_with_carry(cpu.accumulator, value, cpu.carry_flag);

                    cpu.accumulator = res.result;
                    cpu.carry_flag = res.carry_flag;
                    cpu.zero_flag = res.zero_flag;
                    cpu.overflow_flag = res.overflow_flag;
                    cpu.negative_flag = res.negative_flag;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::LAX => {
                if let Value(m) = addr_result {
                    cpu.accumulator = m;
                    cpu.x = m;
                    cpu.zero_flag = m == 0;
                    cpu.negative_flag = m >= 128u8;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::LDA => {
                if let Value(m) = addr_result {
                    cpu.accumulator = m;
                    cpu.zero_flag = m == 0;
                    cpu.negative_flag = m >= 128u8;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::LDX => {
                if let Value(m) = addr_result {
                    cpu.x = m;
                    cpu.zero_flag = m == 0;
                    cpu.negative_flag = m >= 128u8;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::LDY => {
                if let Value(m) = addr_result {
                    cpu.y = m;
                    cpu.zero_flag = m == 0;
                    cpu.negative_flag = m >= 128u8;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::JMP => {
                if let Address(address) = addr_result {
                    cpu.program_counter = address;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!("{:?}", addr_result);
                }
            }
            BaseInstruction::JSR(pcl) => match cpu.instr_cycle {
                2 => {
                    if let Implied(byte) = addr_result {
                        *pcl = byte;
                        cpu.program_counter += 1;
                    } else {
                        unreachable!()
                    }
                }
                3 => {
                    let _ = cpu.memory.read_u8(cpu.stack_address());
                }
                4 => {
                    let (_, high) = split_u16(cpu.program_counter);
                    cpu.memory.write_u8(cpu.stack_address(), high);
                    cpu.decrement_stack_pointer();
                }
                5 => {
                    let (low, _) = split_u16(cpu.program_counter);
                    cpu.memory.write_u8(cpu.stack_address(), low);
                    cpu.decrement_stack_pointer();
                }
                6 => {
                    let pch = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter = merge_u16(*pcl, pch);
                    //println!("jumping to subroutine at address {:#06x}", cpu.program_counter);

                    cpu.instr_cycle = 0;
                }
                _ => unimplemented!("JSR instruction at cycle {}", cpu.instr_cycle),
            },
            BaseInstruction::LSR => {
                let res = match addr_result {
                    Value(m) => {
                        let res = shift_right(m);
                        cpu.accumulator = res.result;
                        res
                    }
                    ValueAddress(val, addr) => {
                        let res = shift_right(val);
                        cpu.memory.write_u8(addr, res.result);
                        res
                    }
                    _ => unreachable!(),
                };
                cpu.carry_flag = res.carry_flag;
                cpu.zero_flag = res.zero_flag;
                cpu.negative_flag = res.negative_flag;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::NOP => cpu.instr_cycle = 0,
            BaseInstruction::ORA => {
                if let Value(m) = addr_result {
                    let acc = cpu.accumulator | m;

                    cpu.accumulator = acc;
                    cpu.zero_flag = acc == 0;
                    cpu.negative_flag = acc >= 128u8;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::PHA => match addr_result {
                Implied(_) => {}
                Done => {
                    cpu.memory.write_u8(cpu.stack_address(), cpu.accumulator);
                    cpu.decrement_stack_pointer();

                    cpu.instr_cycle = 0;
                }
                _ => unreachable!(),
            },
            BaseInstruction::PHP => match addr_result {
                Implied(_) => {}
                Done => {
                    let status = cpu.get_processor_status();
                    // TODO: check this
                    let status = status | 16; // Break flag
                    cpu.memory.write_u8(cpu.stack_address(), status);
                    cpu.decrement_stack_pointer();

                    cpu.instr_cycle = 0;
                }
                _ => unreachable!(),
            },
            BaseInstruction::PLA => match addr_result {
                Implied(_) => {}
                Done => match cpu.instr_cycle {
                    3 => {
                        let _ = cpu.memory.read_u8(cpu.stack_address());
                        cpu.increment_stack_pointer();
                    }
                    4 => {
                        cpu.accumulator = cpu.memory.read_u8(cpu.stack_address());

                        cpu.zero_flag = cpu.accumulator == 0;
                        cpu.negative_flag = cpu.accumulator >= 128;

                        cpu.instr_cycle = 0;
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            },
            BaseInstruction::PLP => match addr_result {
                Implied(_) => {}
                Done => match cpu.instr_cycle {
                    3 => {
                        let _ = cpu.memory.read_u8(cpu.stack_address());
                        cpu.increment_stack_pointer();
                    }
                    4 => {
                        let status = cpu.memory.read_u8(cpu.stack_address());
                        let status = status & 0b11101111;
                        cpu.set_processor_status(status);

                        cpu.instr_cycle = 0;
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            },
            BaseInstruction::RLA => {
                if let ValueAddress(value, address) = addr_result {
                    let res = rotate_left(value, cpu.carry_flag);
                    cpu.memory.write_u8(address, res.result);

                    cpu.carry_flag = res.carry_flag;

                    let acc = cpu.accumulator & res.result;

                    cpu.accumulator = acc;
                    cpu.zero_flag = acc == 0;
                    cpu.negative_flag = acc >= 128u8;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::RRA => {
                if let ValueAddress(value, address) = addr_result {
                    let ror_res = rotate_right(value, cpu.carry_flag);
                    cpu.memory.write_u8(address, ror_res.result);

                    let adc_res =
                        add_with_carry(cpu.accumulator, ror_res.result, ror_res.carry_flag);
                    cpu.accumulator = adc_res.result;
                    cpu.carry_flag = adc_res.carry_flag;
                    cpu.zero_flag = adc_res.zero_flag;
                    cpu.negative_flag = adc_res.negative_flag;
                    cpu.overflow_flag = adc_res.overflow_flag;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::ROL => {
                let res = match addr_result {
                    Value(m) => {
                        let res = rotate_left(m, cpu.carry_flag);
                        cpu.accumulator = res.result;
                        res
                    }
                    ValueAddress(val, addr) => {
                        let res = rotate_left(val, cpu.carry_flag);
                        cpu.memory.write_u8(addr, res.result);
                        res
                    }
                    _ => unreachable!(),
                };
                cpu.carry_flag = res.carry_flag;
                cpu.zero_flag = res.zero_flag;
                cpu.negative_flag = res.negative_flag;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::ROR => {
                let res = match addr_result {
                    Value(m) => {
                        let res = rotate_right(m, cpu.carry_flag);
                        cpu.accumulator = res.result;
                        res
                    }
                    ValueAddress(val, addr) => {
                        let res = rotate_right(val, cpu.carry_flag);
                        cpu.memory.write_u8(addr, res.result);
                        res
                    }
                    _ => unreachable!(),
                };
                cpu.carry_flag = res.carry_flag;
                cpu.zero_flag = res.zero_flag;
                cpu.negative_flag = res.negative_flag;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::RTI(pcl) => match addr_result {
                Implied(_) => {}
                Done => match cpu.instr_cycle {
                    3 => {
                        let _ = cpu.memory.read_u8(cpu.stack_address());
                        cpu.increment_stack_pointer();
                    }
                    4 => {
                        let val = cpu.memory.read_u8(cpu.stack_address());
                        cpu.set_processor_status(val);
                        cpu.increment_stack_pointer();
                    }
                    5 => {
                        *pcl = cpu
                            .memory
                            .read_u8(cpu.stack_pointer as u16 + cpu::addresses::STACK);
                        cpu.increment_stack_pointer();
                    }
                    6 => {
                        let pch = cpu
                            .memory
                            .read_u8(cpu.stack_pointer as u16 + cpu::addresses::STACK);
                        cpu.program_counter = merge_u16(*pcl, pch);

                        cpu.instr_cycle = 0;
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            },
            BaseInstruction::RTS(pcl) => match addr_result {
                Implied(_) => {}
                Done => match cpu.instr_cycle {
                    3 => {
                        let _ = cpu.memory.read_u8(cpu.stack_address());
                        cpu.increment_stack_pointer();
                    }
                    4 => {
                        *pcl = cpu
                            .memory
                            .read_u8(cpu.stack_pointer as u16 + cpu::addresses::STACK);
                        cpu.stack_pointer = cpu.stack_pointer.wrapping_add(1);
                    }
                    5 => {
                        let pch = cpu
                            .memory
                            .read_u8(cpu.stack_pointer as u16 + cpu::addresses::STACK);
                        cpu.program_counter = merge_u16(*pcl, pch);
                    }
                    6 => {
                        cpu.memory.read_u8(cpu.program_counter);
                        cpu.program_counter += 1;

                        cpu.instr_cycle = 0;
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            },
            BaseInstruction::SBC => {
                if let Value(m) = addr_result {
                    //let old_acc_sign = cpu.accumulator & 128u8;
                    //
                    //let (mut acc, mut carry) = cpu.accumulator.overflowing_sub(m);
                    //if cpu.carry_flag == false {
                    //let (acc2, carry2) = acc.overflowing_sub(1);
                    //acc = acc2;
                    //carry = carry || carry2;
                    //}
                    //
                    //cpu.accumulator = acc;
                    //cpu.carry_flag = carry;
                    //cpu.zero_flag = acc == 0;
                    //
                    //// TODO: check if set overflow flag correctly
                    //let new_acc_sign = cpu.accumulator & 128u8;
                    //cpu.overflow_flag = old_acc_sign != new_acc_sign;
                    //
                    //cpu.negative_flag = acc >= 128u8;

                    let res = subtract_with_carry(cpu.accumulator, m, cpu.carry_flag);

                    cpu.accumulator = res.result;
                    cpu.carry_flag = res.carry_flag;
                    cpu.overflow_flag = res.overflow_flag;
                    cpu.zero_flag = res.zero_flag;
                    cpu.negative_flag = res.negative_flag;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::SEC => {
                match addr_result {
                    Implied(_) => {}
                    _ => unreachable!(),
                }
                cpu.carry_flag = true;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::SED => {
                match addr_result {
                    Implied(_) => {}
                    _ => unreachable!(),
                }
                cpu.decimal_mode = true;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::SEI => {
                match addr_result {
                    Implied(_) => {}
                    _ => unreachable!(),
                }
                cpu.interrupt_disable = true;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::SLO => {
                if let ValueAddress(value, address) = addr_result {
                    let res = shift_left(value);
                    cpu.memory.write_u8(address, res.result);

                    cpu.carry_flag = res.carry_flag;

                    let acc = cpu.accumulator | res.result;

                    cpu.accumulator = acc;
                    cpu.zero_flag = acc == 0;
                    cpu.negative_flag = acc >= 128u8;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::SRE => {
                if let ValueAddress(value, address) = addr_result {
                    let res = shift_right(value);
                    cpu.memory.write_u8(address, res.result);

                    cpu.carry_flag = res.carry_flag;

                    let acc = cpu.accumulator ^ res.result;

                    cpu.accumulator = acc;
                    cpu.zero_flag = acc == 0;
                    cpu.negative_flag = acc >= 128u8;

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::STA => {
                if let Address(address) = addr_result {
                    cpu.memory.write_u8(address, cpu.accumulator);

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!("{:?}", addr_result);
                }
            }
            BaseInstruction::STX => {
                if let Address(address) = addr_result {
                    cpu.memory.write_u8(address, cpu.x);

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::STY => {
                if let Address(address) = addr_result {
                    cpu.memory.write_u8(address, cpu.y);

                    cpu.instr_cycle = 0;
                } else {
                    unreachable!();
                }
            }
            BaseInstruction::TAX => {
                assert_eq!(cpu.instr_cycle, 2);
                cpu.x = cpu.accumulator;
                cpu.zero_flag = cpu.x == 0;
                cpu.negative_flag = cpu.x >= 128;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::TAY => {
                assert_eq!(cpu.instr_cycle, 2);
                cpu.y = cpu.accumulator;
                cpu.zero_flag = cpu.y == 0;
                cpu.negative_flag = cpu.y >= 128;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::TSX => {
                assert_eq!(cpu.instr_cycle, 2);
                cpu.x = cpu.stack_pointer;
                cpu.zero_flag = cpu.x == 0;
                cpu.negative_flag = cpu.x >= 128;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::TXA => {
                assert_eq!(cpu.instr_cycle, 2);
                cpu.accumulator = cpu.x;
                cpu.zero_flag = cpu.accumulator == 0;
                cpu.negative_flag = cpu.accumulator >= 128;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::TXS => {
                assert_eq!(cpu.instr_cycle, 2);
                cpu.stack_pointer = cpu.x;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::TYA => {
                assert_eq!(cpu.instr_cycle, 2);
                cpu.accumulator = cpu.y;
                cpu.zero_flag = cpu.accumulator == 0;
                cpu.negative_flag = cpu.accumulator >= 128;

                cpu.instr_cycle = 0;
            }
            BaseInstruction::TOP => {
                cpu.instr_cycle = 0;
            }
            BaseInstruction::Unknown(byte) => {
                println!("Unknown instruction: {:#04x}", byte);
                println!("cpu status:\n{:?}", cpu);
                println!("0x6000: {}", cpu.peek(0x6000));
                let mut buffer = vec![];
                let mut j = 0x6004;
                let mut b = cpu.peek(j);
                //while b != 0 {
                for _ in 0..500 {
                    buffer.push(b);
                    b = cpu.peek(j);
                    j += 1;
                }
                let s = String::from_utf8(buffer).unwrap();
                println!("{}", s);
                if cpu.logger.is_logging() {
                    cpu.logger.print_log();
                    cpu.logger.write_log("out.txt");
                }
                panic!();
            }
        }
    }
}
