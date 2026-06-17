use crate::mapper::Mapper;

pub struct NROM {
    pub is_32k:     bool,

    pub prg_rom:    Vec<u8>,
    pub chr_rom:    Vec<u8>,
    pub vram:       Box<[u8; 0x0800]>
}

impl NROM {
    pub fn new(prg: Vec<u8>, chr: Vec<u8>) -> NROM {
        NROM
        {
            is_32k:     prg.len() > 0x4000,
            prg_rom:    prg,
            chr_rom:    chr,
            vram:       Box::new([0; 0x0800]),
        }
    }
}

impl Mapper for NROM {
    fn prg_read(&mut self, mut addr: u16) -> u8 {
        if !self.is_32k {
            addr %= 0x4000;
        }

        self.prg_rom[addr as usize]
    }

    fn chr_read(&mut self, addr: u16) -> u8 {
        self.chr_rom[addr as usize]
    }
    
    fn vram_read(&mut self, addr: u16) -> u8 {
        self.vram[addr as usize]
    }
    
    fn vram_write(&mut self, addr: u16, value: u8) {
        self.vram[addr as usize] = value;
    }
}