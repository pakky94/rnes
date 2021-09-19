pub mod buffer;
use buffer::Buffer;

use crate::bus::PpuAction;

pub struct Ppu {
    buffers: Vec<Buffer>,
    current_frame: Buffer,
    even_frame: bool,
    cycles: usize,
    x: usize,
    y: usize,
    nmi_enabled: bool,
    vblank_flag: bool,
    rendering_enabled: bool,

    vram_address_increment: u16,

    // internal registers
    reg_v: u16,
    reg_t: u16,
    reg_x: u8,
    reg_w: bool,

    // PPUCTRL
    grayscale: bool,
    show_backgroudn_leftmost: bool,
    show_sprites_leftmost: bool,
    show_backgroudn: bool,
    show_sprites: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,

    io_latch: u8,
}

pub struct PpuTickResult {
    pub(crate) nmi_triggered: bool,
    pub(crate) frame_ended: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            buffers: Vec::new(),
            current_frame: Buffer::empty(),
            even_frame: false,
            cycles: 0,
            x: 340,
            y: 261,
            nmi_enabled: true,
            vblank_flag: false,
            rendering_enabled: true,

            vram_address_increment: 1,

            reg_v: 0,
            reg_t: 0,
            reg_x: 0,
            reg_w: false,

            grayscale: false,
            show_backgroudn_leftmost: false,
            show_sprites_leftmost: false,
            show_backgroudn: false,
            show_sprites: false,
            emphasize_red: false,
            emphasize_green: false,
            emphasize_blue: false,

            io_latch: 0,
        }
    }

    pub fn tick(&mut self, action: PpuAction) -> PpuTickResult {
        self.update_position();
        self.update_flags();

        self.handle_bus_action(action);
        // TODO:

        self.cycles += 1;

        let mut nmi_triggered = false;
        let mut frame_ended = false;
        if self.x == 1 && self.y == 241 {
            frame_ended = true;
            if self.nmi_enabled {
                nmi_triggered = true;
            }
        }
        PpuTickResult {
            nmi_triggered,
            frame_ended,
        }
    }

    pub fn get_frame(&mut self) -> Buffer {
        let mut new_buffer = if let Some(buf) = self.buffers.pop() {
            buf
        } else {
            Buffer::empty()
        };

        std::mem::swap(&mut self.current_frame, &mut new_buffer);
        new_buffer
    }

    pub fn get_io_registers(&self) -> PpuIoRegisters {
        let status = if self.vblank_flag { 128 } else { 0 };

        PpuIoRegisters {
            status,
            last_written: self.io_latch,
        }
    }

    pub fn return_frame(&mut self, buffer: Buffer) {
        self.buffers.push(buffer);
    }

    fn handle_bus_action(&mut self, action: PpuAction) {
        match action {
            PpuAction::PpuCtrlWrite(val) => {
                println!("t was: {:016b}", self.reg_t);
                self.io_latch = val;

                let t_temp = self.reg_t & 0b11110011_11111111;
                let new_t = ((val & 0b00000011) as u16) << 10;
                self.reg_t = t_temp | new_t;

                let nmi_enable = val & 128 != 0;
                self.nmi_enabled = nmi_enable;

                self.vram_address_increment = if val & 4 == 0 { 1 } else { 32 };
                println!("t is:  {:016b}", self.reg_t);
            }
            PpuAction::PpuMaskWrite(val) => {
                self.io_latch = val;

                self.grayscale = val & 1 != 0;
                self.show_backgroudn_leftmost = val & 2 != 0;
                self.show_sprites_leftmost = val & 4 != 0;
                self.show_backgroudn = val & 8 != 0;
                self.show_sprites = val & 16 != 0;
                self.emphasize_red = val & 32 != 0;
                self.emphasize_green = val & 64 != 0;
                self.emphasize_blue = val & 128 != 0;
            }
            PpuAction::PpuStatusRead => {
                self.vblank_flag = false;

                self.reg_w = false;
            }
            PpuAction::OmaAddrWrite(_) => todo!(),
            PpuAction::OamDataWrite(_) => todo!(),
            PpuAction::PpuScrollWrite(val) => {
                println!("t was: {:016b}", self.reg_t);
                if self.reg_w == false {
                    let t_temp = self.reg_t & 0b11111111_11100000;
                    let new_t = (val as u16) >> 3;
                    self.reg_t = t_temp | new_t;

                    self.reg_x = val & 0b00000111;
                    self.reg_w = true;
                } else {
                    let t_temp = self.reg_t & 0b00001100_00011111;
                    let fgh = ((val & 0b00000111) as u16) << 12;
                    let abcde = ((val & 0b11111000) as u16) << 2;
                    self.reg_t = t_temp | fgh | abcde;

                    self.reg_w = false;
                }
                println!("t is:  {:016b}", self.reg_t);
            }
            PpuAction::PpuAddrWrite(val) => {
                println!("t was: {:016b}", self.reg_t);
                if self.reg_w == false {
                    let t_temp = self.reg_t & 0b10000000_11111111;
                    let cdefgh = ((val & 0b00111111) as u16) << 8;
                    self.reg_t = t_temp | cdefgh;

                    self.reg_w = true;
                } else {
                    let t_temp = self.reg_t & 0b11111111_00000000;
                    self.reg_t = t_temp | (val as u16);
                    self.reg_v = self.reg_t;

                    self.reg_w = false;
                }
                println!("t is:  {:016b}", self.reg_t);
            }
            PpuAction::PpuDataRead => {


                self.reg_v = self.reg_v.wrapping_add(self.vram_address_increment);
                todo!();
            }
            PpuAction::PpuDataWrite(_) => {
                

                self.reg_v = self.reg_v.wrapping_add(self.vram_address_increment);
                todo!();
            }
            PpuAction::OamDmaWrite(_) => todo!(),
            PpuAction::None => {}
        }
    }

    fn update_flags(&mut self) {
        if self.x == 1 {
            if self.y == 241 {
                println!("VBLANK STARTED");
                self.vblank_flag = true;
            } else if self.y == 261 {
                println!("VBLANK ENDED");
                self.vblank_flag = false;
            }
        }
    }

    fn update_position(&mut self) {
        self.x += 1;
        if self.x > 340 {
            self.x = 0;
            self.y += 1;
            if self.y > 261 {
                self.y = 0;
                self.even_frame = !self.even_frame;
            }
        }
    }

    fn is_rendering_enabled(&self) -> bool {
        // TODO: check this
        self.show_backgroudn | self.show_sprites
    }
}

#[derive(Clone, Copy)]
pub struct PpuIoRegisters {
    pub(crate) status: u8,
    pub(crate) last_written: u8,
}

impl PpuIoRegisters {
    pub fn new() -> Self {
        Self {
            status: 0,
            last_written: 0,
        }
    }
}
