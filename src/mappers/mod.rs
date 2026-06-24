pub mod nrom;
pub use nrom::*;

pub mod mmc1;
pub use mmc1::*;

use crate::mapper::{Mapper, NametableMirroring};

fn mirrored_address(mode: NametableMirroring, addr: u16) -> u16 {
    match mode {
        NametableMirroring::Horizontal  => {
            match addr {
                0x0400..0x0C00 => addr - 0x0400,
                0x0C00..0x1000 => addr - 0x0800,

                _ => addr
            }
        }

        NametableMirroring::Vertical    => {
            addr % 0x0800
        }

        NametableMirroring::FourScreen  => addr,
    }
}

pub fn new_mapper(prg: Vec<u8>, chr: Vec<u8>, header: &[u8; 0x10], mapper_number: u8) -> Box<dyn Mapper> {
    match mapper_number {
        0 => Box::new(NROM::new(prg, chr, header)),
        1 => Box::new(MMC1::new(prg, chr, header)),

        _ => unimplemented!()
    }
}