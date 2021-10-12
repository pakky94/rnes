#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io::{Read, Write};

pub mod apu;
pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod input;
pub mod memory;
pub mod ppu;
pub mod roms;
mod utils;

use apu::Apu;
use bus::{BusAction, PpuAction};
pub(crate) use cartridge::Cartridge;
use cpu::Cpu;
use input::InputData;
use ppu::{buffer::Buffer, Ppu};

pub struct Nes {
    memory: memory::MemoryHandle,
    apu: apu::Apu,
    cpu: cpu::Cpu,
    ppu: ppu::Ppu,
    last_cycle: usize,
    running: bool,
}

impl Nes {
    pub fn new() -> Self {
        let (memory_handle, apu_mem, cpu_mem, ppu_mem) = memory::create_memory();
        Self {
            memory: memory_handle,
            apu: Apu::new(apu_mem),
            cpu: Cpu::new(cpu_mem),
            ppu: Ppu::new(ppu_mem),
            last_cycle: 0,
            running: false,
        }
    }

    pub fn cycles(&self) -> usize {
        self.cpu.cycles
    }

    pub fn with_cartridge(cartridge: Cartridge) -> Self {
        let mut nes = Self::new();
        nes.memory.load_cartridge(cartridge);
        nes.cpu.init();
        nes.running = true;
        nes
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.memory.load_cartridge(cartridge);
        self.cpu.init();
        self.running = true;
    }

    pub fn tick(&mut self) -> bool {
        self.cpu.tick();
        let cpu_bus_action = self.cpu.bus_action;

        self.memory.tick();

        self.apu.tick(cpu_bus_action);

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

        //if frame_end {
        //let delta = self.cpu.cycles - self.last_cycle;
        //self.last_cycle = self.cpu.cycles;
        //eprintln!("cycles: {}", delta);
        //}

        frame_end
    }

    pub fn initialize_audio(
        &mut self,
        audio_subsystem: &sdl2::AudioSubsystem,
        audio_specs: &sdl2::audio::AudioSpecDesired,
    ) -> Result<sdl2::audio::AudioDevice<apu::AudioGenerator>, String> {
        audio_subsystem.open_playback(None, &audio_specs, |spec| {
            apu::AudioGenerator::from_spec(spec)
        })
    }

    pub fn update_audio_generator(
        &mut self,
        current_generator: sdl2::audio::AudioDeviceLockGuard<apu::AudioGenerator>,
    ) {
        self.apu.update_audio_generator(current_generator);
    }

    pub fn render_pattern_table(&mut self, address: u16, palette_idx: usize) -> Buffer {
        self.ppu.render_pattern_table(address, palette_idx)
    }

    pub fn render_nametable(&mut self, idx: usize) -> Buffer {
        self.ppu.render_nametable(idx)
    }

    pub fn render_palettes(&mut self) -> Buffer {
        self.ppu.render_palettes()
    }

    pub fn run_until_frame(&mut self) {
        if self.running {
            while !self.tick() {}
        }
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

    pub fn disable_logging(&mut self) {
        self.cpu.logger.disable_logging();
    }

    pub fn write_cpu_logs(&mut self, filename: &str) {
        self.cpu.logger.write_log(filename);
    }

    pub fn clear_logs(&mut self) {
        self.cpu.logger.clear();
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.cpu.set_pc(pc);
    }

    pub fn set_input1(&mut self, input_data: InputData) {
        self.memory.set_controller1_data(input_data);
    }

    pub fn save_data(&self, path: &str) {
        let data = self.memory.get_save_data();
        let mut f = File::create(path).unwrap();
        f.write_all(&data[..]).unwrap();
    }

    pub fn try_load_data(&mut self, path: &str) {
        if let Ok(mut f) = File::open(path) {
            let mut data = vec![];
            if let Ok(_) = f.read_to_end(&mut data) {
                self.memory.set_save_data(data);
            }
        }
    }
}
