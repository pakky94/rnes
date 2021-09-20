#![allow(dead_code)]

pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod memory;
pub mod ppu;
pub mod roms;
mod utils;

use bus::{BusAction, PpuAction};
pub(crate) use cartridge::Cartridge;
use cpu::Cpu;
use ppu::{buffer::Buffer, Ppu};

pub struct Nes {
    memory: memory::MemoryHandle,
    cpu: cpu::Cpu,
    ppu: ppu::Ppu,
}

impl Nes {
    pub fn new() -> Self {
        let (memory_handle, cpu_mem, ppu_mem) = memory::create_memory();
        Self {
            memory: memory_handle,
            cpu: Cpu::new(cpu_mem),
            ppu: Ppu::new(ppu_mem),
        }
    }

    pub fn with_cartridge(cartridge: Cartridge) -> Self {
        let mut nes = Self::new();
        nes.memory.load_cartridge(cartridge);
        nes.cpu.init();
        nes
    }

    pub fn tick(&mut self) -> bool {
        self.cpu.tick();
        let cpu_bus_action = self.cpu.bus_action;

        if self.cpu.logger.is_logging() {
            if self.cpu.cycles > 28000 {
                self.cpu.logger.write_log("out.txt");
                panic!();
            }
        }

        self.memory.tick();

        let ppu_res = self.ppu.tick(PpuAction::None);
        let mut frame_end = ppu_res.frame_ended;
        let mut nmi = ppu_res.nmi_triggered;

        let ppu_action = if let BusAction::PpuAction(action) = cpu_bus_action {
            action
        } else {
            PpuAction::None
        };
        let ppu_res = self.ppu.tick(ppu_action);
        frame_end = frame_end | ppu_res.frame_ended;
        nmi = nmi || ppu_res.nmi_triggered;

        let ppu_res = self.ppu.tick(PpuAction::None);
        frame_end = frame_end | ppu_res.frame_ended;
        nmi = nmi || ppu_res.nmi_triggered;

        if nmi {
            println!("nmi triggered from ppu {}", self.cpu.debug);
            self.cpu.debug = 0;
        }

        self.cpu.nmi_flag = self.cpu.nmi_flag | nmi;

        self.ppu.transfer_io_registers();

        frame_end
    }

    pub fn run_until_frame(&mut self) {
        while !self.tick() {}
    }

    pub fn get_frame(&mut self) -> Buffer {
        self.ppu.get_frame()
    }

    pub fn return_frame(&mut self, frame: Buffer) {
        self.ppu.return_frame(frame);
    }

    pub fn enable_logging(&mut self) {
        self.cpu.logger.enable_logging();
    }
    pub fn set_pc(&mut self, pc: u16) {
        self.cpu.set_pc(pc);
    }
}
