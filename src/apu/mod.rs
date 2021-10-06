mod dmc;
use dmc::Dmc;
mod noise;
use noise::Noise;
mod pulse;
use pulse::Pulse;
mod triangle;
use triangle::Triangle;
mod length_counter;
mod envelope;

use std::{borrow::BorrowMut, collections::VecDeque};


use sdl2::audio::{AudioCallback, AudioDeviceLockGuard, AudioSpec};

use crate::memory::{ApuMemory, BusAction};

const APU_FREQ: u32 = 1789773;

pub(crate) const LENGTH_COUNTER_TABLE: [u8; 0x20] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

lazy_static! {
    static ref PULSE_TABLE: [f32; 31] = {
        let mut table = [0.; 31];

        for n in 0u16..31 {
            let i: f32 = n.into();
            table[n as usize] = 95.52 / (8128.0 / i + 100.0)
        }

        table
    };
    static ref TND_TABLE: [f32; 203] = {
        let mut table = [0.; 203];

        for n in 0u16..203 {
            let i: f32 = n.into();
            table[n as usize] = 163.67 / (24329.0 / i + 100.)
        }

        table
    };
}

pub struct Apu {
    memory: ApuMemory,
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,

    //output: Vec<f32>,
    output: VecDeque<f32>,
    cycle: u32,

    out_cycle: f64,

    frame_counter_mode: u8,
    frame_counter_cycle: u8,
    irq_inhibited: bool,

    pulse1_silenced: bool,
    pulse2_silenced: bool,
    triangle_silenced: bool,
}

struct FrameCounterAction {
    irc: bool,
    sweep: bool,
    envelope: bool,
}

impl Apu {
    pub fn new(memory: ApuMemory) -> Self {
        Self {
            memory,
            pulse1: Pulse::new(),
            pulse2: Pulse::new(),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: Dmc::new(),
            
            //output: Vec::new(),
            output: VecDeque::new(),
            cycle: 0,

            out_cycle: 0.,

            frame_counter_mode: 0,
            frame_counter_cycle: 0,
            irq_inhibited: false,

            pulse1_silenced: false,
            pulse2_silenced: false,
            triangle_silenced: false,
        }
    }

    pub fn tick(&mut self, bus_action: BusAction) {
        let frame_counter = self.frame_counter_action();
        if self.cycle % 2 == 0 {
            self.pulse1
                .tick(frame_counter.sweep, frame_counter.envelope, true);
            self.pulse2
                .tick(frame_counter.sweep, frame_counter.envelope, false);
        }

        self.triangle.tick(frame_counter.sweep, frame_counter.envelope);
        self.dmc.tick(&mut self.memory);

        if let BusAction::ApuWrite((address, value)) = bus_action {
            match address {
                0x4000 => self.pulse1.write0(value),
                0x4001 => self.pulse1.write1(value),
                0x4002 => self.pulse1.write2(value),
                0x4003 => self.pulse1.write3(value),
                0x4004 => self.pulse2.write0(value),
                0x4005 => self.pulse2.write1(value),
                0x4006 => self.pulse2.write2(value),
                0x4007 => self.pulse2.write3(value),
                0x4008 => self.triangle.write0(value),
                0x4009 => self.triangle.write1(value),
                0x400A => self.triangle.write2(value),
                0x400B => self.triangle.write3(value),
                0x400C => self.noise.write0(value),
                0x400E => self.noise.write2(value),
                0x400F => self.noise.write3(value),
                0x4010 => self.dmc.write0(value),
                0x4011 => self.dmc.write1(value),
                0x4012 => self.dmc.write2(value),
                0x4013 => self.dmc.write3(value),
                0x4015 => {
                    if value & 0b0000_0001 == 0 {
                        self.pulse1_silenced = true;
                        self.pulse1.length_counter = 0;
                    } else {
                        self.pulse1_silenced = false;
                    }

                    if value & 0b0000_0010 == 0 {
                        self.pulse2_silenced = true;
                        self.pulse2.length_counter = 0;
                    } else {
                        self.pulse2_silenced = false;
                    }

                    if value & 0b0000_0100 == 0 {
                        self.triangle_silenced = true;
                        self.triangle.length_counter = 0;
                    } else {
                        self.triangle_silenced = false;
                    }

                    if value & 0b0001_0000 == 0 {
                        self.dmc.sample_bytes_remaining = 0;
                    } else {
                        self.dmc.restart_flag = true;
                    }

                    self.dmc.interrupt_flag = false;
                }
                0x4017 => {
                    self.frame_counter_mode = (value & 0b1000_0000) >> 7;
                    self.irq_inhibited = value & 0b0100_0000 != 0;
                }
                _ => {}
            }
        }

        if self.is_output_cycle() {
            // write value to buffer
            let p1 = if self.pulse1_silenced {
                0
            } else {
                self.pulse1.next_value()
            };
            let p2 = if self.pulse2_silenced {
                0
            } else {
                self.pulse2.next_value()
            };
            let p = (p1 + p2) as usize;

            let t = if self.triangle_silenced {
                0
            } else {
                self.triangle.next_value()
            };

            let d = self.dmc.next_value();

            let tnd = (t + d) as usize;

            let out = PULSE_TABLE[p] + TND_TABLE[tnd];
            //let out = PULSE_TABLE[0] + TND_TABLE[t as usize];
            //let out = TND_TABLE[d as usize];

            self.output.push_back(out);
        }

        self.cycle = self.cycle.wrapping_add(1);
        //self.out_cycle += 1.;
    }

    fn is_output_cycle(&mut self) -> bool {
        //self.cycle % (APU_FREQ / 44100) == 0
        self.cycle % 40 == 0
    }

    fn frame_counter_action(&mut self) -> FrameCounterAction {
        if self.cycle % 7457 == 0 {
            if self.frame_counter_mode == 0 {
                let v = match self.frame_counter_cycle {
                    0 => FrameCounterAction {
                        irc: false,
                        sweep: false,
                        envelope: true,
                    },
                    1 => FrameCounterAction {
                        irc: false,
                        sweep: true,
                        envelope: true,
                    },
                    2 => FrameCounterAction {
                        irc: false,
                        sweep: false,
                        envelope: true,
                    },
                    3 => FrameCounterAction {
                        irc: !self.irq_inhibited,
                        sweep: true,
                        envelope: true,
                    },
                    _ => unreachable!(),
                };
                self.frame_counter_cycle = (self.frame_counter_cycle + 1) % 4;
                v
            } else {
                let v = match self.frame_counter_cycle {
                    0 => FrameCounterAction {
                        irc: false,
                        sweep: false,
                        envelope: true,
                    },
                    1 => FrameCounterAction {
                        irc: false,
                        sweep: true,
                        envelope: true,
                    },
                    2 => FrameCounterAction {
                        irc: false,
                        sweep: false,
                        envelope: true,
                    },
                    3 => FrameCounterAction {
                        irc: false,
                        sweep: false,
                        envelope: false,
                    },
                    4 => FrameCounterAction {
                        irc: false,
                        sweep: true,
                        envelope: true,
                    },
                    _ => unreachable!(),
                };
                self.frame_counter_cycle = (self.frame_counter_cycle + 1) % 5;
                v
            }
        } else {
            FrameCounterAction {
                irc: false,
                sweep: false,
                envelope: false,
            }
        }
    }

    pub fn update_audio_generator(&mut self, mut generator: AudioDeviceLockGuard<AudioGenerator>) {
        generator.values.append(&mut self.output);
    }
}

pub struct AudioGenerator {
    //values: Vec<f32>,
    values: VecDeque<f32>,
}
impl AudioGenerator {
    pub fn from_spec(_spec: AudioSpec) -> Self {
        Self {
            //values: Vec::new(),
            values: VecDeque::new(),
        }
    }
}

impl AudioCallback for AudioGenerator {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        //merge_samples(&self.values[..], out);
        //self.values.clear();
        //eprintln!("{}", self.values.len());

        let l = self.values.len();
        let skipping = l > 2800;
        let skip_every = if l > 4000 { 30 } else { 80 };

        if self.values.len() < out.len() {
            eprintln!("Too few samples");
        }

        if self.values.len() > 5000 {
            eprintln!("fuckup {} {}", self.values.len(), out.len());
        }
        //let mut fuckup = false;
        //while self.values.len() > 5000 {
            //if !fuckup {
                //eprintln!("fuckup {} {}", self.values.len(), out.len());
                //fuckup = true;
            //}
            //self.values.pop_front();
        //}
        for (i, x) in out.iter_mut().enumerate() {
            if skipping && i % skip_every == 0 {
                self.values.pop_front();
            }

            *x = self.values.pop_front().unwrap_or_default();
        }

        //if self.values.len() >= out.len() {
        //eprintln!("too many");
        //for (i, x) in out.iter_mut().enumerate().rev() {
        //if self.values.len() > i + 1 {
        //let v1 = self.values.pop_back().unwrap();
        //let v2 = self.values.pop_back().unwrap();
        //*x = (v1 + v2) / 2.;
        //} else {
        //*x = self.values.pop_back().unwrap();
        //}
        //}
        //} else {
        //eprintln!("missing");
        //for (i, x) in out.iter_mut().enumerate().rev() {
        //if self.values.len() < i + 1 {
        //*x = self.values.back().cloned().unwrap_or_default();
        //} else {
        //*x = self.values.pop_back().unwrap_or_default();
        //}
        //}
        //}
    }
}

//fn merge_samples(input: &[f32], out: &mut [f32]) {
    //let l_in = input.len();
    //let l_out = out.len();
//
    //let mut j = 0;
    //let mut k = 0;
    //for (i, x) in out.iter_mut().enumerate() {
        //*x = (input[j] + input[k]) / 2.;
    //}
//}
