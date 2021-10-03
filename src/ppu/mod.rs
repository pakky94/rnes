pub mod buffer;
mod sprite_unit;

use self::sprite_unit::SpriteUnit;
use crate::{bus::PpuAction, memory::PpuMemory};
use buffer::Buffer;

const IMAGE_COLOR_PALETTE_ADDRESS: u16 = 0x3F00;
const SPRITE_COLOR_PALETTE_ADDRESS: u16 = 0x3F10;
const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

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

    sprite_zero_hit: bool,

    sprite_height: u8,
    sprite_pattern_table: u16,
    background_pattern_table: u16,
    sprites_pattern_table: u16,
    vram_address_increment: u16,

    // internal registers
    reg_v: u16,
    reg_t: u16,
    reg_x: u8,
    reg_w: bool,

    fine_x: u8,
    x_increment_counter: u8,

    // OAM stuff
    current_line_has_sprite_zero: bool,
    next_line_has_sprite_zero: bool,
    sprite_evaluation_state: SpriteEvaluationState,
    oam_n: usize,
    secondary_oam: [u8; 0x20],
    secondary_oam_pointer: usize,
    oam_addr: u8,
    oam_buffer: u8,
    oam_initializing: bool,

    sprite_units: [SpriteUnit; 8],
    sprite_pattern_address: u16,

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
    current_attribute_quadrant: u8,
    current_attribute_address: u16,
    pattern_table_reg1: u16,
    pattern_table_reg2: u16,
    palette_attribute_buffer1: u8,
    palette_attribute_buffer2: u8,
    palette_attriubutes_reg1: u16,
    palette_attriubutes_reg2: u16,

    io_latch: u8,

    pixels: usize,
}

pub struct PpuTickResult {
    pub(crate) nmi_triggered: bool,
    pub(crate) frame_ended: bool,
}

enum SpriteEvaluationState {
    Read0(usize),
    Copy0(usize),
    Read1(usize),
    Copy1(usize),
    Read2(usize),
    Copy2(usize),
    Read3(usize),
    Copy3(usize),
}

impl Ppu {
    pub(crate) fn new(memory: PpuMemory) -> Self {
        Ppu {
            memory,
            buffers: Vec::new(),
            current_frame: Buffer::empty(SCREEN_WIDTH, SCREEN_HEIGHT),
            even_frame: false,
            cycles: 0,
            x: 340,
            y: 261,
            nmi_enabled: true,
            vblank_flag: false,
            rendering_enabled: true,

            sprite_zero_hit: false,

            sprite_height: 8,
            sprite_pattern_table: 0,
            background_pattern_table: 0,
            sprites_pattern_table: 0,
            vram_address_increment: 1,

            reg_v: 0,
            reg_t: 0,
            reg_x: 0,
            reg_w: false,

            fine_x: 0,
            x_increment_counter: 0,

            current_line_has_sprite_zero: false,
            next_line_has_sprite_zero: false,
            sprite_evaluation_state: SpriteEvaluationState::Read0(0),
            oam_n: 0,
            secondary_oam: [0u8; 0x20],
            secondary_oam_pointer: 0,
            oam_addr: 0,
            oam_buffer: 0,
            oam_initializing: false,

            sprite_units: SpriteUnit::units(),
            sprite_pattern_address: 0,

            grayscale: false,
            show_backgroudn_leftmost: false,
            show_sprites_leftmost: false,
            show_background: false,
            show_sprites: false,
            emphasize_red: false,
            emphasize_green: false,
            emphasize_blue: false,

            current_tile: 0,
            current_attribute_quadrant: 0,
            current_attribute_address: 0,
            pattern_table_buffer1: 0,
            pattern_table_buffer2: 0,
            pattern_table_reg1: 0,
            pattern_table_reg2: 0,
            palette_attribute_buffer1: 0,
            palette_attribute_buffer2: 0,
            palette_attriubutes_reg1: 0,
            palette_attriubutes_reg2: 0,

            io_latch: 0,

            pixels: 0,
        }
    }

    pub fn tick(&mut self, action: PpuAction) -> PpuTickResult {
        self.oam_addr = self.memory.oam_address_mirror_read();
        self.update_position();
        self.update_flags();

        self.handle_bus_action(action);
        // TODO:

        self.sprite_evaluation();
        if self.is_rendering_enabled() {
            self.rendering();
        }

        self.cycles += 1;

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
        self.memory.oam_address_mirror_write(self.oam_addr);
        self.memory.set_v(self.reg_v);
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

        if self.y < 240 {
            if self.x >= 2 && self.x <= 257 {
                self.shift_registers();
                self.shift_sprites();
            } else if self.x >= 322 && self.x <= 337 {
                self.shift_registers();
            }
        } else if self.y == 261 {
            if self.x >= 322 && self.x <= 337 {
                //if self.x == 322 {
                //println!("fine_x: {}", self.fine_x);
                //self.fine_x = self.reg_x;
                //}
                self.shift_registers();
            }
        }

        if self.x <= 256 {
            if self.y < 240 {
                let fine_x = self.reg_x;
                let mut bg_color = (self.pattern_table_reg1 >> (15 - fine_x)) & 1;
                bg_color = bg_color | ((self.pattern_table_reg2 >> (15 - fine_x)) & 1) << 1;
                //color = color | 0b0000;
                bg_color = bg_color | ((self.palette_attriubutes_reg1 >> (15 - fine_x)) & 1) << 2;
                bg_color = bg_color | ((self.palette_attriubutes_reg2 >> (15 - fine_x)) & 1) << 3;

                // TODO: render sprites here
                let (sprite_color, sprite_fg, sprite_zero) = self.get_sprite_color();

                if sprite_zero && (sprite_color & 3 != 0) && (bg_color & 3 != 0) {
                    self.sprite_zero_hit = true;
                }

                //if (self.y % 8 == 0) && (self.x % 8 < 4) {
                //self.current_frame
                //.set_pixel(self.x - 1, self.y, 0, 0xFF, 0xFF);
                //} else {
                let (r, g, b) = if (sprite_color & 0x03) == 0 || (!sprite_fg && bg_color & 0x03 != 0) {
                    if bg_color & 0x03 == 0 {
                        bg_color = 0;
                    }
                    self.map_background_color(bg_color as u8)
                } else {
                    //println!("sprite_color: {:08b}, fg: {}", sprite_color, sprite_fg);
                    self.map_sprite_color(sprite_color)
                };

                self.current_frame.set_pixel(self.x - 1, self.y, r, g, b);
                //}
            }
        }

        if self.y < 240 {
            if (self.x % 8 == 1)
                && ((self.x >= 9 && self.x <= 257) || (self.x == 329 || self.x == 337))
            {
                self.merge_buffers_into_shift_registers();
            }

            if self.x >= 1 && self.x <= 256 {
                //self.shift_registers();
                self.fetch_background_data();
            } else if self.x >= 321 && self.x <= 336 {
                //self.shift_registers();
                self.fetch_background_data();
            }
        } else if self.y == 261 {
            if self.x == 329 || self.x == 337 {
                self.merge_buffers_into_shift_registers();
            }

            if self.x >= 321 && self.x <= 336 {
                //self.shift_registers();
                self.fetch_background_data();
            }
        }

        if self.y < 240 {
            if (self.x >= 1 && self.x <= 240) || (self.x >= 321 && self.x <= 336) {
                self.increment_x();
            } else if self.x == 256 {
                self.increment_y();
                self.pixels += 1;
            }
        } else if self.y == 261 {
            if self.x == 1 {
                self.sprite_zero_hit = false;
            }
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

    fn sprite_evaluation(&mut self) {
        //if self.x == 0 || (self.y != 261 && self.y >= 240) {
        if self.x == 0 || self.y >= 240 {
            return;
        }

        if self.x == 1 {
            self.oam_n = 0;
            self.secondary_oam_pointer = 0;
            self.sprite_evaluation_state = SpriteEvaluationState::Read0(0);
            self.current_line_has_sprite_zero = self.next_line_has_sprite_zero;
            self.next_line_has_sprite_zero = false;
        }

        //let eval_y = if self.y == 261 { 0 } else { self.y + 1 };
        let eval_y = self.y;

        if self.x <= 64 {
            if self.x % 2 == 0 {
                self.secondary_oam[(self.x / 2) as usize - 1] = 0xFF;
            }
        } else if self.x <= 256 {
            // TODO: Sprite overflow flag
            //println!("secondary oam pointer: {}", self.secondary_oam_pointer);
            match self.sprite_evaluation_state {
                SpriteEvaluationState::Read0(n) => {
                    self.oam_buffer = self.memory.read_oam(n * 4);
                    self.sprite_evaluation_state = SpriteEvaluationState::Copy0(n);
                }
                SpriteEvaluationState::Copy0(n) => {
                    if self.secondary_oam_pointer >= 32 {
                        // Already found 8 sprites, ignore write.
                        self.sprite_evaluation_state = SpriteEvaluationState::Read0(0);
                    } else {
                        self.secondary_oam[self.secondary_oam_pointer] = self.oam_buffer;
                        self.sprite_evaluation_state =
                            if self.sprite_in_range(self.oam_buffer, eval_y) {
                                if n == 0 {
                                    self.next_line_has_sprite_zero = true;
                                }
                                self.secondary_oam_pointer += 1;
                                SpriteEvaluationState::Read1(n)
                            } else {
                                SpriteEvaluationState::Read0((n + 1) % 64)
                            };
                    }
                }
                SpriteEvaluationState::Read1(n) => {
                    self.oam_buffer = self.memory.read_oam(n * 4 + 1);
                    self.sprite_evaluation_state = SpriteEvaluationState::Copy1(n);
                }
                SpriteEvaluationState::Copy1(n) => {
                    self.secondary_oam[self.secondary_oam_pointer] = self.oam_buffer;
                    self.secondary_oam_pointer += 1;
                    self.sprite_evaluation_state = SpriteEvaluationState::Read2(n);
                }
                SpriteEvaluationState::Read2(n) => {
                    self.oam_buffer = self.memory.read_oam(n * 4 + 2);
                    self.sprite_evaluation_state = SpriteEvaluationState::Copy2(n);
                }
                SpriteEvaluationState::Copy2(n) => {
                    self.secondary_oam[self.secondary_oam_pointer] = self.oam_buffer;
                    self.secondary_oam_pointer += 1;
                    self.sprite_evaluation_state = SpriteEvaluationState::Read3(n);
                }
                SpriteEvaluationState::Read3(n) => {
                    self.oam_buffer = self.memory.read_oam(n * 4 + 3);
                    self.sprite_evaluation_state = SpriteEvaluationState::Copy3(n);
                }
                SpriteEvaluationState::Copy3(n) => {
                    self.secondary_oam[self.secondary_oam_pointer] = self.oam_buffer;
                    self.secondary_oam_pointer += 1;
                    self.sprite_evaluation_state = SpriteEvaluationState::Read0((n + 1) % 64);
                }
            }
        } else if self.x <= 320 {
            // Sprite fetches
            let sprite_id = ((self.x - 257) & 0b111000) >> 3;
            let sprite_found = (sprite_id * 4 + 1) < self.secondary_oam_pointer;

            if sprite_found {
                match (self.x - 257) % 8 {
                    0 => {
                        self.sprite_units[sprite_id].y = self.secondary_oam[sprite_id * 4];
                    }
                    1 => {
                        self.sprite_units[sprite_id].tile_number =
                            self.secondary_oam[sprite_id * 4 + 1];
                    }
                    2 => {
                        self.sprite_units[sprite_id].attributes =
                            self.secondary_oam[sprite_id * 4 + 2];
                    }
                    3 => {
                        self.sprite_units[sprite_id].position =
                            self.secondary_oam[sprite_id * 4 + 3];
                    }
                    4 => {
                        self.sprite_pattern_address = self.sprite_units[sprite_id]
                            .get_low_data_address(
                                eval_y,
                                self.sprite_pattern_table,
                                self.sprite_height == 8,
                            );
                    }
                    5 => {
                        self.sprite_units[sprite_id]
                            .set_low_color(self.memory.read(self.sprite_pattern_address));
                    }
                    6 => {
                        self.sprite_pattern_address = self.sprite_units[sprite_id]
                            .get_high_data_address(
                                eval_y,
                                self.sprite_pattern_table,
                                self.sprite_height == 8,
                            );
                    }
                    7 => {
                        self.sprite_units[sprite_id]
                            .set_high_color(self.memory.read(self.sprite_pattern_address));
                    }
                    _ => unreachable!(),
                }
            } else {
                self.sprite_units[sprite_id].set_transparent();
            }
        }
    }

    fn get_sprite_color(&self) -> (u8, bool, bool) {
        for (i, sprite) in self.sprite_units.iter().enumerate() {
            let color = sprite.get_color();
            if (color & 3) != 0 {
                return (color, sprite.is_foreground(), i == 0);
            }
        }
        (0, false, false)
    }

    fn shift_sprites(&mut self) {
        self.sprite_units.iter_mut().for_each(|sprite| {
            sprite.shift_left();
        });
    }

    fn sprite_in_range(&mut self, sprite_y: u8, screen_y: usize) -> bool {
        (sprite_y as usize) <= screen_y
            && (sprite_y as usize + self.sprite_height as usize) > screen_y
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
        self.pattern_table_reg1 <<= 1;
        self.pattern_table_reg2 <<= 1;
        self.palette_attriubutes_reg1 <<= 1;
        self.palette_attriubutes_reg2 <<= 1;
    }

    fn merge_buffers_into_shift_registers(&mut self) {
        self.pattern_table_reg1 =
            (self.pattern_table_reg1 & 0xFF00) | (self.pattern_table_buffer1 as u16);
        self.pattern_table_reg2 =
            (self.pattern_table_reg2 & 0xFF00) | (self.pattern_table_buffer2 as u16);
        self.palette_attriubutes_reg1 =
            (self.palette_attriubutes_reg1 & 0xFF00) | (self.palette_attribute_buffer1 as u16);
        self.palette_attriubutes_reg2 =
            (self.palette_attriubutes_reg2 & 0xFF00) | (self.palette_attribute_buffer2 as u16);
    }

    fn fetch_background_data(&mut self) {
        match self.x % 8 {
            1 => {
                let tile_address = self.get_tile_address();
                self.current_tile = self.memory.read(tile_address);
                let bit2_y = ((self.reg_v & 0x40) >> 5) as u8; // Second bit of coarse Y
                let bit2_x = ((self.reg_v & 2) >> 1) as u8; // Second bit of coarse X
                self.current_attribute_quadrant = bit2_x | bit2_y;
                //self.current_attribute_quadrant = 3 - self.current_attribute_quadrant;
                self.current_attribute_address = self.get_attribute_address();
            }
            3 => {
                let attribute_data = self.memory.read(self.current_attribute_address);

                self.palette_attribute_buffer1 =
                    ((attribute_data >> (2 * self.current_attribute_quadrant)) & 1) * 255;
                self.palette_attribute_buffer2 =
                    ((attribute_data >> (2 * self.current_attribute_quadrant + 1)) & 1) * 255;
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
                let pattern_address = pattern_address | self.fine_y() | 8;
                self.pattern_table_buffer2 = self.memory.read(pattern_address);
            }
            _ => {}
        }
    }

    fn map_sprite_color(&mut self, color_idx: u8) -> (u8, u8, u8) {
        let system_color = self
            .memory
            .read(SPRITE_COLOR_PALETTE_ADDRESS | color_idx as u16);
        map_system_color_to_rgb(system_color)
    }

    fn map_background_color(&mut self, color_idx: u8) -> (u8, u8, u8) {
        let system_color = self
            .memory
            .read(IMAGE_COLOR_PALETTE_ADDRESS | color_idx as u16);
        map_system_color_to_rgb(system_color)
    }

    pub fn get_frame(&mut self) -> Buffer {
        let mut new_buffer = if let Some(buf) = self.buffers.pop() {
            buf
        } else {
            Buffer::empty(SCREEN_WIDTH, SCREEN_HEIGHT)
        };

        std::mem::swap(&mut self.current_frame, &mut new_buffer);
        new_buffer
    }

    pub fn transfer_io_registers(&mut self) {
        self.memory.set_ppu_io_registers(self.get_io_registers())
    }

    fn get_io_registers(&self) -> PpuIoRegisters {
        let status = if self.vblank_flag { 128 } else { 0 };
        let status = status | if self.sprite_zero_hit { 0x40 } else { 0 };

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

                let sprite_table = val >> 3;
                let sprite_table = sprite_table & 1 == 1;
                if sprite_table {
                    self.sprite_pattern_table = 0x1000;
                } else {
                    self.sprite_pattern_table = 0;
                }

                let sprite_size = val >> 5;
                let sprite_size = sprite_size & 1 == 1;
                if sprite_size {
                    self.sprite_height = 16;
                } else {
                    self.sprite_height = 8;
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
            PpuAction::OamDataWrite(val) => {
                // TODO:
                println!("OAM data write {:#04x} at addr: {:#04x}", val, self.oam_addr);
                if self.y >= 240 && self.y != 261 {
                    self.memory.write_oam(self.oam_addr as usize, val);
                }
                self.oam_addr = self.oam_addr.wrapping_add(1);
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
                    self.reg_t = (t_temp | cdefgh) & 0b00111111_11111111;

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
                println!("ppu address: {:#06x}", self.reg_v);
                // TODO:
                //todo!();
            }
            PpuAction::PpuDataWrite(val) => {
                self.memory.write(self.reg_v, val);

                self.reg_v = self.reg_v.wrapping_add(self.vram_address_increment);
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
        self.x_increment_counter += 1;
        if self.x_increment_counter == 8 {
            self.x_increment_counter = 0;
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

    pub(crate) fn render_pattern_table(&mut self, table_addr: u16, palette_idx: usize) -> Buffer {
        let mut buf = Buffer::empty(128, 128);
        for y in 0..16 {
            for x in 0..16 {
                let tile = self.render_tile(table_addr + y * 0x100 + x * 0x10, palette_idx);
                for y_fine in 0..8 {
                    for x_fine in 0..8 {
                        let pixel = tile[y_fine][x_fine];
                        let (r, g, b) = map_system_color_to_rgb(pixel);
                        buf.set_pixel(x as usize * 8 + x_fine, y as usize * 8 + y_fine, r, g, b);
                    }
                }
            }
        }
        buf
    }

    pub(crate) fn render_nametable(&mut self, idx: usize) -> Buffer {
        let nametable_addr = (idx * 0x400 + 0x2000) as u16;
        let patterntable_addr = self.background_pattern_table;
        let mut buf = Buffer::empty(256, 240);

        for y in 0..30 {
            for x in 0..32 {
                let tile_offset = y * 32 + x;
                let tile_addr_offset =
                    self.memory.read(nametable_addr + tile_offset) as u16 * 16;

                let attribute_addr_offset = ((y & 0b11100) << 1) | ((x & 0b11100) >> 2);
                let attribute_byte = self.memory.read(nametable_addr + 960 + attribute_addr_offset);
                let attribute_idx = y & 0b10 | ((x & 0b10) >> 1);
                
                let palette = (attribute_byte >> (2 * attribute_idx)) & 0b11;

                let tile = self.render_tile(tile_addr_offset + patterntable_addr, palette as usize);
                for y_fine in 0..8 {
                    for x_fine in 0..8 {
                        let pixel = tile[y_fine][x_fine];
                        let (r, g, b) = map_system_color_to_rgb(pixel);
                        buf.set_pixel(x as usize * 8 + x_fine, y as usize * 8 + y_fine, r, g, b);
                    }
                }
            }
        }

        buf
    }

    fn render_tile(&mut self, address: u16, palette_idx: usize) -> [[u8; 8]; 8] {
        let mut palette = [0u8; 4];

        let bg_color = self.memory.read(IMAGE_COLOR_PALETTE_ADDRESS);
        for i in 0..4 {
            palette[i] = self
                .memory
                .read(IMAGE_COLOR_PALETTE_ADDRESS + palette_idx as u16 * 4 + i as u16);
        }

        let mut out = [[0; 8]; 8];
        for y in 0..8 {
            let low = self.memory.read(address + y);
            let high = self.memory.read(address + y + 8);

            for x in 0..8 {
                let pixel = ((low >> (7 - x)) & 1) | (((high >> (7 - x)) & 1) << 1);
                let pixel = if pixel == 0 {
                    bg_color
                } else {
                    palette[pixel as usize]
                };
                out[y as usize][x as usize] = pixel
            }
        }
        out
    }
}

fn map_system_color_to_rgb(color: u8) -> (u8, u8, u8) {
    match color & 0x3F {
        0x00 => (0x75, 0x75, 0x75),
        0x01 => (0x27, 0x1B, 0x8F),
        0x02 => (0x00, 0x00, 0xAB),
        0x03 => (0x47, 0x00, 0x9F),
        0x04 => (0x8F, 0x00, 0x77),
        0x05 => (0xAB, 0x00, 0x13),
        0x06 => (0xA7, 0x00, 0x00),
        0x07 => (0x7F, 0x0B, 0x00),
        0x08 => (0x43, 0x2F, 0x00),
        0x09 => (0x00, 0x47, 0x00),
        0x0A => (0x00, 0x51, 0x00),
        0x0B => (0x00, 0x3F, 0x17),
        0x0C => (0x1B, 0x3F, 0x5F),
        0x0D => (0x00, 0x00, 0x00),
        0x0E => (0x00, 0x00, 0x00),
        0x0F => (0x00, 0x00, 0x00),
        0x10 => (0xBC, 0xBC, 0xBC),
        0x11 => (0x00, 0x73, 0xEF),
        0x12 => (0x23, 0x3B, 0xEF),
        0x13 => (0x83, 0x00, 0xF3),
        0x14 => (0xBF, 0x00, 0xBF),
        0x15 => (0xE7, 0x00, 0x5B),
        0x16 => (0xDB, 0x2B, 0x00),
        0x17 => (0xCB, 0x4F, 0x0F),
        0x18 => (0x8B, 0x73, 0x00),
        0x19 => (0x00, 0x97, 0x00),
        0x1A => (0x00, 0xAB, 0x00),
        0x1B => (0x00, 0x93, 0x3B),
        0x1C => (0x00, 0x83, 0x8B),
        0x1D => (0x00, 0x00, 0x00),
        0x1E => (0x00, 0x00, 0x00),
        0x1F => (0x00, 0x00, 0x00),
        0x20 => (0xFF, 0xFF, 0xFF),
        0x21 => (0x3F, 0xBF, 0xFF),
        0x22 => (0x5F, 0x97, 0xFF),
        0x23 => (0xA7, 0x8B, 0xFD),
        0x24 => (0xF7, 0x7B, 0xFF),
        0x25 => (0xFF, 0x77, 0xB7),
        0x26 => (0xFF, 0x77, 0x63),
        0x27 => (0xFF, 0x9B, 0x3B),
        0x28 => (0xF3, 0xBF, 0x3F),
        0x29 => (0x83, 0xD3, 0x13),
        0x2A => (0x4F, 0xDF, 0x4B),
        0x2B => (0x58, 0xF8, 0x98),
        0x2C => (0x00, 0xEB, 0xDB),
        0x2D => (0x00, 0x00, 0x00),
        0x2E => (0x00, 0x00, 0x00),
        0x2F => (0x00, 0x00, 0x00),
        0x30 => (0xFF, 0xFF, 0xFF),
        0x31 => (0xAB, 0xE7, 0xFF),
        0x32 => (0xC7, 0xD7, 0xFF),
        0x33 => (0xD7, 0xCB, 0xFF),
        0x34 => (0xFF, 0xC7, 0xFF),
        0x35 => (0xFF, 0xC7, 0xDB),
        0x36 => (0xFF, 0xBF, 0xB3),
        0x37 => (0xFF, 0xDB, 0xAB),
        0x38 => (0xFF, 0xE7, 0xA3),
        0x39 => (0xE3, 0xFF, 0xA3),
        0x3A => (0xAB, 0xF3, 0xBF),
        0x3B => (0xB3, 0xFF, 0xCF),
        0x3C => (0x9F, 0xFF, 0xF3),
        0x3D => (0x00, 0x00, 0x00),
        0x3E => (0x00, 0x00, 0x00),
        0x3F => (0x00, 0x00, 0x00),
        //_ => (0xFF, 0x00, 0x00),
        _ => unreachable!("Unimplemented color {:#04x}", color),
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
