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

    let old = memory_bus.ppu.odd_frame;
    let mut cycles = 0;

    while old == memory_bus.ppu.odd_frame {
        memory_bus.ppu.tick();
        cycles += 1;
    }
    
    println!("{}", cycles);
}
