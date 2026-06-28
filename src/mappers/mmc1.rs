use crate::{bit_utils::{bit_set, get_bits, set_bit}, mapper::{Mapper, NametableMirroring}, mappers::mirrored_address};

pub struct MMC1 {
    pub prg_ram:    Vec<u8>,

    pub prg_rom:    Vec<u8>,
    pub chr_rom:    Vec<u8>,

    pub vram:       Box<[u8; 0x0800]>,
    
    pub shift:      u8,
    pub writes:     u8,

    pub control:    u8,

    pub prg_bank:   usize,

    pub chr_bank_1: usize,
    pub chr_bank_2: usize,
}

impl MMC1 {
    pub fn new(prg: Vec<u8>, chr: Vec<u8>, prg_ram: Option<Vec<u8>>, _header: &[u8; 0x10]) -> MMC1 {
        MMC1
        {
            prg_ram:    if let Some(ram) = prg_ram { ram } else { vec![0; 0x2000] },
            
            prg_rom:    prg,
            chr_rom:    chr,

            vram:       Box::new([0; 0x0800]),

            shift:      0,
            writes:     0,

            control:    0b01100,

            prg_bank:   0,

            chr_bank_1: 0,
            chr_bank_2: 0,
        }
    }

    fn mirrored_addr(&self, addr: u16) -> usize {
        match get_bits(self.control, 0..2) {
            0 => (addr % 0x0400) as usize,
            1 => (addr % 0x0400) as usize + 0x0400,
            2 => mirrored_address(NametableMirroring::Vertical, addr) as usize,
            3 => mirrored_address(NametableMirroring::Horizontal, addr) as usize,

            _ => 0
        }
    }
}

impl Mapper for MMC1 {
    fn prg_ram_read(&mut self, addr: u16) -> u8 {
        self.prg_ram[addr as usize]
    }

    fn prg_ram_write(&mut self, addr: u16, value: u8) {
        self.prg_ram[addr as usize] = value;
    }

    fn prg_read(&mut self, addr: u16) -> u8 {
        let mode = get_bits(self.control, 2..4);

        match mode {
            0 | 1   => self.prg_rom[addr as usize + self.prg_bank * 0x8000],
            2       => {
                if addr < 0x4000 { self.prg_rom[addr as usize] }
                else { self.prg_rom[(addr - 0x4000) as usize + self.prg_bank * 0x4000] }
            },
            3       => {
                if addr < 0x4000 { self.prg_rom[addr as usize + self.prg_bank * 0x4000] }
                // Truncated calculation of `addr - 0x4000` as usize + `self.prg_rom.len() - 0x4000`
                // which would achieve the same effect.
                // Note that prg_rom.len() - 0x4000 is the start address last bank.
                else             { self.prg_rom[addr as usize + (self.prg_rom.len() - 0x8000)]}
            },

            _ => 0
        }
    }

    fn prg_write(&mut self, addr: u16, value: u8) {
        if bit_set(value, 7) {
            self.writes = 0;
            self.shift = 0;
            
            return;
        }

        self.writes += 1;

        self.shift >>= 1;
        self.shift = set_bit(self.shift, 4, bit_set(value, 0));

        if self.writes < 5 {
            return;
        }


        // Addresses are normalized through `addr - 0x8000` so 
        // these ranges are changed to reflect such.
        match addr {
            0x0000..0x2000  => self.control = self.shift,
            0x2000..0x4000  => {
                if bit_set(self.control, 4) { 
                    self.chr_bank_1 = self.shift as usize;
                }
                else {
                    self.chr_bank_1 = get_bits(self.shift, 1..5) as usize;
                }
            }
            0x4000..0x6000  => {
                if bit_set(self.control, 4) {
                    self.chr_bank_2 = self.shift as usize;
                }
            }
            0x6000..        => {
                let mode = get_bits(self.control, 2..4);

                match mode {
                    0 | 1   => self.prg_bank = get_bits(self.shift, 1..5) as usize,
                    _       => self.prg_bank = self.shift as usize
                }
            }
        }

        self.writes = 0;
        self.shift = 0;
    }

    fn chr_read(&mut self, addr: u16) -> u8 {
        if bit_set(self.control, 4) {
            if addr < 0x1000 { self.chr_rom[addr as usize + self.chr_bank_1 * 0x1000] }
            else { self.chr_rom[(addr - 0x1000) as usize  + self.chr_bank_2 * 0x1000] }
        }
        else {
            self.chr_rom[addr as usize + self.chr_bank_1 * 0x2000]
        }
    }

    // Some MMC1 images use CHR RAM.
    fn chr_write(&mut self, addr: u16, value: u8) {
        if bit_set(self.control, 4) {
            if addr < 0x1000 { self.chr_rom[addr as usize + self.chr_bank_1 * 0x1000] = value; }
            else { self.chr_rom[(addr - 0x1000) as usize  + self.chr_bank_2 * 0x1000] = value; }
        }
        else {
            self.chr_rom[addr as usize + self.chr_bank_1 * 0x2000] = value;
        }
    }
    
    fn vram_read(&mut self, addr: u16) -> u8 {
        self.vram[self.mirrored_addr(addr % 0x1000)]
    }
    
    fn vram_write(&mut self, addr: u16, value: u8) {
        self.vram[self.mirrored_addr(addr % 0x1000)] = value;
    }
}
