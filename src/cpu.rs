use crate::bit_utils::{address_from_bytes};


#[repr(usize)]
pub enum CPUFlags {
    Carry               = 0,
    Zero                = 1,
    InterruptDisable    = 2,
    Decimal             = 3,
    //                  B
    //                  1
    Overflow            = 4,
    Negative            = 5,
}

enum AddressingMode {
    // Implicit,

    Accumulator,

    Immediate,

    ZeroPage,
    
    ZeroPageX,

    ZeroPageY,

    Relative,

    // Absolute,

    AbsoluteX,

    AbsoluteY,

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

    pub ram:    Box<[u8; 0x8000]>,
}

impl CPU {
    pub fn new() -> CPU {
        CPU
        {
            pc:     0,
            sp:     0xFF,
            p:      0,
            a:      0,
            x:      0,
            y:      0,
            ram: Box::new([0; 0x8000]),
        }
    }

    pub fn push8(&mut self, b: u8) {
        if self.sp == 0x00 {
            panic!("Tried to push onto a full stack.");
        }

        self.ram[0x100 + self.sp as usize] = b;
        self.sp -= 1;
    }

    pub fn pop8(&mut self) -> u8 {
        if self.sp == 0xFF {
            panic!("Tried to pop an empty stack.");
        }

        self.sp += 1;
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


    fn get_address(&self, mode: AddressingMode) -> usize {  
        match mode {
            AddressingMode::ZeroPage => {
                self.ram[self.pc as usize + 1] as usize
            }

            AddressingMode::ZeroPageX => {
                self.x
                    .wrapping_add(self.ram[self.pc as usize + 1])
                    as usize
            }

            AddressingMode::ZeroPageY => {
                self.y
                    .wrapping_add(self.ram[self.pc as usize + 1])
                    as usize
            }

            AddressingMode::AbsoluteX => {
                let lsb = self.ram[self.pc as usize + 1];
                let msb = self.ram[self.pc as usize + 2];

                (address_from_bytes(lsb, msb) + self.x as u16) as usize
            }

            AddressingMode::AbsoluteY => {
                let lsb = self.ram[self.pc as usize + 1];
                let msb = self.ram[self.pc as usize + 2];

                (address_from_bytes(lsb, msb) + self.y as u16) as usize
            }

            AddressingMode::IndexedIndirect => {
                let ptr = self.x
                    .wrapping_add(self.ram[self.pc as usize + 1]);

                let lsb = self.ram[ptr as usize];
                let msb = self.ram[ptr.wrapping_add(1) as usize];

                address_from_bytes(lsb, msb) as usize
            }

            AddressingMode::IndirectIndexed => {
                let ptr = self.ram[self.pc as usize + 1];

                let lsb = self.ram[ptr as usize];
                let msb = self.ram[ptr.wrapping_add(1) as usize];

                address_from_bytes(lsb, msb) as usize + self.y as usize
            }

            _ => unreachable!("mode has no address"),
        }
    }


    fn get8(&self, mode: AddressingMode) -> u8 {
        match mode {
            AddressingMode::Accumulator => self.a,

            AddressingMode::Immediate |
            AddressingMode::Relative => {
                self.ram[self.pc as usize + 1]
            }

            _ => {
                let addr = self.get_address(mode);
                self.ram[addr]
            }
        }
    }

    fn set8(&mut self, mode: AddressingMode, value: u8) {
        match mode {
            AddressingMode::Accumulator => {
                self.a = value;
            }

            AddressingMode::Immediate |
            AddressingMode::Relative => {
                panic!("cannot write to immediate/relative operand");
            }

            _ => {
                let addr = self.get_address(mode);
                self.ram[addr] = value;
            }
        }
    }
}