use crate::{mapper::{Mapper, NametableMirroring}, mappers::{fixed_mirroring, mirrored_address}};

pub struct UNROM {
    pub mirroring:  NametableMirroring,

    pub prg_rom:    Vec<u8>,
    pub chr_rom:    Vec<u8>,

    pub vram:       Box<[u8; 0x0800]>,

    pub bank:       usize
}

impl UNROM {
    pub fn new(prg: Vec<u8>, chr: Vec<u8>, _prg_ram: Option<Vec<u8>>, header: &[u8; 0x10]) -> UNROM {
        UNROM
        {
            mirroring:  fixed_mirroring(header[6]),

            prg_rom:    prg,
            chr_rom:    chr,

            vram:       Box::new([0; 0x0800]),
            bank:       0
        }
    }
}

impl Mapper for UNROM {
    fn prg_read(&mut self, addr: u16) -> u8 {
        if addr < 0x4000 {
            self.prg_rom[addr as usize + self.bank * 0x4000]
        }
        else {
            self.prg_rom[addr as usize + self.prg_rom.len() - 0x8000]
        }
    }

    fn prg_write(&mut self, _addr: u16, value: u8) {
        self.bank = value as usize;
    }

    fn chr_read(&mut self, addr: u16) -> u8 {
        self.chr_rom[addr as usize]
    }
    
    fn chr_write(&mut self, addr: u16, value: u8) {
        self.chr_rom[addr as usize] = value;
    }
    

    fn vram_read(&mut self, addr: u16) -> u8 {
        self.vram[mirrored_address(self.mirroring, addr % 0x1000) as usize % 0x0800]
    }
    
    fn vram_write(&mut self, addr: u16, value: u8) {
        self.vram[mirrored_address(self.mirroring, addr % 0x1000) as usize % 0x0800] = value;
    }
}