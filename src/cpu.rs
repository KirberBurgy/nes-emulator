use crate::bit_utils::{address_from_bytes, bit_set, get_bits, set_bit};


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

pub struct CPU {
    pub pc:     u16,

    pub sp:     u8,
    pub p:      u8,

    pub a:      u8,

    pub x:      u8,
    pub y:      u8,

    pub ram:    Box<[u8; 0x10000]>,
    
    pub cycles: usize
}

impl CPU {
    pub fn new() -> CPU {
        CPU
        {
            pc:     0,
            sp:     0xFF,
            p:      0b00110000,
            a:      0,
            x:      0,
            y:      0,
            cycles: 0,
            ram: Box::new([0; 0x10000]),
        }
    }

    pub fn set_flag(&mut self, flag: CPUFlags, to: bool) {
        self.p = set_bit(self.p, flag as usize, to)
    }

    pub fn flag_set(&self, flag: CPUFlags) -> bool {
        bit_set(self.p, flag as usize)
    }


    pub fn read8(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    pub fn read16(&self, addr: u16) -> u16 {
        address_from_bytes(self.ram[addr as usize], self.ram[addr.wrapping_add(1) as usize])
    }


    pub fn write8(&mut self, addr: u16, to: u8) {
        self.ram[addr as usize] = to;
    }


    pub fn push8(&mut self, b: u8) {
        self.ram[0x100 + self.sp as usize] = b;
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn pop8(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.ram[0x100 + self.sp as usize]
    }


    pub fn push16(&mut self, value: u16) {
        self.push8((value >> 8) as u8);
        self.push8(value as u8);
    }

    pub fn pop16(&mut self) -> u16 {
        let lo = self.pop8() as u16;
        let hi = self.pop8() as u16;

        (hi << 8) | lo
    }

    
    pub fn jump_to_interrupt_handler(&mut self) {
        let lsb = self.ram[0xFFFE];
        let msb = self.ram[0xFFFF];

        self.pc = address_from_bytes(lsb, msb);
    }

    pub(crate) fn get_address(&self, mode: AddressingMode) -> usize {  
        match mode {
            AddressingMode::ZeroPage => {
                self.read8(self.pc.wrapping_add(1)) as usize
            }

            AddressingMode::ZeroPageX => {
                self.x
                    .wrapping_add(self.read8(self.pc.wrapping_add(1)))
                    as usize
            }

            AddressingMode::ZeroPageY => {
                self.y
                    .wrapping_add(self.read8(self.pc.wrapping_add(1)))
                    as usize
            }

            AddressingMode::Absolute => {
                self.read16(self.pc.wrapping_add(1)) as usize
            }

            AddressingMode::AbsoluteX => {
                self.read16(self.pc.wrapping_add(1)).wrapping_add(self.x as u16) as usize
            }

            AddressingMode::AbsoluteY => {
                self.read16(self.pc.wrapping_add(1)).wrapping_add(self.y as u16) as usize
            }

            AddressingMode::Relative  => {
                let offset = self.read8(self.pc.wrapping_add(1)) as i8;

                self.pc
                    .wrapping_add(2)
                    .wrapping_add_signed(offset as i16) as usize
            }

            AddressingMode::Indirect => {
                let ptr = self.read16(self.pc.wrapping_add(1));

                let lsb = self.read8(ptr);

                let msb_addr =
                    (ptr & 0xFF00) |
                    ((ptr.wrapping_add(1)) & 0x00FF);

                let msb = self.read8(msb_addr);

                address_from_bytes(lsb, msb) as usize
            }

            AddressingMode::IndexedIndirect => {
                let ptr = self.x
                    .wrapping_add(self.read8(self.pc.wrapping_add(1)));

                // Must read byte by byte to preserve page wrapping behaviors
                let lsb = self.read8(ptr as u16);
                let msb = self.read8(ptr.wrapping_add(1) as u16);

                address_from_bytes(lsb, msb) as usize
            }

            AddressingMode::IndirectIndexed => {
                let ptr = self.read8(self.pc.wrapping_add(1));

                let lsb = self.read8(ptr as u16);
                let msb = self.read8(ptr.wrapping_add(1) as u16);

                address_from_bytes(lsb, msb).wrapping_add(self.y as u16) as usize
            }

            _ => unreachable!("mode has no address"),
        }
    }

    pub(crate) fn page_crossed(&self, mode: AddressingMode) -> bool {
        match mode {
            AddressingMode::AbsoluteX       => {
                let lsb = self.ram[self.pc.wrapping_add(1) as usize];
                let msb = self.ram[self.pc.wrapping_add(2) as usize];

                let address_before_add = address_from_bytes(lsb, msb);

                let page_crossed =
                    get_bits(address_before_add, 0..8) + self.x as u16 > 0xFF;

                page_crossed
            }

            AddressingMode::AbsoluteY       => {
                let lsb = self.ram[self.pc.wrapping_add(1) as usize];
                let msb = self.ram[self.pc.wrapping_add(2) as usize];

                let address_before_add = address_from_bytes(lsb, msb);

                let page_crossed =
                    get_bits(address_before_add, 0..8) + self.y as u16 > 0xFF;

                page_crossed
            },

            AddressingMode::IndirectIndexed => {
                let ptr = self.ram[self.pc.wrapping_add(1) as usize];

                let lsb = self.ram[ptr as usize];
                let msb = self.ram[ptr.wrapping_add(1) as usize];

                let address_before_add = address_from_bytes(lsb, msb);

                let page_crossed =
                    get_bits(address_before_add, 0..8) + self.y as u16 > 0xFF;

                page_crossed
            },

            AddressingMode::Relative => {
                let offset = self.ram[self.pc.wrapping_add(1) as usize] as i8;

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


    pub(crate) fn get8(&self, mode: AddressingMode) -> u8 {
        match mode {
            AddressingMode::Accumulator => self.a,

            AddressingMode::Immediate |
            AddressingMode::Relative => {
                self.read8(self.pc.wrapping_add(1))
            }

            _ => {
                let addr = self.get_address(mode);
                self.read8(addr as u16)
            }
        }
    }

    pub(crate) fn set8(&mut self, mode: AddressingMode, value: u8) {
        match mode {
            AddressingMode::Accumulator => {
                self.a = value;
            }

            AddressingMode::Immediate |
            AddressingMode::Relative => {
                panic!("cannot write to immediate/relative operand");
            }

            _ => {
                let addr = self.get_address(mode) as u16;
                self.write8(addr, value);
            }
        }
    }

    pub fn dbg(&self) {
        println!("PC: {:X}\n    NV  DIZC\nP:  {:08b}\nSP: {}\nA:  {}\nX:  {}\nY:  {}", self.pc, self.p, self.sp, self.a, self.x, self.y);
    }
}