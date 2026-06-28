use std::{fs::File, io::{BufReader, Read, Write}, path::Path};

use crate::{bit_utils::{bit_set, get_bits}, mapper::{Mapper}, mappers::new_mapper};

pub struct Cartridge {
    pub name:       String,
    pub mapper:     Box<dyn Mapper>
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

            Self::load_nes20(name, &mut reader, header)
        }
        else {
            println!("Loading iNES image...");

            Self::load_ines(name, &mut reader, header)
        }
    }

    fn load_ines(name: &str, reader: &mut BufReader<File>, header: [u8; 16]) -> Option<Cartridge> {
        let prg_rom_size = 0x4000 * header[4] as usize;
        let chr_rom_size = 0x2000 * header[5] as usize;
        let prg_ram_size = 0x2000 * header[8] as usize;

        println!("Program ROM size: {}", prg_rom_size);
        println!("Character ROM size: {}", chr_rom_size);

        let mapper_number = {
            let lsn = get_bits(header[6], 4..8);
            let msn = get_bits(header[7], 4..8);

            lsn | (msn << 4)
        };

        if bit_set(header[6], 2) {
            println!("Cartridge contains a trainer.");
        }
        
        println!("Using mapper #{}.", mapper_number);

        let mut prg_rom = Vec::with_capacity(prg_rom_size);
        prg_rom.resize(prg_rom_size, 0);
        reader.read_exact(&mut prg_rom).unwrap();

        let chr = if chr_rom_size != 0 {
            let mut chr_rom = Vec::with_capacity(chr_rom_size);
            chr_rom.resize(chr_rom_size, 0);
            reader.read_exact(&mut chr_rom).unwrap();

            chr_rom
        }
        else { println!("Cartridge uses CHR RAM."); vec![0; 0x2000] };


        let prg_ram = if bit_set(header[6], 1) {
            let save_file_path = Path::new(name)
                .with_extension("sram");

            if let Ok(mut save_file) = File::open(save_file_path) {
                println!("Save file identified.");
                
                let mut sram = Vec::new();
                save_file.read_to_end(&mut sram).unwrap();

                Some(sram)
            }
            else {
                None
            }
        }
        else if prg_ram_size > 0 {
            println!("Cartridge uses non-battery-backed PRG RAM.");

            Some(vec![0; prg_ram_size])
        }
        else {
            None
        };

        let mapper = new_mapper(prg_rom, chr, prg_ram, &header, mapper_number);

        Some(Cartridge { name: name.to_owned(), mapper })
    }

    fn load_nes20(name: &str, reader: &mut BufReader<File>, header: [u8; 16]) -> Option<Cartridge> {
        // Right now there are no differences.
        Self::load_ines(name, reader, header)
    }

    pub fn prg_ram_read(&mut self, addr: u16) -> u8 {
        self.mapper.prg_ram_read(addr - 0x6000)
    }

    // Assumes `addr` is in the range 0x8000-0xFFFF
    pub fn prg_read(&mut self, addr: u16) -> u8 {
        self.mapper.prg_read(addr - 0x8000)
    }

    // Assumes `addr` is in the range 0x0000-0x1FFF
    pub fn chr_read(&mut self, addr: u16) -> u8 {
        self.mapper.chr_read(addr)
    }

    // Assumes `addr` is in the range 0x2000-0x3FFF
    pub fn vram_read(&mut self, addr: u16) -> u8 {
        self.mapper.vram_read(addr - 0x2000)
    }

    pub fn prg_ram_write(&mut self, addr: u16, value: u8) {
        self.mapper.prg_ram_write(addr - 0x6000, value)
    }

    pub fn prg_write(&mut self, addr: u16, value: u8) {
        self.mapper.prg_write(addr - 0x8000, value);
    }

    pub fn chr_write(&mut self, addr: u16, value: u8) {
        self.mapper.chr_write(addr, value);
    }

    pub fn vram_write(&mut self, addr: u16, value: u8) {
        self.mapper.vram_write(addr - 0x2000, value);
    }

    pub fn save_sram(&mut self) {
        let mut file = File::create(Path::new(&self.name).with_extension("sram")).unwrap();

        let mut ram = [0; 0x2000];

        for i in 0x0000..0x2000 {
            ram[i] = self.mapper.prg_ram_read(i as u16);
        }

        file.write_all(&mut ram).unwrap();
    }
}