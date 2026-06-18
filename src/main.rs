use crate::{cartridge::Cartridge, cpu::CPU, memory_bus::MemoryBus, nes::NES};

pub mod bit_utils;

pub mod memory_bus;

pub mod cpu;
pub mod instructions;

pub mod ppu;

pub mod mapper;
pub mod mappers;

pub mod cartridge;

pub mod nes;

fn main() {
    let donkey_kong = Cartridge::load("tests/roms/Donkey Kong.nes").unwrap();
    let mut nes = NES::new(donkey_kong);
}
