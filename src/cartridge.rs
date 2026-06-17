use std::{fs::File, io::{BufReader, Read}};

use crate::{bit_utils::{bit_set, get_bits}, mapper::{Mapper, NametableMirroring}, mappers::new_mapper};

pub struct Cartridge {
    pub mirroring:  NametableMirroring,
    pub mapper:     Box<dyn Mapper>,
}

impl Cartridge {
    pub fn load(name: &str) -> Option<Cartridge> {
        let file = File::open(name).unwrap();
        let mut reader = BufReader::new(file);

        let mut header = [0u8; 16];
        reader.read_exact(&mut header).unwrap();

        let nes_header = *b"NES\x1A";

        let is_nes = header[0..=3] == nes_header;
        if !is_nes {
            return None;
        }

        let is_nes20 = !bit_set(header[7], 2) && bit_set(header[7], 3);

        if is_nes20 {
            println!("Loading NES 2.0 image...");

            Self::load_nes20(&mut reader, header)
        }
        else {
            println!("Loading iNES image...");

            Self::load_ines(&mut reader, header)
        }
    }

    fn load_ines(reader: &mut BufReader<File>, header: [u8; 16]) -> Option<Cartridge> {
        let prg_rom_size = 0x4000 * header[4] as usize;
        let chr_rom_size = 0x2000 * header[5] as usize;

        println!("Program ROM size: {}", prg_rom_size);
        println!("Character ROM size: {}", chr_rom_size);

        let mapper_number = {
            let lsn = get_bits(header[6], 4..8);
            let msn = get_bits(header[7], 4..8);

            lsn | (msn << 4)
        };

        println!("Using mapper #{}.", mapper_number);

        let mirroring = 
            if bit_set(header[6], 0) { NametableMirroring::Vertical }
            else { NametableMirroring::Horizontal };

        println!("Using nametable mirroring '{:?}'", mirroring);
        
        let mut prg_rom = Vec::with_capacity(prg_rom_size);
        reader.read_exact(&mut prg_rom).unwrap();

        let mut chr_rom = Vec::with_capacity(chr_rom_size);
        reader.read_exact(&mut chr_rom).unwrap();

        let mapper = new_mapper(prg_rom, chr_rom, mapper_number);

        Some(Cartridge { mirroring, mapper })
    }

    fn load_nes20(reader: &mut BufReader<File>, header: [u8; 16]) -> Option<Cartridge> {
        // Right now there are no differences.
        Self::load_ines(reader, header)
    }
}