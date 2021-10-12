use crate::utils::merge_u16;

use super::Cpu;
use AddressingResult::{Address, Implied, NotDone, Relative, Value, ValueAddress};

#[derive(Clone, Copy, Debug)]
pub(crate) enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    AbsoluteJMP {
        low_addr: u8,
    },
    AbsoluteAddr {
        low_addr: u8,
        high_addr: u8,
    },
    AbsoluteVal {
        low_addr: u8,
        high_addr: u8,
    },
    AbsoluteValAddr {
        low_addr: u8,
        high_addr: u8,
        value: u8,
    },
    ZeroPageAddr {
        address: u8,
    },
    ZeroPageVal {
        address: u8,
    },
    ZeroPageValAddr {
        address: u8,
        value: u8,
    },
    ZeroPageXAddr {
        address: u8,
        effective_address: u8,
    },
    ZeroPageXVal {
        address: u8,
        effective_address: u8,
    },
    ZeroPageXValAddr {
        address: u8,
        effective_address: u8,
        value: u8,
    },
    ZeroPageYAddr {
        address: u8,
        effective_address: u8,
    },
    ZeroPageYVal {
        address: u8,
        effective_address: u8,
    },
    Relative,
    AbsoluteXAddr {
        low_addr: u8,
        high_addr: u8,
        page_crossed: bool,
    },
    AbsoluteXVal {
        low_addr: u8,
        high_addr: u8,
    },
    AbsoluteXValAddr {
        low_addr: u8,
        high_addr: u8,
        value: u8,
    },
    AbsoluteYAddr {
        low_addr: u8,
        high_addr: u8,
        page_crossed: bool,
    },
    AbsoluteYVal {
        low_addr: u8,
        high_addr: u8,
    },
    AbsoluteYValAddr {
        low_addr: u8,
        high_addr: u8,
        value: u8,
    },
    Indirect {
        low_addr: u8,
        high_addr: u8,
        latch: u8,
    },
    IndexedIndirectAddr {
        pointer: u8,
        low_addr: u8,
        high_addr: u8,
    },
    IndexedIndirectVal {
        pointer: u8,
        low_addr: u8,
        high_addr: u8,
    },
    IndexedIndirectValAddr {
        pointer: u8,
        low_addr: u8,
        high_addr: u8,
        value: u8,
    },
    IndirectIndexedAddr {
        pointer: u8,
        low_addr: u8,
        high_addr: u8,
        page_crossed: bool,
    },
    IndirectIndexedVal {
        pointer: u8,
        low_addr: u8,
        high_addr: u8,
    },
    IndirectIndexedValAddr {
        pointer: u8,
        low_addr: u8,
        high_addr: u8,
        page_crossed: bool,
        value: u8,
    },
}

impl AddressingMode {
    pub(crate) fn absolute_jmp() -> Self {
        Self::AbsoluteJMP { low_addr: 0 }
    }
    pub(crate) fn absolute_addr() -> Self {
        Self::AbsoluteAddr {
            low_addr: 0,
            high_addr: 0,
        }
    }
    pub(crate) fn absolute_val() -> Self {
        Self::AbsoluteVal {
            low_addr: 0,
            high_addr: 0,
        }
    }
    pub(crate) fn absolute_val_addr() -> Self {
        Self::AbsoluteValAddr {
            low_addr: 0,
            high_addr: 0,
            value: 0,
        }
    }
    pub(crate) fn absolute_x_addr() -> Self {
        Self::AbsoluteXAddr {
            low_addr: 0,
            high_addr: 0,
            page_crossed: false,
        }
    }
    pub(crate) fn absolute_x_val() -> Self {
        Self::AbsoluteXVal {
            low_addr: 0,
            high_addr: 0,
        }
    }
    pub(crate) fn absolute_x_val_addr() -> Self {
        Self::AbsoluteXValAddr {
            low_addr: 0,
            high_addr: 0,
            value: 0,
        }
    }
    pub(crate) fn absolute_y_addr() -> Self {
        Self::AbsoluteYAddr {
            low_addr: 0,
            high_addr: 0,
            page_crossed: false,
        }
    }
    pub(crate) fn absolute_y_val() -> Self {
        Self::AbsoluteYVal {
            low_addr: 0,
            high_addr: 0,
        }
    }
    pub(crate) fn absolute_y_val_addr() -> Self {
        Self::AbsoluteYValAddr {
            low_addr: 0,
            high_addr: 0,
            value: 0,
        }
    }
    pub(crate) fn indirect() -> Self {
        Self::Indirect {
            low_addr: 0,
            high_addr: 0,
            latch: 0,
        }
    }
    pub(crate) fn zero_page_addr() -> Self {
        AddressingMode::ZeroPageAddr { address: 0 }
    }
    pub(crate) fn zero_page_val() -> Self {
        AddressingMode::ZeroPageVal { address: 0 }
    }
    pub(crate) fn zero_page_val_addr() -> Self {
        AddressingMode::ZeroPageValAddr {
            address: 0,
            value: 0,
        }
    }
    pub(crate) fn zero_page_x_addr() -> Self {
        AddressingMode::ZeroPageXAddr {
            address: 0,
            effective_address: 0,
        }
    }
    pub(crate) fn zero_page_x_val() -> Self {
        AddressingMode::ZeroPageXVal {
            address: 0,
            effective_address: 0,
        }
    }
    pub(crate) fn zero_page_x_val_addr() -> Self {
        AddressingMode::ZeroPageXValAddr {
            address: 0,
            effective_address: 0,
            value: 0,
        }
    }
    pub(crate) fn zero_page_y_addr() -> Self {
        AddressingMode::ZeroPageYAddr {
            address: 0,
            effective_address: 0,
        }
    }
    pub(crate) fn zero_page_y_val() -> Self {
        AddressingMode::ZeroPageYVal {
            address: 0,
            effective_address: 0,
        }
    }
    pub(crate) fn indexed_indirect_addr() -> Self {
        AddressingMode::IndexedIndirectAddr {
            pointer: 0,
            low_addr: 0,
            high_addr: 0,
        }
    }
    pub(crate) fn indexed_indirect_val() -> Self {
        AddressingMode::IndexedIndirectVal {
            pointer: 0,
            low_addr: 0,
            high_addr: 0,
        }
    }
    pub(crate) fn indexed_indirect_val_addr() -> Self {
        AddressingMode::IndexedIndirectValAddr {
            pointer: 0,
            low_addr: 0,
            high_addr: 0,
            value: 0,
        }
    }
    pub(crate) fn indirect_indexed_addr() -> Self {
        AddressingMode::IndirectIndexedAddr {
            pointer: 0,
            low_addr: 0,
            high_addr: 0,
            page_crossed: false,
        }
    }
    pub(crate) fn indirect_indexed_val() -> Self {
        AddressingMode::IndirectIndexedVal {
            pointer: 0,
            low_addr: 0,
            high_addr: 0,
        }
    }
    pub(crate) fn indirect_indexed_val_addr() -> Self {
        AddressingMode::IndirectIndexedValAddr {
            pointer: 0,
            low_addr: 0,
            high_addr: 0,
            page_crossed: false,
            value: 0,
        }
    }
}

impl AddressingMode {
    pub(crate) fn tick(cpu: &mut Cpu) -> AddressingResult {
        match &mut cpu.current_instr.addressing_mode {
            AddressingMode::Implied => Implied(cpu.memory.read_u8(cpu.program_counter)),
            AddressingMode::Accumulator => {
                cpu.memory.read_u8(cpu.program_counter);
                Value(cpu.accumulator)
            }
            AddressingMode::Immediate => {
                assert_eq!(cpu.instr_cycle, 2);
                let m = cpu.memory.read_u8(cpu.program_counter);
                cpu.logger.add_data_to_last_instr(m);
                cpu.program_counter += 1;
                Value(m)
            }
            AddressingMode::AbsoluteJMP { low_addr } => match cpu.instr_cycle {
                2 => {
                    *low_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.logger.add_data_to_last_instr(*low_addr);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    let high_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.logger.add_data_to_last_instr(high_addr);
                    let address = (high_addr as u16) << 8 | (*low_addr as u16);
                    Address(address)
                }
                _ => unreachable!(),
            },
            AddressingMode::AbsoluteAddr {
                low_addr,
                high_addr,
            } => match cpu.instr_cycle {
                2 => {
                    *low_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.logger.add_data_to_last_instr(*low_addr);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    *high_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.logger.add_data_to_last_instr(*high_addr);
                    cpu.program_counter += 1;
                    NotDone
                }
                4 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    Address(address)
                }
                _ => unreachable!(),
            },
            AddressingMode::AbsoluteVal {
                low_addr,
                high_addr,
            } => match cpu.instr_cycle {
                2 => {
                    *low_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.logger.add_data_to_last_instr(*low_addr);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    *high_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.logger.add_data_to_last_instr(*high_addr);
                    cpu.program_counter += 1;
                    NotDone
                }
                4 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    Value(cpu.memory.read_u8(address))
                }
                _ => unreachable!(),
            },
            AddressingMode::AbsoluteValAddr {
                low_addr,
                high_addr,
                value,
            } => match cpu.instr_cycle {
                2 => {
                    *low_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.logger.add_data_to_last_instr(*low_addr);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    *high_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.logger.add_data_to_last_instr(*high_addr);
                    cpu.program_counter += 1;
                    NotDone
                }
                4 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    *value = cpu.memory.read_u8(address);
                    NotDone
                }
                5 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    cpu.memory.write_u8(address, *value);
                    NotDone
                }
                6 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    ValueAddress(*value, address)
                }
                _ => unreachable!(),
            },
            AddressingMode::ZeroPageAddr { address: addr } => match cpu.instr_cycle {
                2 => {
                    *addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => Address(*addr as u16),
                _ => unreachable!(),
            },
            AddressingMode::ZeroPageVal { address: addr } => match cpu.instr_cycle {
                2 => {
                    *addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => Value(cpu.memory.read_u8(*addr as u16)),
                _ => unreachable!(),
            },
            AddressingMode::ZeroPageValAddr {
                address: addr,
                value: val,
            } => match cpu.instr_cycle {
                2 => {
                    *addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    *val = cpu.memory.read_u8(*addr as u16);
                    NotDone
                }
                4 => {
                    cpu.memory.write_u8(*addr as u16, *val);
                    NotDone
                }
                5 => ValueAddress(*val, *addr as u16),
                _ => unreachable!(),
            },
            AddressingMode::ZeroPageXAddr {
                address,
                effective_address,
            } => match cpu.instr_cycle {
                2 => {
                    *address = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    let _ = cpu.memory.read_u8(*address as u16);
                    *effective_address = address.wrapping_add(cpu.x);
                    NotDone
                }
                4 => Address(*effective_address as u16),
                _ => unreachable!(),
            },
            AddressingMode::ZeroPageXVal {
                address,
                effective_address,
            } => match cpu.instr_cycle {
                2 => {
                    *address = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    let _ = cpu.memory.read_u8(*address as u16);
                    *effective_address = address.wrapping_add(cpu.x);
                    NotDone
                }
                4 => {
                    let value = cpu.memory.read_u8(*effective_address as u16);
                    Value(value)
                }
                _ => unreachable!(),
            },
            AddressingMode::ZeroPageXValAddr {
                address,
                effective_address,
                value,
            } => match cpu.instr_cycle {
                2 => {
                    *address = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    let _ = cpu.memory.read_u8(*address as u16);
                    *effective_address = address.wrapping_add(cpu.x);
                    NotDone
                }
                4 => {
                    *value = cpu.memory.read_u8(*effective_address as u16);
                    NotDone
                }
                5 => {
                    cpu.memory.write_u8(*effective_address as u16, *value);
                    NotDone
                }
                6 => ValueAddress(*value, *effective_address as u16),
                _ => unreachable!(),
            },
            AddressingMode::ZeroPageYAddr {
                address,
                effective_address,
            } => match cpu.instr_cycle {
                2 => {
                    *address = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    let _ = cpu.memory.read_u8(*address as u16);
                    *effective_address = address.wrapping_add(cpu.y);
                    NotDone
                }
                4 => Address(*effective_address as u16),
                _ => unreachable!(),
            },
            AddressingMode::ZeroPageYVal {
                address,
                effective_address,
            } => match cpu.instr_cycle {
                2 => {
                    *address = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    let _ = cpu.memory.read_u8(*address as u16);
                    *effective_address = address.wrapping_add(cpu.y);
                    NotDone
                }
                4 => {
                    let value = cpu.memory.read_u8(*effective_address as u16);
                    Value(value)
                }
                _ => unreachable!(),
            },
            AddressingMode::Relative => {
                assert_eq!(cpu.instr_cycle, 2);
                let delta = cpu.memory.read_u8(cpu.program_counter) as i8;
                cpu.logger.add_data_to_last_instr(delta as u8);
                cpu.program_counter += 1;
                Relative(delta)
            }
            AddressingMode::AbsoluteXAddr {
                low_addr,
                high_addr,
                page_crossed,
            } => match cpu.instr_cycle {
                2 => {
                    *low_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    *high_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                4 => {
                    let (low_addr_fixed, overflow) = low_addr.overflowing_add(cpu.x);
                    *page_crossed = overflow;
                    let address = (*high_addr as u16) << 8 | (low_addr_fixed as u16);

                    cpu.memory.read_u8(address);
                    *low_addr = low_addr_fixed;
                    NotDone
                }
                5 => {
                    let address = if *page_crossed {
                        merge_u16(*low_addr, *high_addr + 1)
                        //(*high_addr as u16 + 1) << 8 | (*low_addr as u16)
                    } else {
                        merge_u16(*low_addr, *high_addr)
                        //(*high_addr as u16 + 1) << 8 | (*low_addr as u16)
                    };
                    Address(address)
                }
                _ => unreachable!(),
            },
            AddressingMode::AbsoluteXVal {
                low_addr,
                high_addr,
            } => match cpu.instr_cycle {
                2 => {
                    *low_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    *high_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                4 => {
                    let (low_addr_fixed, page_crossed) = low_addr.overflowing_add(cpu.x);
                    let address = (*high_addr as u16) << 8 | (low_addr_fixed as u16);
                    if page_crossed {
                        cpu.memory.read_u8(address);
                        *low_addr = low_addr_fixed;
                        NotDone
                    } else {
                        Value(cpu.memory.read_u8(address))
                    }
                }
                5 => {
                    // Page boundary was crossed, read from fixed address
                    let address = (*high_addr as u16 + 1) << 8 | (*low_addr as u16);
                    Value(cpu.memory.read_u8(address))
                }
                _ => unreachable!(),
            },
            AddressingMode::AbsoluteXValAddr {
                low_addr,
                high_addr,
                value,
            } => match cpu.instr_cycle {
                2 => {
                    *low_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    *high_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                4 => {
                    let low_addr_fixed = low_addr.wrapping_add(cpu.x);
                    let address = (*high_addr as u16) << 8 | (low_addr_fixed as u16);

                    let _ = cpu.memory.read_u8(address);
                    NotDone
                }
                5 => {
                    let (low_addr_fixed, page_crossed) = low_addr.overflowing_add(cpu.x);
                    *low_addr = low_addr_fixed;
                    if page_crossed {
                        *high_addr = high_addr.wrapping_add(1);
                    }
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    *value = cpu.memory.read_u8(address);
                    NotDone
                }
                6 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    cpu.memory.write_u8(address as u16, *value);
                    NotDone
                }
                7 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    ValueAddress(*value, address)
                }
                _ => unreachable!(),
            },
            AddressingMode::AbsoluteYAddr {
                low_addr,
                high_addr,
                page_crossed,
            } => match cpu.instr_cycle {
                2 => {
                    *low_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    *high_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                4 => {
                    let (low_addr_fixed, overflow) = low_addr.overflowing_add(cpu.y);
                    *page_crossed = overflow;
                    let address = (*high_addr as u16) << 8 | (low_addr_fixed as u16);

                    cpu.memory.read_u8(address);
                    *low_addr = low_addr_fixed;
                    NotDone
                }
                5 => {
                    let address = if *page_crossed {
                        merge_u16(*low_addr, *high_addr + 1)
                    } else {
                        merge_u16(*low_addr, *high_addr)
                    };
                    Address(address)
                }
                _ => unreachable!(),
            },
            AddressingMode::AbsoluteYVal {
                low_addr,
                high_addr,
            } => match cpu.instr_cycle {
                2 => {
                    *low_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    *high_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                4 => {
                    let (low_addr_fixed, page_crossed) = low_addr.overflowing_add(cpu.y);
                    let address = (*high_addr as u16) << 8 | (low_addr_fixed as u16);
                    if page_crossed {
                        cpu.memory.read_u8(address);
                        *low_addr = low_addr_fixed;
                        NotDone
                    } else {
                        Value(cpu.memory.read_u8(address))
                    }
                }
                5 => {
                    // Page boundary was crossed, read from fixed address
                    let address = (*high_addr as u16 + 1) << 8 | (*low_addr as u16);
                    Value(cpu.memory.read_u8(address))
                }
                _ => unreachable!(),
            },
            AddressingMode::AbsoluteYValAddr {
                low_addr,
                high_addr,
                value,
            } => match cpu.instr_cycle {
                2 => {
                    *low_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    *high_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                4 => {
                    let low_addr_fixed = low_addr.wrapping_add(cpu.y);
                    let address = (*high_addr as u16) << 8 | (low_addr_fixed as u16);

                    let _ = cpu.memory.read_u8(address);
                    NotDone
                }
                5 => {
                    let (low_addr_fixed, page_crossed) = low_addr.overflowing_add(cpu.y);
                    *low_addr = low_addr_fixed;
                    if page_crossed {
                        *high_addr = high_addr.wrapping_add(1);
                    }
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    *value = cpu.memory.read_u8(address);
                    NotDone
                }
                6 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    cpu.memory.write_u8(address as u16, *value);
                    NotDone
                }
                7 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    ValueAddress(*value, address)
                }
                _ => unreachable!(),
            },
            AddressingMode::Indirect {
                low_addr,
                high_addr,
                latch,
            } => match cpu.instr_cycle {
                2 => {
                    *low_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    *high_addr = cpu.memory.read_u8(cpu.program_counter);
                    cpu.program_counter += 1;
                    NotDone
                }
                4 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    *latch = cpu.memory.read_u8(address);
                    NotDone
                }
                5 => {
                    let address = (*high_addr as u16) << 8 | ((*low_addr).wrapping_add(1) as u16);
                    let target_high = cpu.memory.read_u8(address);
                    let target = (target_high as u16) << 8 | (*latch as u16);
                    Address(target)
                }
                _ => unreachable!(),
            },
            AddressingMode::IndexedIndirectAddr {
                low_addr,
                high_addr,
                pointer,
            } => match cpu.instr_cycle {
                2 => {
                    *pointer = cpu.memory.read_u8(cpu.program_counter);
                    cpu.logger.add_data_to_last_instr(*pointer);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    let _ = cpu.memory.read_u8(*pointer as u16);
                    //*pointer += cpu.x;
                    *pointer = pointer.wrapping_add(cpu.x);
                    NotDone
                }
                4 => {
                    *low_addr = cpu.memory.read_u8(*pointer as u16);
                    NotDone
                }
                5 => {
                    *high_addr = cpu.memory.read_u8(pointer.wrapping_add(1) as u16);
                    NotDone
                }
                6 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    Address(address)
                }
                _ => unreachable!(),
            },
            AddressingMode::IndexedIndirectVal {
                low_addr,
                high_addr,
                pointer,
            } => match cpu.instr_cycle {
                2 => {
                    *pointer = cpu.memory.read_u8(cpu.program_counter);
                    cpu.logger.add_data_to_last_instr(*pointer);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    let _ = cpu.memory.read_u8(*pointer as u16);
                    *pointer = pointer.wrapping_add(cpu.x);
                    NotDone
                }
                4 => {
                    *low_addr = cpu.memory.read_u8(*pointer as u16);
                    NotDone
                }
                5 => {
                    *high_addr = cpu.memory.read_u8(pointer.wrapping_add(1) as u16);
                    NotDone
                }
                6 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    let target = cpu.memory.read_u8(address);
                    Value(target)
                }
                _ => unreachable!(),
            },
            AddressingMode::IndexedIndirectValAddr {
                pointer,
                low_addr,
                high_addr,
                value,
            } => match cpu.instr_cycle {
                2 => {
                    *pointer = cpu.memory.read_u8(cpu.program_counter);
                    cpu.logger.add_data_to_last_instr(*pointer);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    let _ = cpu.memory.read_u8(*pointer as u16);
                    *pointer = pointer.wrapping_add(cpu.x);
                    NotDone
                }
                4 => {
                    *low_addr = cpu.memory.read_u8(*pointer as u16);
                    NotDone
                }
                5 => {
                    *high_addr = cpu.memory.read_u8(pointer.wrapping_add(1) as u16);
                    NotDone
                }
                6 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    *value = cpu.memory.read_u8(address);
                    NotDone
                }
                7 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    cpu.memory.write_u8(address, *value);
                    NotDone
                }
                8 => {
                    let address = (*high_addr as u16) << 8 | (*low_addr as u16);
                    ValueAddress(*value, address)
                }
                _ => unreachable!(),
            },
            AddressingMode::IndirectIndexedAddr {
                pointer,
                low_addr,
                high_addr,
                page_crossed,
            } => match cpu.instr_cycle {
                2 => {
                    *pointer = cpu.memory.read_u8(cpu.program_counter);
                    cpu.logger.add_data_to_last_instr(*pointer);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    *low_addr = cpu.memory.read_u8(*pointer as u16);
                    NotDone
                }
                4 => {
                    *high_addr = cpu.memory.read_u8(pointer.wrapping_add(1) as u16);
                    NotDone
                }
                5 => {
                    let (low_addr_fixed, overflow) = low_addr.overflowing_add(cpu.y);
                    *page_crossed = overflow;
                    let address = (*high_addr as u16) << 8 | (low_addr_fixed as u16);

                    cpu.memory.read_u8(address);
                    *low_addr = low_addr_fixed;
                    NotDone
                }
                6 => {
                    let address = if *page_crossed {
                        merge_u16(*low_addr, *high_addr + 1)
                    } else {
                        merge_u16(*low_addr, *high_addr)
                    };
                    Address(address)
                }
                _ => unreachable!(),
            },
            AddressingMode::IndirectIndexedVal {
                pointer,
                low_addr,
                high_addr,
            } => match cpu.instr_cycle {
                2 => {
                    *pointer = cpu.memory.read_u8(cpu.program_counter);
                    cpu.logger.add_data_to_last_instr(*pointer);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    *low_addr = cpu.memory.read_u8(*pointer as u16);
                    NotDone
                }
                4 => {
                    *high_addr = cpu.memory.read_u8(pointer.wrapping_add(1) as u16);
                    NotDone
                }
                5 => {
                    let (low_addr_fixed, page_crossed) = low_addr.overflowing_add(cpu.y);
                    let address = (*high_addr as u16) << 8 | (low_addr_fixed as u16);
                    if page_crossed {
                        cpu.memory.read_u8(address);
                        *low_addr = low_addr_fixed;
                        NotDone
                    } else {
                        let value = cpu.memory.read_u8(address);
                        cpu.logger.set_last_target_address(address, value);
                        Value(value)
                    }
                }
                6 => {
                    // Page boundary was crossed, read from fixed address
                    let address = (*high_addr as u16 + 1) << 8 | (*low_addr as u16);
                    let value = cpu.memory.read_u8(address);
                    cpu.logger.set_last_target_address(address, value);
                    Value(value)
                }
                _ => unreachable!(),
            },
            AddressingMode::IndirectIndexedValAddr {
                pointer,
                low_addr,
                high_addr,
                page_crossed,
                value,
            } => match cpu.instr_cycle {
                2 => {
                    *pointer = cpu.memory.read_u8(cpu.program_counter);
                    cpu.logger.add_data_to_last_instr(*pointer);
                    cpu.program_counter += 1;
                    NotDone
                }
                3 => {
                    *low_addr = cpu.memory.read_u8(*pointer as u16);
                    NotDone
                }
                4 => {
                    *high_addr = cpu.memory.read_u8(pointer.wrapping_add(1) as u16);
                    NotDone
                }
                5 => {
                    let (low_addr_fixed, overflow) = low_addr.overflowing_add(cpu.y);
                    let address = (*high_addr as u16) << 8 | (low_addr_fixed as u16);
                    let _ = cpu.memory.read_u8(address);
                    *page_crossed = overflow;
                    *low_addr = low_addr_fixed;
                    NotDone
                }
                6 => {
                    if *page_crossed {
                        *high_addr = high_addr.wrapping_add(1);
                    };
                    let address = merge_u16(*low_addr, *high_addr);
                    *value = cpu.memory.read_u8(address);
                    NotDone
                }
                7 => {
                    let address = merge_u16(*low_addr, *high_addr);
                    let _ = cpu.memory.read_u8(address);
                    NotDone
                }
                8 => {
                    let address = merge_u16(*low_addr, *high_addr);
                    ValueAddress(*value, address)
                }
                _ => unreachable!(),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum AddressingResult {
    NotDone,
    Done,
    Implied(u8),
    Value(u8),
    Address(u16),
    ValueAddress(u8, u16),
    Relative(i8),
}
