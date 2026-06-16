use crate::{bit_utils::{bit_set, set_bit}, cpu::{AddressingMode, CPU, CPUFlags}};

#[allow(unused)]
impl CPU {
    fn set_zn_flags(&mut self, value: u8) {
        self.p = set_bit(self.p, CPUFlags::Zero as usize,                value == 0 );
        self.p = set_bit(self.p, CPUFlags::Negative as usize,    bit_set(value,   7));
    }

    fn advance(&mut self, mode: AddressingMode) {
        self.pc += match mode {
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
            AddressingMode::IndexedIndirect => 3,
            AddressingMode::IndirectIndexed => 3,
        }
    }

    fn access_cycles(&self, mode: AddressingMode) -> usize {
        match mode {
            AddressingMode::Immediate   => 2,

            AddressingMode::ZeroPage    => 3,
            AddressingMode::ZeroPageX   => 4,
            AddressingMode::ZeroPageY   => 4,

            AddressingMode::Absolute    => 4,
            AddressingMode::AbsoluteX   => 4 + (if self.page_crossed(mode) { 1 } else { 0 }),
            AddressingMode::AbsoluteY   => 4 + (if self.page_crossed(mode) { 1 } else { 0 }),

            AddressingMode::IndexedIndirect => 6,
            AddressingMode::IndirectIndexed => 5 + (if self.page_crossed(mode) { 1 } else { 0 }),

            _ => unreachable!()
        }
    }

    fn arithmetic_cycles(&self, mode: AddressingMode) -> usize {
        match mode {
            AddressingMode::Accumulator => 2,

            AddressingMode::ZeroPage    => 5,
            AddressingMode::ZeroPageX   => 6,

            AddressingMode::Absolute    => 6,
            AddressingMode::AbsoluteX   => 7,

            _ => unreachable!()
        }
    }

    fn store_cycles(&self, mode: AddressingMode) -> usize {
        match mode {
            AddressingMode::ZeroPage    => 3,
            AddressingMode::ZeroPageX   => 4,
            AddressingMode::ZeroPageY   => 4,

            AddressingMode::Absolute    => 4,
            AddressingMode::AbsoluteX   => 5,
            AddressingMode::AbsoluteY   => 5,

            AddressingMode::IndexedIndirect => 6,
            AddressingMode::IndirectIndexed => 6,

            _ => unreachable!()
        }
    }

    // Access
    fn lda(&mut self, mode: AddressingMode) -> usize {
        let byte = self.get8(mode);

        self.a = byte;
        
        self.advance(mode);
        self.access_cycles(mode)
    }

    fn sta(&mut self, mode: AddressingMode) -> usize {
        self.set8(mode, self.a);
        self.advance(mode);
        self.store_cycles(mode)
    }

    fn ldx(&mut self, mode: AddressingMode) -> usize {
        let byte = self.get8(mode);

        self.x = byte;
        self.advance(mode);
        self.access_cycles(mode)
    }

    fn stx(&mut self, mode: AddressingMode) -> usize {
        self.set8(mode, self.x);
        self.advance(mode);
        self.store_cycles(mode)
    }

    fn ldy(&mut self, mode: AddressingMode) -> usize {
        let byte = self.get8(mode);

        self.y = byte;
        self.advance(mode);
        self.access_cycles(mode)
    }

    fn sty(&mut self, mode: AddressingMode) -> usize {
        self.set8(mode, self.y);
        self.advance(mode);
        self.store_cycles(mode)
    }

    // Transfer
    fn tax(&mut self, mode: AddressingMode) -> usize {
        self.x = self.a;
        self.set_zn_flags(self.a);
        self.advance(mode);

        2
    }

    fn txa(&mut self, mode: AddressingMode) -> usize {
        self.a = self.x;
        self.set_zn_flags(self.x);
        self.advance(mode);

        2
    }

    fn tay(&mut self, mode: AddressingMode) -> usize {
        self.y = self.a;
        self.set_zn_flags(self.a);
        self.advance(mode);

        2
    }

    fn tya(&mut self, mode: AddressingMode) -> usize {
        self.a = self.y;
        self.set_zn_flags(self.y);
        self.advance(mode);

        2
    }

    // Arithmetic
    fn adc(&mut self, mode: AddressingMode) -> usize {
        let addend = self.get8(mode);
        let carry_in = self.flag_set(CPUFlags::Carry) as u8;

        let sum1 = self.a.wrapping_add(addend);
        let (result, carry1) = sum1.overflowing_add(carry_in);

        self.set_flag(CPUFlags::Carry, carry1);

        self.set_flag(CPUFlags::Zero, result == 0);

        self.set_flag(
            CPUFlags::Overflow,
            bit_set((self.a ^ result) & (addend ^ result), 7),
        );

        self.set_flag(CPUFlags::Negative, bit_set(result, 7));

        self.a = result;

        self.advance(mode);
        self.access_cycles(mode)
    }

    fn sbc(&mut self, mode: AddressingMode) -> usize {
        let value = !self.get8(mode);
        let carry_in = self.flag_set(CPUFlags::Carry) as u8;

        let sum1 = self.a.wrapping_add(value);
        let (result, carry1) = sum1.overflowing_add(carry_in);

        self.set_flag(CPUFlags::Carry, carry1);
        self.set_flag(CPUFlags::Zero, result == 0);

        self.set_flag(
            CPUFlags::Overflow,
            bit_set((self.a ^ result) & (value ^ result), 7),
        );

        self.set_flag(CPUFlags::Negative, bit_set(result, 7));

        self.a = result;

        self.advance(mode);
        self.access_cycles(mode)
    }

    fn inc(&mut self, mode: AddressingMode) -> usize {
        let byte = self.get8(mode).wrapping_add(1);
        self.set8(mode, byte);

        self.set_zn_flags(byte);

        self.advance(mode);
        self.arithmetic_cycles(mode)
    }

    fn dec(&mut self, mode: AddressingMode) -> usize {
        let byte = self.get8(mode).wrapping_sub(1);
        self.set8(mode, byte);

        self.set_zn_flags(byte);

        self.advance(mode);
        self.arithmetic_cycles(mode)
    }

    fn inx(&mut self, mode: AddressingMode) -> usize {
        self.x = self.x.wrapping_add(1);

        self.advance(mode);
        2
    }

    fn dex(&mut self, mode: AddressingMode) -> usize {
        self.x = self.x.wrapping_sub(1);

        self.advance(mode);
        2
    }

    fn iny(&mut self, mode: AddressingMode) -> usize {
        self.y = self.y.wrapping_add(1);

        self.advance(mode);
        2
    }

    fn dey(&mut self, mode: AddressingMode) -> usize {
        self.y = self.y.wrapping_sub(1);

        self.advance(mode);
        2
    }

    // Shift
    fn asl(&mut self, mode: AddressingMode) -> usize {
        let value = self.get8(mode);

        self.p = set_bit(self.p, CPUFlags::Carry as usize, bit_set(value, 7));
        let result = value << 1;

        self.set_zn_flags(result);
        self.set8(mode, result);

        self.advance(mode);
        self.arithmetic_cycles(mode)
    }

    fn lsr(&mut self, mode: AddressingMode) -> usize {
        let value = self.get8(mode);

        self.p = set_bit(self.p, CPUFlags::Carry as usize, bit_set(value, 0));
        let result = value >> 1;

        self.set_zn_flags(result);
        self.set8(mode, result);

        self.advance(mode);
        self.arithmetic_cycles(mode)
    }

    fn rol(&mut self, mode: AddressingMode) -> usize {
        let value = self.get8(mode);

        let carry_in = self.flag_set(CPUFlags::Carry) as u8;
        let carry_out = bit_set(value, 7);

        let result = (value << 1) | carry_in;

        self.set_flag(CPUFlags::Carry, carry_out);
        self.set_zn_flags(result);
        self.set8(mode, result);

        self.advance(mode);
        self.arithmetic_cycles(mode)
    }

    fn ror(&mut self, mode: AddressingMode) -> usize {
        let value = self.get8(mode);

        let carry_in = (self.flag_set(CPUFlags::Carry) as u8) << 7;
        let carry_out = bit_set(value, 0);

        let result = (value >> 1) | carry_in;

        self.set_flag(CPUFlags::Carry, carry_out);
        self.set_zn_flags(result);
        self.set8(mode, result);

        self.advance(mode);
        self.arithmetic_cycles(mode)
    }

    // Bitwise
    fn and(&mut self, mode: AddressingMode) -> usize {
        let value = self.get8(mode);
        self.a &= value;

        self.set_zn_flags(self.a);
        self.advance(mode);
        self.access_cycles(mode)
    }

    fn ora(&mut self, mode: AddressingMode) -> usize {
        let value = self.get8(mode);
        self.a |= value;

        self.set_zn_flags(self.a);
        self.advance(mode);
        self.access_cycles(mode)
    }

    fn eor(&mut self, mode: AddressingMode) -> usize {
        let value = self.get8(mode);
        self.a ^= value;

        self.set_zn_flags(self.a);
        self.advance(mode);
        self.access_cycles(mode)
    }

    fn bit(&mut self, mode: AddressingMode) -> usize {
        let value = self.get8(mode);
        let result = self.a & value;

        self.set_flag(CPUFlags::Zero, result == 0);
        self.set_flag(CPUFlags::Overflow, bit_set(value, 6));
        self.set_flag(CPUFlags::Negative, bit_set(value, 7));

        self.advance(mode);
        match mode {
            AddressingMode::ZeroPage => 3,
            AddressingMode::Absolute => 4,
            _ => unreachable!()
        }
    }

    // Compare
    fn cmp(&mut self, mode: AddressingMode) -> usize {
        let value = self.get8(mode);

        self.set_flag(CPUFlags::Carry, self.a >= value);
        self.set_flag(CPUFlags::Zero,  self.a == value);
        self.set_flag(CPUFlags::Negative, bit_set(self.a.wrapping_sub(value), 7));

        self.advance(mode);
        self.access_cycles(mode)
    }

    fn cpx(&mut self, mode: AddressingMode) -> usize {
        let value = self.get8(mode);

        self.set_flag(CPUFlags::Carry, self.x >= value);
        self.set_flag(CPUFlags::Zero,  self.x == value);
        self.set_flag(CPUFlags::Negative, bit_set(self.x.wrapping_sub(value), 7));
        
        self.advance(mode);
        self.access_cycles(mode)
    }

    fn cpy(&mut self, mode: AddressingMode) -> usize {
        let value = self.get8(mode);

        self.set_flag(CPUFlags::Carry, self.y >= value);
        self.set_flag(CPUFlags::Zero,  self.y == value);
        self.set_flag(CPUFlags::Negative, bit_set(self.y.wrapping_sub(value), 7));
        
        self.advance(mode);
        self.access_cycles(mode)
    }

    fn branch(&mut self, mode: AddressingMode, b: bool) -> usize {
        if b {
            let branch_position = self.get_address(mode);
            let crossed_page = self.page_crossed(mode);

            self.pc = branch_position as u16;

            3 + if crossed_page { 1 } else { 0 }
        }
        else {
            self.advance(mode);
            
            2
        }
    }

    // Branch
    fn bcc(&mut self, mode: AddressingMode) -> usize {
        self.branch(mode, !self.flag_set(CPUFlags::Carry))
    }

    fn bcs(&mut self, mode: AddressingMode) -> usize {
        self.branch(mode, self.flag_set(CPUFlags::Carry))
    }

    fn beq(&mut self, mode: AddressingMode) -> usize {
        self.branch(mode, self.flag_set(CPUFlags::Zero))
    }

    fn bne(&mut self, mode: AddressingMode) -> usize {
        self.branch(mode, !self.flag_set(CPUFlags::Zero))
    }

    fn bpl(&mut self, mode: AddressingMode) -> usize {
        self.branch(mode, !self.flag_set(CPUFlags::Negative))
    }

    fn bmi(&mut self, mode: AddressingMode) -> usize {
        self.branch(mode, self.flag_set(CPUFlags::Negative))
    }

    fn bvc(&mut self, mode: AddressingMode) -> usize {
        self.branch(mode, !self.flag_set(CPUFlags::Overflow))
    }

    fn bvs(&mut self, mode: AddressingMode) -> usize {
        self.branch(mode, self.flag_set(CPUFlags::Overflow))
    }

    // Jump
    fn jmp(&mut self, mode: AddressingMode) -> usize {
        let addr = self.get_address(mode);
        self.pc = addr as u16;

        match mode {
            AddressingMode::Absolute => 3,
            AddressingMode::Indirect => 5,

            _ => unreachable!()
        }
    }

    fn jsr(&mut self, mode: AddressingMode) -> usize {
        let ret_target = self.pc + 2;
        self.push16(ret_target);

        let jmp_target = self.get_address(mode);
        self.pc = jmp_target as u16;

        6
    }

    fn rts(&mut self, mode: AddressingMode) -> usize {
        let ret_target = self.pop16();
        self.pc = ret_target + 1;

        6
    }

    fn brk(&mut self, mode: AddressingMode) -> usize {
        let ret_target = self.pc + 2;
        self.push16(ret_target);
        self.push8(self.p);

        self.set_flag(CPUFlags::InterruptDisable, true);
        self.jump_to_interrupt_handler();

        7
    }

    fn rti(&mut self, mode: AddressingMode) -> usize {
        self.p = self.pop8();
        self.pc = self.pop16();

        6
    }

    // Stack
    fn pha(&mut self, mode: AddressingMode) -> usize {
        self.push8(self.a);
        self.advance(mode);

        3
    }

    fn pla(&mut self, mode: AddressingMode) -> usize {
        self.a = self.pop8();
        self.set_zn_flags(self.a);
        self.advance(mode);

        4
    }

    fn php(&mut self, mode: AddressingMode) -> usize {
        self.push8(self.p);
        self.advance(mode);

        3
    }

    fn plp(&mut self, mode: AddressingMode) -> usize {
        self.p = self.pop8();
        self.advance(mode);

        4
    }

    fn txs(&mut self, mode: AddressingMode) -> usize {
        self.sp = self.x;
        self.advance(mode);

        2
    }

    fn tsx(&mut self, mode: AddressingMode) -> usize {
        self.x = self.sp;
        self.advance(mode);

        2
    }

    // Flags
    fn clc(&mut self, mode: AddressingMode) -> usize {
        self.set_flag(CPUFlags::Carry, false);
        self.advance(mode);

        2
    }
    fn sec(&mut self, mode: AddressingMode) -> usize {
        self.set_flag(CPUFlags::Carry, true);
        self.advance(mode);

        2
    }

    fn cli(&mut self, mode: AddressingMode) -> usize {
        self.set_flag(CPUFlags::InterruptDisable, false);
        self.advance(mode);

        2
    }

    fn sei(&mut self, mode: AddressingMode) -> usize {
        self.set_flag(CPUFlags::InterruptDisable, true);
        self.advance(mode);

        2
    }

    fn cld(&mut self, mode: AddressingMode) -> usize  {
        self.set_flag(CPUFlags::Decimal, false);
        self.advance(mode);

        2
    }

    fn sed(&mut self, mode: AddressingMode) -> usize {
        self.set_flag(CPUFlags::Decimal, true);
        self.advance(mode);

        2
    }

    fn clv(&mut self, mode: AddressingMode) -> usize {
        self.set_flag(CPUFlags::Overflow, false);
        self.advance(mode);

        2
    }

    // Other
    fn nop(&mut self, mode: AddressingMode) -> usize {
        self.advance(mode);

        2
    }

    fn execute(&mut self, opcode: u8) -> usize {
        match opcode {
            0x69 => self.adc(AddressingMode::Immediate),
            0x65 => self.adc(AddressingMode::ZeroPage),
            0x75 => self.adc(AddressingMode::ZeroPageX),
            0x6D => self.adc(AddressingMode::Absolute),
            0x7D => self.adc(AddressingMode::AbsoluteX),
            0x79 => self.adc(AddressingMode::AbsoluteY),
            0x61 => self.adc(AddressingMode::IndexedIndirect),
            0x71 => self.adc(AddressingMode::IndirectIndexed),

            0x29 => self.and(AddressingMode::Immediate),
            0x25 => self.and(AddressingMode::ZeroPage),
            0x35 => self.and(AddressingMode::ZeroPageX),
            0x2D => self.and(AddressingMode::Absolute),
            0x3D => self.and(AddressingMode::AbsoluteX),
            0x39 => self.and(AddressingMode::AbsoluteY),
            0x21 => self.and(AddressingMode::IndexedIndirect),
            0x31 => self.and(AddressingMode::IndirectIndexed),

            0x0A => self.asl(AddressingMode::Accumulator),
            0x06 => self.asl(AddressingMode::ZeroPage),
            0x16 => self.asl(AddressingMode::ZeroPageX),
            0x0E => self.asl(AddressingMode::Absolute),
            0x1E => self.asl(AddressingMode::AbsoluteX),

            0x90 => self.bcc(AddressingMode::Relative),
            0xB0 => self.bcs(AddressingMode::Relative),
            0xF0 => self.beq(AddressingMode::Relative),
            0x30 => self.bmi(AddressingMode::Relative),
            0xD0 => self.bne(AddressingMode::Relative),
            0x10 => self.bpl(AddressingMode::Relative),
            0x50 => self.bvc(AddressingMode::Relative),
            0x70 => self.bvs(AddressingMode::Relative),

            0x24 => self.bit(AddressingMode::ZeroPage),
            0x2C => self.bit(AddressingMode::Absolute),

            0x00 => self.brk(AddressingMode::Immediate),

            0x18 => self.clc(AddressingMode::Implied),
            0xD8 => self.cld(AddressingMode::Implied),
            0x58 => self.cli(AddressingMode::Implied),
            0xB8 => self.clv(AddressingMode::Implied),

            0xC9 => self.cmp(AddressingMode::Immediate),
            0xC5 => self.cmp(AddressingMode::ZeroPage),
            0xD5 => self.cmp(AddressingMode::ZeroPageX),
            0xCD => self.cmp(AddressingMode::Absolute),
            0xDD => self.cmp(AddressingMode::AbsoluteX),
            0xD9 => self.cmp(AddressingMode::AbsoluteY),
            0xC1 => self.cmp(AddressingMode::IndexedIndirect),
            0xD1 => self.cmp(AddressingMode::IndirectIndexed),
            
            0xE0 => self.cpx(AddressingMode::Immediate),
            0xE4 => self.cpx(AddressingMode::ZeroPage),
            0xEC => self.cpx(AddressingMode::Absolute),

            0xC0 => self.cpy(AddressingMode::Immediate),
            0xC4 => self.cpy(AddressingMode::ZeroPage),
            0xCC => self.cpy(AddressingMode::Absolute),
            
            0xE6 => self.inc(AddressingMode::ZeroPage),
            0xF6 => self.inc(AddressingMode::ZeroPageX),
            0xEE => self.inc(AddressingMode::Absolute),
            0xFE => self.inc(AddressingMode::AbsoluteX),

            0xC6 => self.dec(AddressingMode::ZeroPage),
            0xD6 => self.dec(AddressingMode::ZeroPageX),
            0xCE => self.dec(AddressingMode::Absolute),
            0xDE => self.dec(AddressingMode::AbsoluteX),

            0xE8 => self.inx(AddressingMode::Implied),
            0xCA => self.dex(AddressingMode::Implied),

            0xC8 => self.iny(AddressingMode::Implied),
            0x88 => self.dey(AddressingMode::Implied),

            0x49 => self.eor(AddressingMode::Immediate),
            0x45 => self.eor(AddressingMode::ZeroPage),
            0x55 => self.eor(AddressingMode::ZeroPageX),
            0x4D => self.eor(AddressingMode::Absolute),
            0x5D => self.eor(AddressingMode::AbsoluteX),
            0x59 => self.eor(AddressingMode::AbsoluteY),
            0x41 => self.eor(AddressingMode::IndexedIndirect),
            0x51 => self.eor(AddressingMode::IndirectIndexed),

            0x4C => self.jmp(AddressingMode::Absolute),
            0x6C => self.jmp(AddressingMode::Indirect),
            
            0x20 => self.jsr(AddressingMode::Absolute),

            0xA9 => self.lda(AddressingMode::Immediate),
            0xA5 => self.lda(AddressingMode::ZeroPage),
            0xB5 => self.lda(AddressingMode::ZeroPageX),
            0xAD => self.lda(AddressingMode::Absolute),
            0xBD => self.lda(AddressingMode::AbsoluteX),
            0xB9 => self.lda(AddressingMode::AbsoluteY),
            0xA1 => self.lda(AddressingMode::IndexedIndirect),
            0xB1 => self.lda(AddressingMode::IndirectIndexed),

            0xA2 => self.ldx(AddressingMode::Immediate),
            0xA6 => self.ldx(AddressingMode::ZeroPage),
            0xB6 => self.ldx(AddressingMode::ZeroPageY),
            0xAE => self.ldx(AddressingMode::Absolute),
            0xBE => self.ldx(AddressingMode::AbsoluteY),
            
            0xA0 => self.ldy(AddressingMode::Immediate),
            0xA4 => self.ldy(AddressingMode::ZeroPage),
            0xB4 => self.ldy(AddressingMode::ZeroPageX),
            0xAC => self.ldy(AddressingMode::Absolute),
            0xBC => self.ldy(AddressingMode::AbsoluteX),

            0x4A => self.lsr(AddressingMode::Accumulator),
            0x46 => self.lsr(AddressingMode::ZeroPage),
            0x56 => self.lsr(AddressingMode::ZeroPageX),
            0x4E => self.lsr(AddressingMode::Absolute),
            0x5E => self.lsr(AddressingMode::AbsoluteX),

            0xEA => self.nop(AddressingMode::Implied),

            0x09 => self.ora(AddressingMode::Immediate),
            0x05 => self.ora(AddressingMode::ZeroPage),
            0x15 => self.ora(AddressingMode::ZeroPageX),
            0x0D => self.ora(AddressingMode::Absolute),
            0x1D => self.ora(AddressingMode::AbsoluteX),
            0x19 => self.ora(AddressingMode::AbsoluteY),
            0x01 => self.ora(AddressingMode::IndexedIndirect),
            0x11 => self.ora(AddressingMode::IndirectIndexed),

            0x48 => self.pha(AddressingMode::Implied),
            0x08 => self.php(AddressingMode::Implied),
            0x68 => self.pla(AddressingMode::Implied),
            0x28 => self.plp(AddressingMode::Implied),

            0x2A => self.rol(AddressingMode::Accumulator),
            0x26 => self.rol(AddressingMode::ZeroPage),
            0x36 => self.rol(AddressingMode::ZeroPageX),
            0x2E => self.rol(AddressingMode::Absolute),
            0x3E => self.rol(AddressingMode::AbsoluteX),
            
            0x6A => self.ror(AddressingMode::Accumulator),
            0x66 => self.ror(AddressingMode::ZeroPage),
            0x76 => self.ror(AddressingMode::ZeroPageX),
            0x6E => self.ror(AddressingMode::Absolute),
            0x7E => self.ror(AddressingMode::AbsoluteX),

            0x40 => self.rti(AddressingMode::Implied),

            0x60 => self.rts(AddressingMode::Implied),

            0xE9 => self.sbc(AddressingMode::Immediate),
            0xE5 => self.sbc(AddressingMode::ZeroPage),
            0xF5 => self.sbc(AddressingMode::ZeroPageX),
            0xED => self.sbc(AddressingMode::Absolute),
            0xFD => self.sbc(AddressingMode::AbsoluteX),
            0xF9 => self.sbc(AddressingMode::AbsoluteY),
            0xE1 => self.sbc(AddressingMode::IndexedIndirect),
            0xF1 => self.sbc(AddressingMode::IndirectIndexed),

            0x38 => self.sec(AddressingMode::Implied),
            0xF8 => self.sed(AddressingMode::Implied),
            0x78 => self.sei(AddressingMode::Implied),

            0x85 => self.sta(AddressingMode::ZeroPage),
            0x95 => self.sta(AddressingMode::ZeroPageX),
            0x8D => self.sta(AddressingMode::Absolute),
            0x9D => self.sta(AddressingMode::AbsoluteX),
            0x99 => self.sta(AddressingMode::AbsoluteY),
            0x81 => self.sta(AddressingMode::IndexedIndirect),
            0x91 => self.sta(AddressingMode::IndirectIndexed),
            
            0x86 => self.stx(AddressingMode::ZeroPage),
            0x96 => self.stx(AddressingMode::ZeroPageY),
            0x8E => self.stx(AddressingMode::Absolute),
            
            0x84 => self.sty(AddressingMode::ZeroPage),
            0x94 => self.sty(AddressingMode::ZeroPageX),
            0x8C => self.sty(AddressingMode::Absolute),

            0xAA => self.tax(AddressingMode::Implied),

            0xA8 => self.tay(AddressingMode::Implied),

            0xBA => self.tsx(AddressingMode::Implied),
            
            0x8A => self.txa(AddressingMode::Implied),

            0x9A => self.txs(AddressingMode::Implied),

            0x98 => self.tya(AddressingMode::Implied),

            _ => panic!("Unknown instruction!")
        }
    }

    pub fn step(&mut self) -> usize {
        self.execute(self.ram[self.pc as usize])
    }
}