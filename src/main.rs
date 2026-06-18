use crate::{cartridge::Cartridge, nes::NES};

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
    unsafe { std::env::set_var("RUST_BACKTRACE", "1"); }
    let donkey_kong = Cartridge::load("tests/roms/Donkey Kong.nes").unwrap();
    let nes = NES::new(donkey_kong);
}
