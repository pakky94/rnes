#![allow(dead_code)]

pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod ppu;
pub mod roms;
mod utils;

use bus::{BusAction, PpuAction};
pub(crate) use cartridge::Cartridge;
use cpu::Cpu;
use ppu::{buffer::Buffer, Ppu};

pub struct Nes {
    cpu: cpu::Cpu,
    ppu: ppu::Ppu,
}

impl Nes {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
        }
    }

    pub fn with_cartridge(cartridge: Cartridge) -> Self {
        let mut nes = Self::new();
        nes.cpu.load_cartridge(cartridge);
        nes.cpu.init();
        nes
    }

    pub fn tick(&mut self) -> bool {
        self.cpu.tick();
        let cpu_bus_action = self.cpu.bus_action;

        let ppu_res = self.ppu.tick(PpuAction::None);
        let mut frame_end = ppu_res.frame_ended;

        let ppu_action = if let BusAction::PpuAction(action) = cpu_bus_action {
            action
        } else {
            PpuAction::None
        };
        let ppu_res = self.ppu.tick(ppu_action);
        frame_end = frame_end | ppu_res.frame_ended;

        let ppu_res = self.ppu.tick(PpuAction::None);
        frame_end = frame_end | ppu_res.frame_ended;

        let ppu_io_registers = self.ppu.get_io_registers();
        self.cpu.set_ppu_io_registers(ppu_io_registers);

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
}
