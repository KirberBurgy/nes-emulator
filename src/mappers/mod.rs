pub mod nrom;
pub use nrom::*;

pub mod mmc1;
pub use mmc1::*;

pub mod unrom;
pub use unrom::*;


pub mod mmc3;
pub use mmc3::*;

use crate::{bit_utils::bit_set, mapper::{Mapper, NametableMirroring}};

pub fn fixed_mirroring(header_byte_6: u8) -> NametableMirroring {
    if bit_set(header_byte_6, 0) { NametableMirroring::Vertical }
    else { NametableMirroring::Horizontal }
}

fn mirrored_address(mode: NametableMirroring, addr: u16) -> u16 {
    match mode {
        NametableMirroring::Horizontal  =>
            if addr < 0x0800 { addr % 0x0400 }
            else  { (addr % 0x0400) + 0x0400 }

        NametableMirroring::Vertical    => addr % 0x0800,

        NametableMirroring::FourScreen  => addr,
    }
}

pub fn new_mapper(prg: Vec<u8>, chr: Vec<u8>, prg_ram: Option<Vec<u8>>, header: &[u8; 0x10], mapper_number: u8) -> Box<dyn Mapper> {
    match mapper_number {
        0 => Box::new(NROM::new(prg, chr, prg_ram, header)),
        1 => Box::new(MMC1::new(prg, chr, prg_ram, header)),
        2 => Box::new(UNROM::new(prg, chr, prg_ram, header)),
        4 => Box::new(MMC3::new(prg, chr, prg_ram, header)),

        _ => unimplemented!()
    }
}