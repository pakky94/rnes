mod addressing_mode;
mod instructions;
mod logger;

pub use logger::Logger;

use crate::{bus::BusAction, memory::CpuMemory};
pub(crate) use addressing_mode::{AddressingMode, AddressingResult};
use instructions::Instruction;

pub(crate) mod addresses {
    pub(crate) const ZERO_PAGE_START: u16 = 0x0000;
    pub(crate) const STACK: u16 = 0x0100;
    pub(crate) const IO_REGISTERS_START: u16 = 0x2000;
    pub(crate) const CARTRIDGE_SPACE: u16 = 0x4020;
    pub(crate) const EXPANSION_ROM: u16 = 0x4020;
    pub(crate) const SAVE_RAM: u16 = 0x6000;
    pub(crate) const PRG_ROM_LOWER: u16 = 0x8000;
    pub(crate) const PRG_ROM_UPPER: u16 = 0xC000;
    pub(crate) const NMI_VECTOR: u16 = 0xFFFA;
    pub(crate) const RESET_VECTOR: u16 = 0xFFFC;
    pub(crate) const IRQ_BRK_VECTOR: u16 = 0xFFFE;
}

#[derive(Clone, Copy)]
pub struct CpuStatus {
    program_counter: u16,
    stack_pointer: u8,
    accumulator: u8,
    x: u8,
    y: u8,
    p: u8,
    cycles: usize,
    current_instr: Instruction,
    instr_cycle: usize,
}

pub struct Cpu {
    program_counter: u16,
    stack_pointer: u8,
    accumulator: u8,
    x: u8,
    y: u8,
    carry_flag: bool,
    zero_flag: bool,
    interrupt_disable: bool,
    decimal_mode: bool,
    break_command: bool,
    overflow_flag: bool,
    negative_flag: bool,
    pub(crate) cycles: usize,
    memory: CpuMemory,
    current_instr: Instruction,
    instr_cycle: usize,
    pub logger: Logger,
    pub(crate) bus_action: BusAction,
    pub(crate) nmi_flag: bool,
    nmi: bool,
    pub debug: usize,
}

impl Cpu {
    pub fn new(memory: CpuMemory) -> Self {
        Self {
            program_counter: 0,
            stack_pointer: 0,
            accumulator: 0,
            x: 0,
            y: 0,
            carry_flag: false,
            zero_flag: false,
            interrupt_disable: false,
            decimal_mode: false,
            break_command: false,
            overflow_flag: false,
            negative_flag: false,
            cycles: 0,
            memory,
            current_instr: Instruction::no_op(),
            instr_cycle: 0,
            logger: Logger::new(),
            bus_action: BusAction::None,
            nmi_flag: false,
            nmi: false,
            debug: 0,
        }
    }

    pub fn init(&mut self) {
        self.program_counter = self.memory.read_u16(addresses::RESET_VECTOR);
        println!(
            "starting address: {:#06x} => {:#04x}\n",
            self.program_counter,
            self.memory.read_u8(self.program_counter)
        );
        self.stack_pointer = self.stack_pointer.wrapping_sub(3);
        // TODO: initialize state
        self.reset_processor_status();
    }

    pub fn peek(&self, address: u16) -> u8 {
        self.memory.read_unchanged_u8(address)
    }

    pub fn get_acc(&self) -> u8 {
        self.accumulator
    }

    pub fn get_x(&self) -> u8 {
        self.x
    }

    pub fn get_y(&self) -> u8 {
        self.y
    }

    pub fn set_pc(&mut self, address: u16) {
        self.program_counter = address;
    }

    pub fn get_pc(&self) -> u16 {
        self.program_counter
    }

    pub fn get_sp(&self) -> u8 {
        self.stack_pointer
    }

    pub fn get_cycles(&self) -> usize {
        self.cycles
    }

    pub fn get_cpu_status(&self) -> CpuStatus {
        CpuStatus {
            program_counter: self.program_counter,
            stack_pointer: self.stack_pointer,
            accumulator: self.accumulator,
            x: self.x,
            y: self.y,
            p: self.get_processor_status(),
            cycles: self.cycles,
            current_instr: self.current_instr,
            instr_cycle: self.instr_cycle,
        }
    }

    pub fn tick(&mut self) {
        if !self.memory.try_dma() {
            if self.instr_cycle == 0 {
                if self.nmi {
                    let _ = self.memory.read_u8(self.program_counter);
                    self.current_instr = Instruction::from_opcode(0x00); // force BRK instruction
                    // Don't increment PC
                    self.instr_cycle = 2;
                    
                    self.nmi_flag = false;
                } else {
                    let opcode = self.memory.read_u8(self.program_counter);
                    if self.logger.is_logging() {
                        self.logger.start_new_instr(self.program_counter, opcode);
                        self.logger.set_proc_status(self.get_cpu_status());
                    }
                    self.current_instr = Instruction::from_opcode(opcode);
                    self.program_counter = self.program_counter.wrapping_add(1);
                    self.instr_cycle = 2;
                }
            } else {
                Instruction::tick(self);
            }
        }
        self.cycles += 1;

        self.bus_action = self.memory.take_bus_action();
    }

    pub(crate) fn pool_interrupts(&mut self) {
        // TODO: everithing
        self.debug += 1;
        if self.debug % 100000 == 0 {
            println!("{}", self.debug);
        }
        if self.nmi_flag {
            self.nmi = true;
            println!("NMI triggered");
        }
    }

    fn interrupt(&mut self) {
        todo!("handle interrupt")
    }

    pub(crate) fn stack_address(&self) -> u16 {
        self.stack_pointer as u16 + addresses::STACK
    }

    pub(crate) fn decrement_stack_pointer(&mut self) {
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    pub(crate) fn increment_stack_pointer(&mut self) {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
    }

    /// Returns the P register of the processor
    /// NB: bit 5 of the status register is always 1
    pub fn get_processor_status(&self) -> u8 {
        let mut out = 1u8 << 5;
        if self.negative_flag {
            out = out | (1u8 << 7);
        }
        if self.overflow_flag {
            out = out | (1u8 << 6);
        }
        if self.break_command {
            out = out | (1u8 << 4);
        }
        if self.decimal_mode {
            out = out | (1u8 << 3);
        }
        if self.interrupt_disable {
            out = out | (1u8 << 2);
        }
        if self.zero_flag {
            out = out | (1u8 << 1);
        }
        if self.carry_flag {
            out = out | 1u8;
        }
        out
    }

    fn reset_processor_status(&mut self) {
        self.carry_flag = false;
        self.zero_flag = false;
        self.interrupt_disable = true;
        self.decimal_mode = false;
        self.break_command = false;
        self.overflow_flag = false;
        self.negative_flag = false;
    }

    fn set_processor_status(&mut self, status: u8) {
        //self.reset_processor_status();
        self.carry_flag = (status & 1u8) != 0;
        self.zero_flag = (status & 2u8) != 0;
        self.interrupt_disable = (status & 4u8) != 0;
        self.decimal_mode = (status & 8u8) != 0;
        self.break_command = (status & 16u8) != 0;
        self.overflow_flag = (status & 64u8) != 0;
        self.negative_flag = (status & 128u8) != 0;
    }
}

impl std::fmt::Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PC:  {:#06x}   [{:#04x}]\n",
            self.program_counter,
            self.peek(self.program_counter)
        )?;
        write!(
            f,
            "SP:  {:#04x}   value: {:#04x}\n",
            self.stack_pointer,
            self.memory.read_unchanged_u8(self.stack_address() + 1)
        )?;
        write!(f, "Acc: {:#04x}\n", self.accumulator)?;
        write!(f, "X:   {:#04x}\n", self.x)?;
        write!(f, "Y:   {:#04x}\n", self.y)?;
        write!(f, "cyc: {}\n", self.cycles)?;
        write!(f, "flags {:#010b}\n", self.get_processor_status())?;
        write!(f, "        NV BDIZC\n")?;
        write!(f, "current instruction: {:?}\n", self.current_instr)
    }
}
