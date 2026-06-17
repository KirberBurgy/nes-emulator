use crate::{cartridge::Cartridge, cpu::CPU, memory_bus::MemoryBus};

pub mod bit_utils;

pub mod memory_bus;

pub mod cpu;
pub mod instructions;

pub mod ppu;

pub mod mapper;
pub mod mappers;

pub mod cartridge;

fn main() {
    let donkey_kong = Cartridge::load("tests/roms/Donkey Kong.nes").unwrap();
    let mut memory_bus = MemoryBus::new(donkey_kong);
    let mut cpu = CPU::new();

    cpu.jump_to_startup(&mut memory_bus);
    cpu.debug();
    println!();

    for i in 0..12 {
        cpu.step(&mut memory_bus);
        cpu.debug();
        println!();
    }

    cpu.debug();
    println!();
}
