pub mod buffer;
use buffer::Buffer;

use crate::{bus::PpuAction, memory::PpuMemory};

pub struct Ppu {
    memory: PpuMemory,
    buffers: Vec<Buffer>,
    current_frame: Buffer,
    even_frame: bool,
    cycles: usize,
    x: usize,
    y: usize,
    nmi_enabled: bool,
    vblank_flag: bool,
    rendering_enabled: bool,

    sprite_pattern_table: u16,
    background_pattern_table: u16,
    vram_address_increment: u16,

    // internal registers
    reg_v: u16,
    reg_t: u16,
    reg_x: u8,
    reg_w: bool,

    // OAM stuff
    oam_addr: u8,

    // PPUCTRL
    grayscale: bool,
    show_backgroudn_leftmost: bool,
    show_sprites_leftmost: bool,
    show_background: bool,
    show_sprites: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,

    // rendering pipeline
    pattern_table_buffer1: u8,
    pattern_table_buffer2: u8,
    current_tile: u8,
    current_attribute: u8,
    pattern_table_reg1: u16,
    pattern_table_reg2: u16,
    palette_attriubutes_reg1: u16,
    palette_attriubutes_reg2: u16,

    io_latch: u8,

    pixels: usize,
}

pub struct PpuTickResult {
    pub(crate) nmi_triggered: bool,
    pub(crate) frame_ended: bool,
}

impl Ppu {
    pub(crate) fn new(memory: PpuMemory) -> Self {
        Ppu {
            memory,
            buffers: Vec::new(),
            current_frame: Buffer::empty(),
            even_frame: false,
            cycles: 0,
            x: 340,
            y: 261,
            nmi_enabled: true,
            vblank_flag: false,
            rendering_enabled: true,

            sprite_pattern_table: 0,
            background_pattern_table: 0,
            vram_address_increment: 1,

            reg_v: 0,
            reg_t: 0,
            reg_x: 0,
            reg_w: false,

            oam_addr: 0,

            grayscale: false,
            show_backgroudn_leftmost: false,
            show_sprites_leftmost: false,
            show_background: false,
            show_sprites: false,
            emphasize_red: false,
            emphasize_green: false,
            emphasize_blue: false,

            current_tile: 0,
            current_attribute: 0,
            pattern_table_buffer1: 0,
            pattern_table_buffer2: 0,
            pattern_table_reg1: 0,
            pattern_table_reg2: 0,
            palette_attriubutes_reg1: 0,
            palette_attriubutes_reg2: 0,

            io_latch: 0,

            pixels: 0,
        }
    }

    pub fn tick(&mut self, action: PpuAction) -> PpuTickResult {
        self.update_position();
        self.update_flags();

        self.handle_bus_action(action);
        // TODO:

        if self.is_rendering_enabled() {
            self.rendering();
        }

        self.cycles += 1;
        self.memory.set_v(self.reg_v);

        let mut nmi_triggered = false;
        let mut frame_ended = false;
        if self.x == 1 && self.y == 241 {
            frame_ended = true;
            println!("pixels this frame: {}", self.pixels);
            self.pixels = 0;
            if self.nmi_enabled {
                nmi_triggered = true;
            } else {
                println!("NMI disabled")
            }
        }
        PpuTickResult {
            nmi_triggered,
            frame_ended,
        }
    }

    fn rendering(&mut self) {
        if self.x == 0 {
            // Idle cycle
            return;
        }

        if self.x <= 256 {
            if self.y < 240 {
                let fine_x = 0;
                let mut color = (self.pattern_table_reg1 >> fine_x) & 1;
                color = color | ((self.pattern_table_reg2 >> fine_x) & 1) << 1;
                color = color | ((self.palette_attriubutes_reg1 >> fine_x) & 1) << 2;
                color = color | ((self.palette_attriubutes_reg2 >> fine_x) & 1) << 3;

                // TODO: render sprites here

                let (r, g, b) = self.map_background_color(color as u8);
                self.current_frame.set_pixel(self.x - 1, self.y, r, g, b);
            }
        }

        if self.y < 240 {
            if (self.x % 8 == 1)
                && ((self.x >= 9 && self.x <= 241)
                    || (self.y != 239 && (self.x == 329 || self.x == 337)))
            {
                self.merge_buffers_into_shift_registers();
            }

            if self.x >= 1 && self.x <= 256 {
                self.shift_registers();
                self.fetch_background_data();
            } else if self.x >= 321 && self.x <= 336 {
                self.shift_registers();
                self.fetch_background_data();
            }
        } else if self.y == 261 {
            if self.x == 329 || self.x == 337 {
                self.merge_buffers_into_shift_registers();
            }

            if self.x >= 321 && self.x <= 336 {
                self.shift_registers();
                self.fetch_background_data();
            }
        }

        if self.y < 240 {
            if (self.x >= 1 && self.x <= 240) || (self.x >= 321 && self.x <= 336) {
                self.increment_x();
            } else if self.x == 256 {
                self.increment_y();
                //self.pixels += 1;
            }
        } else if self.y == 261 {
            if self.x >= 321 && self.x <= 336 {
                self.increment_x();
                //self.pixels += 1;
            } else if self.x >= 280 && self.x <= 304 {
                self.copy_ty_to_v();
            }
        }

        if self.x == 257 {
            self.copy_tx_to_v();
        }
    }

    fn copy_tx_to_v(&mut self) {
        let t_x = self.reg_t & 0b00000100_00011111;
        let new_v = self.reg_v & 0b11111011_11100000;
        self.reg_v = new_v | t_x;
    }

    fn copy_ty_to_v(&mut self) {
        let t_y = self.reg_t & 0b01111011_11100000;
        let new_v = self.reg_v & 0b10000100_00011111;
        self.reg_v = new_v | t_y;
    }

    fn shift_registers(&mut self) {
        // shift bits
        self.pattern_table_reg1 >>= 1;
        self.pattern_table_reg2 >>= 1;
        self.palette_attriubutes_reg1 >>= 1;
        self.palette_attriubutes_reg2 >>= 1;
    }

    fn merge_buffers_into_shift_registers(&mut self) {
        self.pixels += 1;

        self.pattern_table_reg1 =
            (self.pattern_table_reg1 & 255) | ((self.pattern_table_buffer1 as u16) << 8);
        self.pattern_table_reg2 =
            (self.pattern_table_reg2 & 255) | ((self.pattern_table_buffer2 as u16) << 8);
        self.palette_attriubutes_reg1 =
            (self.palette_attriubutes_reg1 & 255) | (((self.current_attribute) & 1) as u16 * 255);
        self.palette_attriubutes_reg2 =
            (self.palette_attriubutes_reg2 & 255) | (((self.current_attribute) & 2) as u16 * 255);
    }

    fn fetch_background_data(&mut self) {
        match self.x % 8 {
            1 => {
                //println!("tile addr: {:#06x}", self.get_tile_address());
                self.current_tile = self.memory.read(self.get_tile_address());
            }
            3 => {
                self.current_attribute = self.memory.read(self.get_attribute_address());
            }
            5 => {
                let pattern_address =
                    self.background_pattern_table | ((self.current_tile as u16) << 4);
                let pattern_address = pattern_address | self.fine_y();
                self.pattern_table_buffer1 = self.memory.read(pattern_address);
            }
            7 => {
                let pattern_address =
                    self.background_pattern_table | ((self.current_tile as u16) << 4);
                let pattern_address = pattern_address | (self.fine_y() << 1) | 8;
                self.pattern_table_buffer2 = self.memory.read(pattern_address);
            }
            _ => {}
        }
    }

    fn map_background_color(&self, color_idx: u8) -> (u8, u8, u8) {
        // TODO: actually map color
        let c = color_idx << 4;
        (c, c, c)
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

    pub fn transfer_io_registers(&mut self) {
        self.memory.set_ppu_io_registers(self.get_io_registers())
    }

    fn get_io_registers(&self) -> PpuIoRegisters {
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

                let background_table = val >> 4;
                let background_table = background_table & 1 == 1;
                if background_table {
                    self.background_pattern_table = 0x1000;
                } else {
                    self.background_pattern_table = 0;
                }

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
                self.show_background = val & 8 != 0;
                self.show_sprites = val & 16 != 0;
                self.emphasize_red = val & 32 != 0;
                self.emphasize_green = val & 64 != 0;
                self.emphasize_blue = val & 128 != 0;
            }
            PpuAction::PpuStatusRead => {
                self.vblank_flag = false;

                self.reg_w = false;
            }
            PpuAction::OamAddrWrite(val) => {
                self.oam_addr = val;
            }
            PpuAction::OamDataWrite(_) => {
                // TODO:
            }
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
            PpuAction::PpuDataWrite(val) => {
                self.memory.write(self.reg_v, val);

                self.reg_v = self.reg_v.wrapping_add(self.vram_address_increment);
                println!("v is:  {:016b}", self.reg_v);
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
                if !self.even_frame {
                    self.x = 1;
                }
            }
        }
    }

    fn is_rendering_enabled(&self) -> bool {
        // TODO: check this
        self.show_background | self.show_sprites
    }

    fn get_tile_address(&self) -> u16 {
        let v = self.reg_v;
        let addr = 0x2000 | (v & 0x0FFF);
        //println!("{:#06x}", addr);
        addr
    }

    fn get_attribute_address(&self) -> u16 {
        let v = self.reg_v;
        0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)
    }

    fn increment_x(&mut self) {
        self.reg_x += 1;
        if self.reg_x == 8 {
            self.reg_x = 0;
            self.coarse_increment_x();
        }
    }

    fn coarse_increment_x(&mut self) {
        let mut v = self.reg_v;
        // Increment coarse X
        if (v & 0x001F) == 31 {
            // if coarse X == 31
            v &= !0x001F; // coarse X = 0
            v ^= 0x0400; // switch horizontal nametable
        } else {
            v += 1 // increment coarse X
        }

        self.reg_v = v;
    }

    fn increment_y(&mut self) {
        let mut v = self.reg_v;

        if (v & 0x7000) != 0x7000 {
            // if fine Y < 7
            v += 0x1000 // increment fine Y
        } else {
            v &= !0x7000; // fine Y = 0
            let mut y = (v & 0x03E0) >> 5; // let y = coarse Y
            if y == 29 {
                y = 0; // coarse Y = 0
                v ^= 0x0800; // switch vertical nametable
            } else if y == 31 {
                y = 0; // coarse Y = 0, nametable not switched
            } else {
                y += 1; // increment coarse Y
            }
            v = (v & !0x03E0) | (y << 5); // put coarse Y back into v
        }

        self.reg_v = v;
    }

    fn fine_y(&self) -> u16 {
        let y = self.reg_v >> 12;
        assert!(y < 8);

        y
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
