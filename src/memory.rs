use std::{cell::RefCell, rc::Rc};

use crate::cpu::addresses::{EXPANSION_ROM, IO_REGISTERS_START, PRG_ROM_LOWER};
use crate::input::{Controller, InputData};
use crate::ppu::PpuIoRegisters;
pub(crate) use crate::{
    bus::{BusAction, PpuAction},
    Cartridge,
};

struct MemoryInt {
    cpu_memory: Vec<u8>,
    internal_vram: [u8; 0x800],
    palette_ram_indexes: [u8; 0x20],
    cartridge: Cartridge,
    bus_action: BusAction,
    ppudata_buffer: u8,
    oam_memory: [u8; 0x100],
    oam_address_mirror: u8,
    dma_address: u16,
    dma_buffer: u8,
    dma_cycle: usize,
    dma_offset: u8,
    ppu_io_registers: PpuIoRegisters,
    ppu_v: u16,
    controller1: Controller,
}
impl MemoryInt {
    fn new() -> Self {
        Self {
            // memory: [0u8; 2 ^ 16],
            cpu_memory: vec![0x00; 0x10000],
            internal_vram: [0x00; 0x800],
            palette_ram_indexes: [0x00; 0x20],
            cartridge: Cartridge::empty(),
            bus_action: BusAction::None,
            ppudata_buffer: 0,
            oam_memory: [0u8; 0x100],
            oam_address_mirror: 0,
            dma_address: 0,
            dma_buffer: 0,
            dma_cycle: 1000, // 0-511 -> dma running, else ended
            dma_offset: 0,
            ppu_io_registers: PpuIoRegisters::new(),
            ppu_v: 0,
            controller1: Controller::new(),
        }
    }

    fn from_vec(vec: Vec<u8>) -> Self {
        let mut out = Self::new();
        out.cpu_memory = vec;
        out
    }

    fn cpu_read_unchanged_u8(&self, address: u16) -> u8 {
        let address = Self::cpu_unmirror_address(address);

        // if address < CARTRIDGE_SPACE {
        if address < PRG_ROM_LOWER {
            self.cpu_memory[address as usize]
        } else {
            self.cartridge.cpu_read(address)
        }
    }

    fn cpu_read_u8(&mut self, address: u16) -> u8 {
        let address = Self::cpu_unmirror_address(address);

        // if address < CARTRIDGE_SPACE {
        if address < IO_REGISTERS_START {
            self.cpu_memory[address as usize]
        } else if address < EXPANSION_ROM {
            // IO register stuff
            if address < 0x4000 {
                // mirrors of 0x2000-0x2008
                let address = address % 8 + 0x2000;
                match address {
                    0x2000 => self.ppu_io_registers.last_written,
                    0x2001 => self.ppu_io_registers.last_written,
                    0x2002 => {
                        // PPU STATUS
                        self.bus_action = BusAction::PpuAction(PpuAction::PpuStatusRead);
                        if self.ppu_io_registers.status != 0 {
                            println!(
                                "Reading from PPUSTATUS: {:08b}",
                                self.ppu_io_registers.status
                            );
                        }
                        self.ppu_io_registers.status
                    }
                    0x2003 => self.ppu_io_registers.last_written,
                    0x2004 => {
                        // OAM data
                        self.oam_memory[self.oam_address_mirror as usize]
                    }
                    0x2005 => self.ppu_io_registers.last_written,
                    0x2006 => self.ppu_io_registers.last_written,
                    0x2007 => {
                        // Data
                        let data = self.ppu_read(self.ppu_v);
                        let out = self.ppudata_buffer;
                        self.ppudata_buffer = data;
                        self.bus_action = BusAction::PpuAction(PpuAction::PpuDataRead);
                        println!("Reading from PPUDATA: {:08b}", data);
                        out
                    }
                    _ => unreachable!("Reading from address {:#06x}", address),
                }
            } else {
                // 0x4000-0x4020
                match address {
                    0x4016 => {
                        // Controller 1
                        let val = self.controller1.read_from();
                        println!("Reading from controller 1: {}", val);
                        val
                    }
                    _ => {
                        println!("Reading from I/O register: {:#06x}", address);
                        0
                    }
                }
            }
        } else if address < PRG_ROM_LOWER {
            self.cpu_memory[address as usize]
        } else {
            self.cartridge.cpu_read(address)
        }
    }

    fn cpu_write_u8(&mut self, address: u16, value: u8) {
        let address = Self::cpu_unmirror_address(address);

        // if address < CARTRIDGE_SPACE {
        if address < IO_REGISTERS_START {
            self.cpu_memory[address as usize] = value;
        } else if address < EXPANSION_ROM {
            // IO register stuff
            if address < 0x4000 {
                // mirrors of 0x2000-0x2008
                let address = address % 8 + 0x2000;
                match address {
                    0x2000 => {
                        println!("Writing to PPUCTRL: {:08b}", value);
                        self.bus_action = BusAction::PpuAction(PpuAction::PpuCtrlWrite(value));
                    }
                    0x2001 => {
                        println!("Writing to PPUMASK: {:08b}", value);
                        self.bus_action = BusAction::PpuAction(PpuAction::PpuMaskWrite(value));
                    }
                    0x2002 => {
                        println!("Writing to status: {:08b}", value);
                        //self.bus_action = BusAction::PpuAction(PpuAction::PpuMaskWrite(value));
                    }
                    0x2003 => {
                        println!("Writing to OAMADDR: {:08b}", value);
                        self.bus_action = BusAction::PpuAction(PpuAction::OamAddrWrite(value));
                        if value != 0 {
                            panic!()
                        }
                    }
                    0x2004 => {
                        println!("Writing to OAMDATA: {:08b}", value);
                        self.bus_action = BusAction::PpuAction(PpuAction::OamDataWrite(value));
                    }
                    0x2005 => {
                        println!("Writing to PPUSCROLL: {:08b}", value);
                        self.bus_action = BusAction::PpuAction(PpuAction::PpuScrollWrite(value))
                    }
                    0x2006 => {
                        println!("Writing to PPUADDR: {:08b}", value);
                        self.bus_action = BusAction::PpuAction(PpuAction::PpuAddrWrite(value))
                    }
                    0x2007 => {
                        println!("Writing to PPUDATA: {:08b}, addr: {:#06x}", value, self.ppu_v);
                        self.bus_action = BusAction::PpuAction(PpuAction::PpuDataWrite(value))
                    }
                    _ => unreachable!(
                        "Writing to PPU at address {:#06x} value: {:08b}",
                        address, value
                    ),
                }
            } else {
                // 0x4000-0x4020
                match address {
                    0x4014 => {
                        self.dma_address = (value as u16) << 8;
                        self.dma_cycle = 0;
                        self.dma_offset = 0;
                    }
                    0x4016 => {
                        self.controller1.write_to(value);
                        println!("Writing to controller: {:#04x}", value);
                    }
                    _ => println!("Writing to I/O register: {:#06x}, {:#04x}", address, value),
                }
            }
        } else if address < PRG_ROM_LOWER {
            self.cpu_memory[address as usize] = value;
        } else {
            self.cartridge.cpu_write(address, value);
        }
    }

    fn cpu_read_u16(&mut self, address: u16) -> u16 {
        let lower = self.cpu_read_u8(address);
        let higher = self.cpu_read_u8(address + 1);
        (higher as u16) << 8 | (lower as u16)
    }

    fn cpu_unmirror_address(address: u16) -> u16 {
        //return address;
        if address < 0x2000 {
            address % 0x0800
        } else if address < 0x4000 {
            (address % 0x0008) + 0x2000
        } else {
            address
        }
    }

    fn cpu_take_bus_action(&mut self) -> BusAction {
        let bus_action = self.bus_action;
        self.bus_action = BusAction::None;
        bus_action
    }

    fn set_ppu_io_registers(&mut self, regs: PpuIoRegisters) {
        self.ppu_io_registers = regs;
    }

    fn ppu_write(&mut self, address: u16, value: u8) {
        let address = Self::ppu_unmirror_address_write(address);
        if address < 0x3F00 {
            self.cartridge
                .ppu_write(address, value, &mut self.internal_vram);
        } else {
            self.palette_ram_indexes[address as usize - 0x3F00] = value;
        }
    }

    fn ppu_read(&mut self, address: u16) -> u8 {
        let address = Self::ppu_unmirror_address_read(address);
        if address < 0x3F00 {
            self.cartridge.ppu_read(address, &self.internal_vram)
        } else {
            self.palette_ram_indexes[address as usize - 0x3F00]
        }
    }

    fn ppu_oam_read(&mut self, address: usize) -> u8 {
        self.oam_memory[address]
    }

    fn ppu_oam_write(&mut self, address: usize, value: u8) {
        self.oam_memory[address] = value;
    }

    fn ppu_unmirror_address_write(address: u16) -> u16 {
        let address = address % 0x4000;

        if address >= 0x3F00 {
            if address % 4 == 0 {
                0x3F00 | address % 0x10
            } else {
                (address % 0x20) + 0x3F00
            }
        } else if address >= 0x3000 {
            address - 0x1000
        } else {
            address
        }
    }

    fn ppu_unmirror_address_read(address: u16) -> u16 {
        let address = address % 0x4000;

        if address >= 0x3F00 {
            if address % 4 == 0 {
                //0x3F00 | (address & 0x10)
                0x3F00 | address % 0x10
            } else {
                (address % 0x20) + 0x3F00
            }
        } else if address >= 0x3000 {
            address - 0x1000
        } else {
            address
        }
    }

    fn try_dma(&mut self) -> bool {
        if self.dma_cycle < 512 {
            if self.dma_cycle % 2 == 0 {
                self.dma_buffer = self.cpu_read_u8(self.dma_address + self.dma_offset as u16);
            } else {
                self.oam_memory[self.dma_offset as usize] = self.dma_buffer;
                self.dma_offset += 1;
                //println!("OAM addr {:#04x} -> {:#04x}", self.dma_offset, self.dma_buffer);
            }
            self.dma_cycle += 1;
            true
        } else {
            false
        }
    }

    fn set_controller1_input(&mut self, input_data: InputData) {
        self.controller1.set_input(input_data);
    }
}

pub struct CpuMemory(Rc<RefCell<MemoryInt>>);
impl CpuMemory {
    pub(crate) fn read_unchanged_u8(&self, address: u16) -> u8 {
        self.0.borrow().cpu_read_unchanged_u8(address)
    }

    pub(crate) fn read_u8(&mut self, address: u16) -> u8 {
        self.0.as_ref().borrow_mut().cpu_read_u8(address)
    }

    pub(crate) fn write_u8(&mut self, address: u16, value: u8) {
        self.0.as_ref().borrow_mut().cpu_write_u8(address, value)
    }

    pub(crate) fn read_u16(&mut self, address: u16) -> u16 {
        self.0.as_ref().borrow_mut().cpu_read_u16(address)
    }

    pub(crate) fn take_bus_action(&mut self) -> BusAction {
        self.0.as_ref().borrow_mut().cpu_take_bus_action()
    }

    pub(crate) fn try_dma(&mut self) -> bool {
        self.0.as_ref().borrow_mut().try_dma()
    }
}

pub struct PpuMemory(Rc<RefCell<MemoryInt>>);
impl PpuMemory {
    pub(crate) fn set_ppu_io_registers(&mut self, regs: PpuIoRegisters) {
        self.0.as_ref().borrow_mut().set_ppu_io_registers(regs);
    }

    pub(crate) fn set_v(&mut self, v: u16) {
        self.0.as_ref().borrow_mut().ppu_v = v;
    }

    pub(crate) fn read_oam(&mut self, address: usize) -> u8 {
        self.0.as_ref().borrow_mut().ppu_oam_read(address)
    }

    pub(crate) fn write_oam(&mut self, address: usize, value: u8) {
        self.0.as_ref().borrow_mut().ppu_oam_write(address, value);
    }

    pub(crate) fn read(&mut self, address: u16) -> u8 {
        self.0.as_ref().borrow_mut().ppu_read(address)
    }

    pub(crate) fn write(&mut self, address: u16, value: u8) {
        self.0.as_ref().borrow_mut().ppu_write(address, value);
    }

    pub(crate) fn print_oam(&self) {
        println!("oam: {:?}", self.0.as_ref().borrow().oam_memory);
    }

    pub(crate) fn oam_address_mirror_read(&self) -> u8 {
        self.0.as_ref().borrow().oam_address_mirror
    }

    pub(crate) fn oam_address_mirror_write(&self, addr : u8) {
        self.0.as_ref().borrow_mut().oam_address_mirror = addr;
    }
}

pub struct MemoryHandle(Rc<RefCell<MemoryInt>>);
impl MemoryHandle {
    pub(crate) fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.0.as_ref().borrow_mut().cartridge = cartridge;
    }

    pub(crate) fn tick(&mut self) {
        self.0.as_ref().borrow_mut().controller1.tick();
        self.0.as_ref().borrow_mut().cartridge.tick();
    }

    pub(crate) fn set_controller1_data(&mut self, input_data: InputData) {
        self.0.as_ref().borrow_mut().controller1.set_input(input_data);
    }
}

pub fn create_memory() -> (MemoryHandle, CpuMemory, PpuMemory) {
    let m = Rc::new(RefCell::new(MemoryInt::new()));
    (
        MemoryHandle(Rc::clone(&m)),
        CpuMemory(Rc::clone(&m)),
        PpuMemory(Rc::clone(&m)),
    )
}
