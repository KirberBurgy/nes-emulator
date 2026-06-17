pub mod nrom;
pub use nrom::*;

use crate::mapper::Mapper;

pub fn new_mapper(prg: Vec<u8>, chr: Vec<u8>, mapper_number: u8) -> Box<dyn Mapper> {
    match mapper_number {
        0 => Box::new(NROM::new(prg, chr)),

        _ => unimplemented!()
    }
}