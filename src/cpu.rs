use crate::{bit_utils::{address_from_bytes, bit_set, get_bits, nth_bit, set_bit}, memory_bus::MemoryBus};


#[derive(Copy, Clone, Debug)]
#[repr(usize)]
pub enum CPUFlags {
    Carry               = 0,
    Zero                = 1,
    InterruptDisable    = 2,
    Decimal             = 3,
    //                  B
    //                  1
    Overflow            = 6,
    Negative            = 7,
}

#[derive(Copy, Clone, Debug)]
pub enum AddressingMode {
    Implied,

    Accumulator,

    Immediate,

    ZeroPage,
    
    ZeroPageX,

    ZeroPageY,

    Relative,

    Absolute,

    AbsoluteX,

    AbsoluteY,

    Indirect,

    IndexedIndirect,

    IndirectIndexed
}

#[derive(Clone, Debug)]
pub struct CPU {
    pub pc:     u16,

    pub sp:     u8,
    pub p:      u8,

    pub a:      u8,

    pub x:      u8,
    pub y:      u8,
    
    pub cycles: usize,
    pub delay:  usize
}

impl CPU {
    pub fn new() -> CPU {
        CPU
        {
            pc:     0,
            sp:     0xFD,
            p:      0b00110000,
            a:      0,
            x:      0,
            y:      0,
            cycles: 0,
            delay:  0
        }
    }

    pub fn set_flag(&mut self, flag: CPUFlags, to: bool) {
        self.p = set_bit(self.p, flag as usize, to)
    }

    pub fn flag_set(&self, flag: CPUFlags) -> bool {
        bit_set(self.p, flag as usize)
    }


    pub fn read8(&self, bus: &mut MemoryBus, addr: u16) -> u8 {
        bus.read(addr)
    }

    pub fn read16(&self, bus: &mut MemoryBus, addr: u16) -> u16 {
        address_from_bytes(bus.read(addr), bus.read(addr.wrapping_add(1)))
    }


    pub fn write8(&mut self, bus: &mut MemoryBus, addr: u16, to: u8) {
        bus.write(addr, to);
    }


    pub fn push8(&mut self, bus: &mut MemoryBus, b: u8) {
        bus.write(0x100 + self.sp as u16, b);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn pop8(&mut self, bus: &mut MemoryBus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        bus.read(0x100 + self.sp as u16)
    }


    pub fn push16(&mut self, bus: &mut MemoryBus, value: u16) {
        self.push8(bus, (value >> 8) as u8);
        self.push8(bus, value as u8);
    }

    pub fn pop16(&mut self, bus: &mut MemoryBus) -> u16 {
        let lo = self.pop8(bus) as u16;
        let hi = self.pop8(bus) as u16;

        (hi << 8) | lo
    }

    pub(crate) fn jump_to_handler(&mut self, bus: &mut MemoryBus, ret_target: u16, lsb_addr: u16) -> usize {
        self.push16(bus, ret_target);

        self.push8(bus, self.p | nth_bit::<u8>(4) | nth_bit::<u8>(5));

        self.set_flag(CPUFlags::InterruptDisable, true);
        self.pc = self.read16(bus, lsb_addr);

        7
    }

    pub fn jump_to_nmi_handler(&mut self, bus: &mut MemoryBus) {
        self.delay += self.jump_to_handler(bus, self.pc, 0xFFFA);
    }

    pub fn jump_to_startup(&mut self, bus: &mut MemoryBus) {
        self.pc = self.read16(bus, 0xFFFC);
        self.delay += 7;
    }


    pub(crate) fn get_address(&self, bus: &mut MemoryBus, mode: AddressingMode) -> u16 {  
        match mode {
            AddressingMode::ZeroPage => {
                self.read8(bus, self.pc.wrapping_add(1)) as u16
            }

            AddressingMode::ZeroPageX => {
                self.x
                    .wrapping_add(self.read8(bus, self.pc.wrapping_add(1)))
                    as u16
            }

            AddressingMode::ZeroPageY => {
                self.y
                    .wrapping_add(self.read8(bus, self.pc.wrapping_add(1)))
                    as u16
            }

            AddressingMode::Absolute => {
                self.read16(bus, self.pc.wrapping_add(1)) as u16
            }

            AddressingMode::AbsoluteX => {
                self.read16(bus, self.pc.wrapping_add(1)).wrapping_add(self.x as u16) as u16
            }

            AddressingMode::AbsoluteY => {
                self.read16(bus, self.pc.wrapping_add(1)).wrapping_add(self.y as u16) as u16
            }

            AddressingMode::Relative  => {
                let offset = self.read8(bus, self.pc.wrapping_add(1)) as i8;

                self.pc
                    .wrapping_add(2)
                    .wrapping_add_signed(offset as i16)
            }

            AddressingMode::Indirect => {
                let ptr = self.read16(bus, self.pc.wrapping_add(1));

                let lsb = self.read8(bus, ptr);

                let msb_addr =
                    (ptr & 0xFF00) |
                    ((ptr.wrapping_add(1)) & 0x00FF);

                let msb = self.read8(bus, msb_addr);

                address_from_bytes(lsb, msb)
            }

            AddressingMode::IndexedIndirect => {
                let ptr = self.x
                    .wrapping_add(self.read8(bus, self.pc.wrapping_add(1)));

                // Must read byte by byte to preserve page wrapping behaviors
                let lsb = self.read8(bus, ptr as u16);
                let msb = self.read8(bus, ptr.wrapping_add(1) as u16);

                address_from_bytes(lsb, msb)
            }

            AddressingMode::IndirectIndexed => {
                let ptr = self.read8(bus, self.pc.wrapping_add(1));

                let lsb = self.read8(bus, ptr as u16);
                let msb = self.read8(bus, ptr.wrapping_add(1) as u16);

                address_from_bytes(lsb, msb).wrapping_add(self.y as u16)
            }

            _ => unreachable!("mode has no address"),
        }
    }

    pub(crate) fn page_crossed(&self, bus: &mut MemoryBus, mode: AddressingMode) -> bool {
        match mode {
            AddressingMode::AbsoluteX       |
            AddressingMode::AbsoluteY       => {
                let address_before_add = self.read16(bus, self.pc.wrapping_add(1));

                let page_crossed =
                    get_bits(address_before_add, 0..8) + 
                    (if let AddressingMode::AbsoluteX = mode { self.x } else { self.y }) as u16 > 0xFF;

                page_crossed
            }

            AddressingMode::IndirectIndexed => {
                let ptr = self.read8(bus, self.pc.wrapping_add(1) as u16);

                let lsb = self.read8(bus, ptr as u16);
                let msb = self.read8(bus, ptr.wrapping_add(1) as u16);

                let address_before_add = address_from_bytes(lsb, msb);

                let page_crossed =
                    get_bits(address_before_add, 0..8) + self.y as u16 > 0xFF;

                page_crossed
            },

            AddressingMode::Relative => {
                let offset = self.read8(bus, self.pc.wrapping_add(1) as u16) as i8;

                let base = self.pc.wrapping_add(2);
                let target = base.wrapping_add_signed(offset as i16);

                let page_crossed = 
                    get_bits(base,   8..16) !=
                    get_bits(target, 8..16);

                page_crossed
            },

            _ => false
        }
    }


    pub(crate) fn get8(&self, bus: &mut MemoryBus, mode: AddressingMode) -> u8 {
        match mode {
            AddressingMode::Accumulator => self.a,

            AddressingMode::Immediate |
            AddressingMode::Relative => {
                self.read8(bus, self.pc.wrapping_add(1))
            }

            _ => {
                let addr = self.get_address(bus, mode);
                self.read8(bus, addr as u16)
            }
        }
    }

    pub(crate) fn set8(&mut self, bus: &mut MemoryBus, mode: AddressingMode, value: u8) {
        match mode {
            AddressingMode::Accumulator => {
                self.a = value;
            }

            AddressingMode::Immediate |
            AddressingMode::Relative => {
                panic!("cannot write to immediate/relative operand");
            }

            _ => {
                let addr = self.get_address(bus, mode) as u16;
                self.write8(bus, addr, value);
            }
        }
    }

    pub fn debug(&self) {
        println!("PC: {:X}\n    NV  DIZC\nP:  {:08b}\nSP: {:02X}\nA:  {:02X}\nX:  {:02X}\nY:  {:02X}", self.pc, self.p, self.sp, self.a, self.x, self.y);
    }
}