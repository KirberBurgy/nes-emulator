use crate::{cartridge::Cartridge};

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
}
