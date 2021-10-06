use crate::memory::ApuMemory;

const RATE_VALUE_LOOKUP_TABLE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

#[derive(Debug)]
pub(crate) struct Dmc {
    irq_enabled_flag: bool,
    loop_flag: bool,
    rate_value: u16,

    sample_address: u16,
    sample_length: u16,

    current_address: u16,
    pub sample_bytes_remaining: u16,

    sample_buffer: u8,
    sample_buffer_empty: bool,
    output_bits_remaining: u8,
    shift_register: u8,
    silence_flag: bool,

    cycle: u16,
    out_value: u8,

    pub interrupt_flag: bool,

    pub restart_flag: bool,
}

impl Dmc {
    pub(crate) fn new() -> Self {
        Self {
            irq_enabled_flag: false,
            loop_flag: false,
            rate_value: 0,

            sample_address: 0,
            sample_length: 0,

            current_address: 0,
            sample_bytes_remaining: 0,

            sample_buffer: 0,
            sample_buffer_empty: true,
            output_bits_remaining: 0,
            shift_register: 0,
            silence_flag: true,

            cycle: 0,
            out_value: 0,

            interrupt_flag: false,

            restart_flag: false,
        }
    }

    pub(crate) fn tick(&mut self, memory: &mut ApuMemory) {
        self.tick_memory_unit(memory);
        if self.cycle >= self.rate_value {
            self.cycle = 0;
            self.advance_out_value();
        } else {
            self.cycle += 1;
        }
    }

    fn tick_memory_unit(&mut self, memory: &mut ApuMemory) {
        if self.restart_flag {
            if self.output_bits_remaining == 8 {
                // restart
                self.current_address = self.sample_address;
                self.sample_bytes_remaining = self.sample_length;
                self.silence_flag = false;
                self.restart_flag = false;
            }
        }

        if self.sample_buffer_empty && self.sample_bytes_remaining != 0 {
            self.sample_buffer = memory.read_u8(self.current_address);
            let (new_addr, overflow) = self.current_address.overflowing_add(1);
            self.current_address = if overflow { 0x8000 } else { new_addr };
            self.sample_bytes_remaining -= 1;
            self.sample_buffer_empty = false;
        }
    }

    fn advance_out_value(&mut self) {
        if self.output_bits_remaining == 0 {
            // output cycle ended
            self.output_bits_remaining = 8;

            if self.sample_buffer_empty {
                self.silence_flag = true;
            } else {
                self.silence_flag = false;
                self.shift_register = self.sample_buffer;
                self.sample_buffer_empty = true;
            }
        }

        if !self.silence_flag {
            let increase = self.shift_register & 1 == 1;
            self.out_value = if increase {
                // increase by 2
                if self.out_value >= 126 {
                    self.out_value
                } else {
                    self.out_value + 2
                }
            } else {
                // decrease by 2
                if self.out_value <= 1 {
                    self.out_value
                } else {
                    self.out_value - 2
                }
            };

            self.shift_register = self.shift_register >> 1;
            self.output_bits_remaining -= 1;
        }
    }

    pub(crate) fn next_value(&mut self) -> u8 {
        self.out_value
    }

    pub(crate) fn write0(&mut self, value: u8) {
        self.irq_enabled_flag = (value & 0b10000000) != 0;
        self.loop_flag = (value & 0b01000000) != 0;
        let rate_index = value & 0b0000_1111;
        self.rate_value = RATE_VALUE_LOOKUP_TABLE[rate_index as usize];
        //eprintln!("0x4010 {:#10b}", value);
    }

    pub(crate) fn write1(&mut self, value: u8) {
        self.out_value = value & 0b0111_1111;
        //eprintln!("out value: {}", value);
    }

    pub(crate) fn write2(&mut self, value: u8) {
        // Sample address = %11AAAAAA.AA000000 = $C000 + (A * 64)
        let a = value as u16;
        self.sample_address = 0xC000 | (a << 6);
        //eprintln!("sample address {}", self.sample_address);
    }

    pub(crate) fn write3(&mut self, value: u8) {
        // Sample length = %LLLL.LLLL0001 = (L * 16) + 1 bytes
        let l = value as u16;
        self.sample_length = 1 | (l << 4);
        //eprintln!("sample length {}", self.sample_length);
    }
}
