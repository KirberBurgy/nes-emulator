use crate::{bit_utils::bit_set, cpu::CPU};

pub mod bit_utils;

pub mod cpu;

pub mod instructions;

fn main() {
    let mut cpu = CPU::new();
    
    let bytecode = [
        // INX
        // TXA
        // DEX
        // LDA $0x69
        // BRK
        0xE8, 0x8A, 0xCA, 0xA9, 0x69, 0x00
    ];

    for (i, byte) in bytecode.iter().enumerate() {
        cpu.ram[i] = *byte;
    }

    while !cpu.flag_set(cpu::CPUFlags::InterruptDisable) {
        cpu.dbg();
        cpu.step();

        println!();
    }

    cpu.dbg();
}
