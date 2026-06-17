use crate::{
    bit_utils::{bit_set, nth_bit, set_bit},
    cpu::{AddressingMode, CPU, CPUFlags}, memory_bus::MemoryBus,
};

#[allow(unused)]
impl CPU {
    fn set_zn_flags(&mut self, value: u8) {
        self.set_flag(CPUFlags::Zero, value == 0);
        self.set_flag(CPUFlags::Negative, bit_set(value, 7));
    }

    fn advance(&mut self, mode: AddressingMode) {
        self.pc = self.pc.wrapping_add(match mode {
            AddressingMode::Implied => 1,
            AddressingMode::Accumulator => 1,
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 2,
            AddressingMode::ZeroPageX => 2,
            AddressingMode::ZeroPageY => 2,
            AddressingMode::Relative => 2,
            AddressingMode::Absolute => 3,
            AddressingMode::AbsoluteX => 3,
            AddressingMode::AbsoluteY => 3,
            AddressingMode::Indirect => 3,
            AddressingMode::IndexedIndirect => 2,
            AddressingMode::IndirectIndexed => 2,
        });
    }

    fn access_cycles(&self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        match mode {
            AddressingMode::Immediate => 2,

            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::ZeroPageY => 4,

            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => 4 + (if self.page_crossed(bus, mode) { 1 } else { 0 }),
            AddressingMode::AbsoluteY => 4 + (if self.page_crossed(bus, mode) { 1 } else { 0 }),

            AddressingMode::IndexedIndirect => 6,
            AddressingMode::IndirectIndexed => 5 + (if self.page_crossed(bus, mode) { 1 } else { 0 }),

            _ => unreachable!(),
        }
    }

    fn arithmetic_cycles(&self, mode: AddressingMode) -> usize {
        match mode {
            AddressingMode::Accumulator => 2,

            AddressingMode::ZeroPage => 5,
            AddressingMode::ZeroPageX => 6,

            AddressingMode::Absolute => 6,
            AddressingMode::AbsoluteX => 7,

            _ => unreachable!(),
        }
    }

    fn store_cycles(&self, mode: AddressingMode) -> usize {
        match mode {
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX => 4,
            AddressingMode::ZeroPageY => 4,

            AddressingMode::Absolute => 4,
            AddressingMode::AbsoluteX => 5,
            AddressingMode::AbsoluteY => 5,

            AddressingMode::IndexedIndirect => 6,
            AddressingMode::IndirectIndexed => 6,

            _ => unreachable!(),
        }
    }

    // Access
    fn lda(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let byte = self.get8(bus, mode);

        self.a = byte;

        self.set_zn_flags(self.a);
        let cycles = self.access_cycles(bus, mode);
        self.advance(mode);
        cycles
    }

    fn sta(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.set8(bus, mode, self.a);
        let cycles = self.store_cycles(mode);
        self.advance(mode);
        cycles
    }

    fn ldx(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let byte = self.get8(bus, mode);

        self.x = byte;
        self.set_zn_flags(self.x);
        let cycles = self.access_cycles(bus, mode);
        self.advance(mode);
        cycles
    }

    fn stx(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.set8(bus, mode, self.x);
        let cycles = self.store_cycles(mode);
        self.advance(mode);
        cycles
    }

    fn ldy(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let byte = self.get8(bus, mode);

        self.y = byte;
        self.set_zn_flags(self.y);
        let cycles = self.access_cycles(bus, mode);
        self.advance(mode);
        cycles
    }

    fn sty(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.set8(bus, mode, self.y);
        let cycles = self.store_cycles(mode);
        self.advance(mode);
        cycles
    }

    // Transfer
    fn tax(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.x = self.a;
        self.set_zn_flags(self.a);
        self.advance(mode);

        2
    }

    fn txa(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.a = self.x;
        self.set_zn_flags(self.x);
        self.advance(mode);

        2
    }

    fn tay(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.y = self.a;
        self.set_zn_flags(self.a);
        self.advance(mode);

        2
    }

    fn tya(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.a = self.y;
        self.set_zn_flags(self.y);
        self.advance(mode);

        2
    }

    // Arithmetic
    fn adc(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let value = self.get8(bus, mode);
        let carry_in = self.flag_set(CPUFlags::Carry) as u8;

        let (sum1, carry1) = self.a.overflowing_add(value);
        let (result, carry2) = sum1.overflowing_add(carry_in);

        let carry = carry1 || carry2;

        self.set_flag(CPUFlags::Carry, carry);
        self.set_flag(CPUFlags::Zero, result == 0);

        self.set_flag(
            CPUFlags::Overflow,
            bit_set((self.a ^ result) & (value ^ result), 7),
        );

        self.set_flag(CPUFlags::Negative, bit_set(result, 7));

        self.a = result;
        let cycles = self.access_cycles(bus, mode);
        self.advance(mode);
        cycles
    }

    fn sbc(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let value = !self.get8(bus, mode);
        let carry_in = self.flag_set(CPUFlags::Carry) as u8;

        let (sum1, carry1) = self.a.overflowing_add(value);
        let (result, carry2) = sum1.overflowing_add(carry_in);

        let carry = carry1 || carry2;

        self.set_flag(CPUFlags::Carry, carry);
        self.set_flag(CPUFlags::Zero, result == 0);

        self.set_flag(
            CPUFlags::Overflow,
            bit_set((self.a ^ result) & (value ^ result), 7),
        );

        self.set_flag(CPUFlags::Negative, bit_set(result, 7));

        self.a = result;
        let cycles = self.access_cycles(bus, mode);
        self.advance(mode);
        cycles
    }

    fn inc(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let byte = self.get8(bus, mode).wrapping_add(1);
        self.set8(bus, mode, byte);

        self.set_zn_flags(byte);

        let cycles = self.arithmetic_cycles(mode);
        self.advance(mode);
        cycles
    }

    fn dec(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let byte = self.get8(bus, mode).wrapping_sub(1);
        self.set8(bus, mode, byte);

        self.set_zn_flags(byte);

        let cycles = self.arithmetic_cycles(mode);
        self.advance(mode);
        cycles
    }

    fn inx(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.x = self.x.wrapping_add(1);

        self.set_zn_flags(self.x);
        self.advance(mode);
        2
    }

    fn dex(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.x = self.x.wrapping_sub(1);

        self.set_zn_flags(self.x);
        self.advance(mode);
        2
    }

    fn iny(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.y = self.y.wrapping_add(1);

        self.set_zn_flags(self.y);
        self.advance(mode);
        2
    }

    fn dey(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.y = self.y.wrapping_sub(1);

        self.set_zn_flags(self.y);
        self.advance(mode);
        2
    }

    // Shift
    fn asl(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let value = self.get8(bus, mode);

        self.p = set_bit(self.p, CPUFlags::Carry as usize, bit_set(value, 7));
        let result = value << 1;

        self.set_zn_flags(result);
        self.set8(bus, mode, result);

        let cycles = self.arithmetic_cycles(mode);
        self.advance(mode);
        cycles
    }

    fn lsr(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let value = self.get8(bus, mode);

        self.p = set_bit(self.p, CPUFlags::Carry as usize, bit_set(value, 0));
        let result = value >> 1;

        self.set_zn_flags(result);
        self.set8(bus, mode, result);

        let cycles = self.arithmetic_cycles(mode);
        self.advance(mode);
        cycles
    }

    fn rol(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let value = self.get8(bus, mode);

        let carry_in = self.flag_set(CPUFlags::Carry) as u8;
        let carry_out = bit_set(value, 7);

        let result = (value << 1) | carry_in;

        self.set_flag(CPUFlags::Carry, carry_out);
        self.set_zn_flags(result);
        self.set8(bus, mode, result);

        let cycles = self.arithmetic_cycles(mode);
        self.advance(mode);
        cycles
    }

    fn ror(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let value = self.get8(bus, mode);

        let carry_in = (self.flag_set(CPUFlags::Carry) as u8) << 7;
        let carry_out = bit_set(value, 0);

        let result = (value >> 1) | carry_in;

        self.set_flag(CPUFlags::Carry, carry_out);
        self.set_zn_flags(result);
        self.set8(bus, mode, result);

        let cycles = self.arithmetic_cycles(mode);
        self.advance(mode);
        cycles
    }

    // Bitwise
    fn and(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let value = self.get8(bus, mode);
        self.a &= value;

        self.set_zn_flags(self.a);
        let cycles = self.access_cycles(bus, mode);
        self.advance(mode);
        cycles
    }

    fn ora(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let value = self.get8(bus, mode);
        self.a |= value;

        self.set_zn_flags(self.a);
        let cycles = self.access_cycles(bus, mode);
        self.advance(mode);
        cycles
    }

    fn eor(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let value = self.get8(bus, mode);
        self.a ^= value;

        self.set_zn_flags(self.a);
        let cycles = self.access_cycles(bus, mode);
        self.advance(mode);
        cycles
    }

    fn bit(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let value = self.get8(bus, mode);
        let result = self.a & value;

        self.set_flag(CPUFlags::Zero, result == 0);
        self.set_flag(CPUFlags::Overflow, bit_set(value, 6));
        self.set_flag(CPUFlags::Negative, bit_set(value, 7));

        self.advance(mode);
        match mode {
            AddressingMode::ZeroPage => 3,
            AddressingMode::Absolute => 4,
            _ => unreachable!(),
        }
    }

    // Compare
    fn cmp(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let value = self.get8(bus, mode);

        self.set_flag(CPUFlags::Carry, self.a >= value);
        self.set_flag(CPUFlags::Zero, self.a == value);
        self.set_flag(CPUFlags::Negative, bit_set(self.a.wrapping_sub(value), 7));
        let cycles = self.access_cycles(bus, mode);
        self.advance(mode);
        cycles
    }

    fn cpx(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let value = self.get8(bus, mode);

        self.set_flag(CPUFlags::Carry, self.x >= value);
        self.set_flag(CPUFlags::Zero, self.x == value);
        self.set_flag(CPUFlags::Negative, bit_set(self.x.wrapping_sub(value), 7));
        let cycles = self.access_cycles(bus, mode);
        self.advance(mode);
        cycles
    }

    fn cpy(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let value = self.get8(bus, mode);

        self.set_flag(CPUFlags::Carry, self.y >= value);
        self.set_flag(CPUFlags::Zero, self.y == value);
        self.set_flag(CPUFlags::Negative, bit_set(self.y.wrapping_sub(value), 7));
        let cycles = self.access_cycles(bus, mode);
        self.advance(mode);
        cycles
    }

    fn branch(&mut self, bus: &mut MemoryBus, mode: AddressingMode, b: bool) -> usize {
        if b {
            let branch_position = self.get_address(bus, mode);
            let crossed_page = self.page_crossed(bus, mode);

            self.pc = branch_position as u16;

            3 + if crossed_page { 1 } else { 0 }
        } else {
            self.advance(mode);

            2
        }
    }

    // Branch
    fn bcc(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.branch(bus, mode, !self.flag_set(CPUFlags::Carry))
    }

    fn bcs(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.branch(bus, mode, self.flag_set(CPUFlags::Carry))
    }

    fn beq(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.branch(bus, mode, self.flag_set(CPUFlags::Zero))
    }

    fn bne(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.branch(bus, mode, !self.flag_set(CPUFlags::Zero))
    }

    fn bpl(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.branch(bus, mode, !self.flag_set(CPUFlags::Negative))
    }

    fn bmi(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.branch(bus, mode, self.flag_set(CPUFlags::Negative))
    }

    fn bvc(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.branch(bus, mode, !self.flag_set(CPUFlags::Overflow))
    }

    fn bvs(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.branch(bus, mode, self.flag_set(CPUFlags::Overflow))
    }

    // Jump
    fn jmp(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let addr = self.get_address(bus, mode);
        self.pc = addr as u16;

        match mode {
            AddressingMode::Absolute => 3,
            AddressingMode::Indirect => 5,

            _ => unreachable!(),
        }
    }

    fn jsr(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let ret_target = self.pc + 2;
        self.push16(bus, ret_target);

        let jmp_target = self.get_address(bus, mode);
        self.pc = jmp_target as u16;

        6
    }

    fn rts(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let ret_target = self.pop16(bus);
        self.pc = ret_target + 1;

        6
    }

    fn brk(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        let ret_target = self.pc + 2;
        self.push16(bus, ret_target);

        self.push8(bus, self.p | nth_bit::<u8>(4) | nth_bit::<u8>(5));

        self.set_flag(CPUFlags::InterruptDisable, true);
        self.jump_to_interrupt_handler(bus);

        7
    }

    fn rti(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.p = self.pop8(bus) & !nth_bit::<u8>(4) | nth_bit::<u8>(5);
        self.pc = self.pop16(bus);

        6
    }

    // Stack
    fn pha(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.push8(bus, self.a);
        self.advance(mode);

        3
    }

    fn pla(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.a = self.pop8(bus);
        self.set_zn_flags(self.a);
        self.advance(mode);

        4
    }

    fn php(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.push8(bus, self.p | nth_bit::<u8>(4));
        self.advance(mode);

        3
    }

    fn plp(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.p = self.pop8(bus) & !nth_bit::<u8>(4) | nth_bit::<u8>(5);
        self.advance(mode);

        4
    }

    fn txs(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.sp = self.x;
        self.advance(mode);

        2
    }

    fn tsx(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.x = self.sp;
        self.set_zn_flags(self.x);
        self.advance(mode);

        2
    }

    // Flags
    fn clc(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.set_flag(CPUFlags::Carry, false);
        self.advance(mode);

        2
    }
    fn sec(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.set_flag(CPUFlags::Carry, true);
        self.advance(mode);

        2
    }

    fn cli(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.set_flag(CPUFlags::InterruptDisable, false);
        self.advance(mode);

        2
    }

    fn sei(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.set_flag(CPUFlags::InterruptDisable, true);
        self.advance(mode);

        2
    }

    fn cld(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.set_flag(CPUFlags::Decimal, false);
        self.advance(mode);

        2
    }

    fn sed(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.set_flag(CPUFlags::Decimal, true);
        self.advance(mode);

        2
    }

    fn clv(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.set_flag(CPUFlags::Overflow, false);
        self.advance(mode);

        2
    }

    // Other
    fn nop(&mut self, bus: &mut MemoryBus, mode: AddressingMode) -> usize {
        self.advance(mode);

        2
    }

    fn execute(&mut self, bus: &mut MemoryBus, opcode: u8) -> Option<usize> {
        Some(match opcode {
            0x69 => self.adc(bus, AddressingMode::Immediate),
            0x65 => self.adc(bus, AddressingMode::ZeroPage),
            0x75 => self.adc(bus, AddressingMode::ZeroPageX),
            0x6D => self.adc(bus, AddressingMode::Absolute),
            0x7D => self.adc(bus, AddressingMode::AbsoluteX),
            0x79 => self.adc(bus, AddressingMode::AbsoluteY),
            0x61 => self.adc(bus, AddressingMode::IndexedIndirect),
            0x71 => self.adc(bus, AddressingMode::IndirectIndexed),

            0x29 => self.and(bus, AddressingMode::Immediate),
            0x25 => self.and(bus, AddressingMode::ZeroPage),
            0x35 => self.and(bus, AddressingMode::ZeroPageX),
            0x2D => self.and(bus, AddressingMode::Absolute),
            0x3D => self.and(bus, AddressingMode::AbsoluteX),
            0x39 => self.and(bus, AddressingMode::AbsoluteY),
            0x21 => self.and(bus, AddressingMode::IndexedIndirect),
            0x31 => self.and(bus, AddressingMode::IndirectIndexed),

            0x0A => self.asl(bus, AddressingMode::Accumulator),
            0x06 => self.asl(bus, AddressingMode::ZeroPage),
            0x16 => self.asl(bus, AddressingMode::ZeroPageX),
            0x0E => self.asl(bus, AddressingMode::Absolute),
            0x1E => self.asl(bus, AddressingMode::AbsoluteX),

            0x90 => self.bcc(bus, AddressingMode::Relative),
            0xB0 => self.bcs(bus, AddressingMode::Relative),
            0xF0 => self.beq(bus, AddressingMode::Relative),
            0x30 => self.bmi(bus, AddressingMode::Relative),
            0xD0 => self.bne(bus, AddressingMode::Relative),
            0x10 => self.bpl(bus, AddressingMode::Relative),
            0x50 => self.bvc(bus, AddressingMode::Relative),
            0x70 => self.bvs(bus, AddressingMode::Relative),

            0x24 => self.bit(bus, AddressingMode::ZeroPage),
            0x2C => self.bit(bus, AddressingMode::Absolute),

            0x00 => self.brk(bus, AddressingMode::Immediate),

            0x18 => self.clc(bus, AddressingMode::Implied),
            0xD8 => self.cld(bus, AddressingMode::Implied),
            0x58 => self.cli(bus, AddressingMode::Implied),
            0xB8 => self.clv(bus, AddressingMode::Implied),

            0xC9 => self.cmp(bus, AddressingMode::Immediate),
            0xC5 => self.cmp(bus, AddressingMode::ZeroPage),
            0xD5 => self.cmp(bus, AddressingMode::ZeroPageX),
            0xCD => self.cmp(bus, AddressingMode::Absolute),
            0xDD => self.cmp(bus, AddressingMode::AbsoluteX),
            0xD9 => self.cmp(bus, AddressingMode::AbsoluteY),
            0xC1 => self.cmp(bus, AddressingMode::IndexedIndirect),
            0xD1 => self.cmp(bus, AddressingMode::IndirectIndexed),

            0xE0 => self.cpx(bus, AddressingMode::Immediate),
            0xE4 => self.cpx(bus, AddressingMode::ZeroPage),
            0xEC => self.cpx(bus, AddressingMode::Absolute),

            0xC0 => self.cpy(bus, AddressingMode::Immediate),
            0xC4 => self.cpy(bus, AddressingMode::ZeroPage),
            0xCC => self.cpy(bus, AddressingMode::Absolute),

            0xE6 => self.inc(bus, AddressingMode::ZeroPage),
            0xF6 => self.inc(bus, AddressingMode::ZeroPageX),
            0xEE => self.inc(bus, AddressingMode::Absolute),
            0xFE => self.inc(bus, AddressingMode::AbsoluteX),

            0xC6 => self.dec(bus, AddressingMode::ZeroPage),
            0xD6 => self.dec(bus, AddressingMode::ZeroPageX),
            0xCE => self.dec(bus, AddressingMode::Absolute),
            0xDE => self.dec(bus, AddressingMode::AbsoluteX),

            0xE8 => self.inx(bus, AddressingMode::Implied),
            0xCA => self.dex(bus, AddressingMode::Implied),

            0xC8 => self.iny(bus, AddressingMode::Implied),
            0x88 => self.dey(bus, AddressingMode::Implied),

            0x49 => self.eor(bus, AddressingMode::Immediate),
            0x45 => self.eor(bus, AddressingMode::ZeroPage),
            0x55 => self.eor(bus, AddressingMode::ZeroPageX),
            0x4D => self.eor(bus, AddressingMode::Absolute),
            0x5D => self.eor(bus, AddressingMode::AbsoluteX),
            0x59 => self.eor(bus, AddressingMode::AbsoluteY),
            0x41 => self.eor(bus, AddressingMode::IndexedIndirect),
            0x51 => self.eor(bus, AddressingMode::IndirectIndexed),

            0x4C => self.jmp(bus, AddressingMode::Absolute),
            0x6C => self.jmp(bus, AddressingMode::Indirect),

            0x20 => self.jsr(bus, AddressingMode::Absolute),

            0xA9 => self.lda(bus, AddressingMode::Immediate),
            0xA5 => self.lda(bus, AddressingMode::ZeroPage),
            0xB5 => self.lda(bus, AddressingMode::ZeroPageX),
            0xAD => self.lda(bus, AddressingMode::Absolute),
            0xBD => self.lda(bus, AddressingMode::AbsoluteX),
            0xB9 => self.lda(bus, AddressingMode::AbsoluteY),
            0xA1 => self.lda(bus, AddressingMode::IndexedIndirect),
            0xB1 => self.lda(bus, AddressingMode::IndirectIndexed),

            0xA2 => self.ldx(bus, AddressingMode::Immediate),
            0xA6 => self.ldx(bus, AddressingMode::ZeroPage),
            0xB6 => self.ldx(bus, AddressingMode::ZeroPageY),
            0xAE => self.ldx(bus, AddressingMode::Absolute),
            0xBE => self.ldx(bus, AddressingMode::AbsoluteY),

            0xA0 => self.ldy(bus, AddressingMode::Immediate),
            0xA4 => self.ldy(bus, AddressingMode::ZeroPage),
            0xB4 => self.ldy(bus, AddressingMode::ZeroPageX),
            0xAC => self.ldy(bus, AddressingMode::Absolute),
            0xBC => self.ldy(bus, AddressingMode::AbsoluteX),

            0x4A => self.lsr(bus, AddressingMode::Accumulator),
            0x46 => self.lsr(bus, AddressingMode::ZeroPage),
            0x56 => self.lsr(bus, AddressingMode::ZeroPageX),
            0x4E => self.lsr(bus, AddressingMode::Absolute),
            0x5E => self.lsr(bus, AddressingMode::AbsoluteX),

            0xEA => self.nop(bus, AddressingMode::Implied),

            0x09 => self.ora(bus, AddressingMode::Immediate),
            0x05 => self.ora(bus, AddressingMode::ZeroPage),
            0x15 => self.ora(bus, AddressingMode::ZeroPageX),
            0x0D => self.ora(bus, AddressingMode::Absolute),
            0x1D => self.ora(bus, AddressingMode::AbsoluteX),
            0x19 => self.ora(bus, AddressingMode::AbsoluteY),
            0x01 => self.ora(bus, AddressingMode::IndexedIndirect),
            0x11 => self.ora(bus, AddressingMode::IndirectIndexed),

            0x48 => self.pha(bus, AddressingMode::Implied),
            0x08 => self.php(bus, AddressingMode::Implied),
            0x68 => self.pla(bus, AddressingMode::Implied),
            0x28 => self.plp(bus, AddressingMode::Implied),

            0x2A => self.rol(bus, AddressingMode::Accumulator),
            0x26 => self.rol(bus, AddressingMode::ZeroPage),
            0x36 => self.rol(bus, AddressingMode::ZeroPageX),
            0x2E => self.rol(bus, AddressingMode::Absolute),
            0x3E => self.rol(bus, AddressingMode::AbsoluteX),

            0x6A => self.ror(bus, AddressingMode::Accumulator),
            0x66 => self.ror(bus, AddressingMode::ZeroPage),
            0x76 => self.ror(bus, AddressingMode::ZeroPageX),
            0x6E => self.ror(bus, AddressingMode::Absolute),
            0x7E => self.ror(bus, AddressingMode::AbsoluteX),

            0x40 => self.rti(bus, AddressingMode::Implied),

            0x60 => self.rts(bus, AddressingMode::Implied),

            0xE9 => self.sbc(bus, AddressingMode::Immediate),
            0xE5 => self.sbc(bus, AddressingMode::ZeroPage),
            0xF5 => self.sbc(bus, AddressingMode::ZeroPageX),
            0xED => self.sbc(bus, AddressingMode::Absolute),
            0xFD => self.sbc(bus, AddressingMode::AbsoluteX),
            0xF9 => self.sbc(bus, AddressingMode::AbsoluteY),
            0xE1 => self.sbc(bus, AddressingMode::IndexedIndirect),
            0xF1 => self.sbc(bus, AddressingMode::IndirectIndexed),

            0x38 => self.sec(bus, AddressingMode::Implied),
            0xF8 => self.sed(bus, AddressingMode::Implied),
            0x78 => self.sei(bus, AddressingMode::Implied),

            0x85 => self.sta(bus, AddressingMode::ZeroPage),
            0x95 => self.sta(bus, AddressingMode::ZeroPageX),
            0x8D => self.sta(bus, AddressingMode::Absolute),
            0x9D => self.sta(bus, AddressingMode::AbsoluteX),
            0x99 => self.sta(bus, AddressingMode::AbsoluteY),
            0x81 => self.sta(bus, AddressingMode::IndexedIndirect),
            0x91 => self.sta(bus, AddressingMode::IndirectIndexed),

            0x86 => self.stx(bus, AddressingMode::ZeroPage),
            0x96 => self.stx(bus, AddressingMode::ZeroPageY),
            0x8E => self.stx(bus, AddressingMode::Absolute),

            0x84 => self.sty(bus, AddressingMode::ZeroPage),
            0x94 => self.sty(bus, AddressingMode::ZeroPageX),
            0x8C => self.sty(bus, AddressingMode::Absolute),

            0xAA => self.tax(bus, AddressingMode::Implied),

            0xA8 => self.tay(bus, AddressingMode::Implied),

            0xBA => self.tsx(bus, AddressingMode::Implied),

            0x8A => self.txa(bus, AddressingMode::Implied),

            0x9A => self.txs(bus, AddressingMode::Implied),

            0x98 => self.tya(bus, AddressingMode::Implied),

            _ => return None,
        })
    }

    pub fn step(&mut self, bus: &mut MemoryBus) -> bool {
        if self.delay > 0 {
            self.delay -= 1;
            self.cycles += 1;

            return true;
        }

        let opcode = self.read8(bus, self.pc);
        if let Some(cycles) = self.execute(bus, opcode) {
            self.cycles += 1;
            self.delay = cycles - 1;

            return true;
        }

        false
    }
}
