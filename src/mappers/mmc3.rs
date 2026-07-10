use crate::{bit_utils::{bit_set, get_bits}, mapper::{Mapper, NametableMirroring}, mappers::mirrored_address};

pub struct MMC3 {
    pub prg_ram:            Vec<u8>,

    pub prg_ram_enabled:    bool,
    pub prg_ram_protected:  bool,

    pub prg_rom:            Vec<u8>,
    pub chr_rom:            Vec<u8>,

    pub vram:               Box<[u8; 0x0800]>,
    
    pub next_write_dest:    u8,
    pub mirroring:          NametableMirroring,

    pub prg_mode:           bool,
    pub chr_mode:           bool,

    pub prg_8kb_1:          usize,
    pub prg_8kb_2:          usize,

    pub chr_2kb_1:          usize,
    pub chr_2kb_2:          usize,

    pub chr_1kb_1:          usize,
    pub chr_1kb_2:          usize,
    pub chr_1kb_3:          usize,
    pub chr_1kb_4:          usize,

    pub irq_enabled:        bool,
    pub irq_reload:         bool,
    pub irq_pending:        bool,
    pub irq_reset:          u8,
    pub irq_counter:        u8,

    pub num_low_clocks:      usize
}

impl MMC3 {
    pub fn new(prg: Vec<u8>, chr: Vec<u8>, prg_ram: Option<Vec<u8>>, header: &[u8; 0x10]) -> MMC3 {
        MMC3
        {
            prg_ram:            if let Some(ram) = prg_ram { ram } else { vec![0; 0x2000] },

            prg_ram_enabled:    false,
            prg_ram_protected:  false,
            
            prg_rom:            prg,
            chr_rom:            chr,

            vram:               Box::new([0; 0x0800]),

            next_write_dest:    0,
            mirroring:          
                if bit_set(header[6], 3)    { NametableMirroring::FourScreen } 
                //                            PLACEHOLDER
                else                        { NametableMirroring::Horizontal },

            prg_mode:           false,
            chr_mode:           false,

            prg_8kb_1:          0,
            prg_8kb_2:          0,

            chr_2kb_1:          0,
            chr_2kb_2:          0,

            chr_1kb_1:          0,
            chr_1kb_2:          0,
            chr_1kb_3:          0,
            chr_1kb_4:          0,

            irq_enabled:        false,
            irq_reload:         false,
            irq_pending:        false,
            irq_reset:          0,
            irq_counter:        0,

            num_low_clocks:      0,
        }
    }

    fn prg_effective_address(&self, addr: u16) -> usize {
        let last_bank_loc = self.prg_rom.len() - 0x2000;
        let second_last_bank_loc = last_bank_loc - 0x2000;

        if self.prg_mode {
            match addr {
                0x0000..0x2000 => (addr as usize)           + second_last_bank_loc,
                0x2000..0x4000 => (addr as usize - 0x2000)  + self.prg_8kb_2 * 0x2000,
                0x4000..0x6000 => (addr as usize - 0x4000)  + self.prg_8kb_1 * 0x2000,
                0x6000..0x8000 => (addr as usize - 0x6000)  + last_bank_loc,

                _ => unreachable!()
            }
        }
        else {
            match addr {
                0x0000..0x2000 => (addr as usize)           + self.prg_8kb_1 * 0x2000,
                0x2000..0x4000 => (addr as usize - 0x2000)  + self.prg_8kb_2 * 0x2000,
                0x4000..0x8000 => (addr as usize - 0x4000)  + second_last_bank_loc,

                _ => unreachable!()
            }
        }
    }

    fn chr_effective_address(&self, addr: u16) -> usize {
        if self.chr_mode {
            match addr {
                0x0000..0x0400 => (addr as usize)           + self.chr_1kb_1 * 0x0400,
                0x0400..0x0800 => (addr as usize - 0x0400)  + self.chr_1kb_2 * 0x0400,
                0x0800..0x0C00 => (addr as usize - 0x0800)  + self.chr_1kb_3 * 0x0400,
                0x0C00..0x1000 => (addr as usize - 0x0C00)  + self.chr_1kb_4 * 0x0400,
                
                0x1000..0x1800 => (addr as usize - 0x1000)  + self.chr_2kb_1 * 0x0800,
                0x1800..0x2000 => (addr as usize - 0x1800)  + self.chr_2kb_2 * 0x0800,

                _ => unreachable!()
            }
        }
        else {
            match addr {
                0x0000..0x0800 => (addr as usize)           + self.chr_2kb_1 * 0x0800,
                0x0800..0x1000 => (addr as usize - 0x0800)  + self.chr_2kb_2 * 0x0800,

                0x1000..0x1400 => (addr as usize - 0x1000)  + self.chr_1kb_1 * 0x0400,
                0x1400..0x1800 => (addr as usize - 0x1400)  + self.chr_1kb_2 * 0x0400,
                0x1800..0x1C00 => (addr as usize - 0x1800)  + self.chr_1kb_3 * 0x0400,
                0x1C00..0x2000 => (addr as usize - 0x1C00)  + self.chr_1kb_4 * 0x0400,

                _ => unreachable!()
            }
        }
    }

    fn write_bank(&mut self, value: u8) {
        match self.next_write_dest {
            0 => self.chr_2kb_1 = get_bits(value, 1..8) as usize,
            1 => self.chr_2kb_2 = get_bits(value, 1..8) as usize,
            2 => self.chr_1kb_1 = value as usize,
            3 => self.chr_1kb_2 = value as usize,
            4 => self.chr_1kb_3 = value as usize,
            5 => self.chr_1kb_4 = value as usize,
            6 => self.prg_8kb_1 = value as usize,
            7 => self.prg_8kb_2 = value as usize,

            _ => unreachable!()
        }
    }

    fn clock_irq(&mut self) {
        if self.irq_counter == 0 || self.irq_reload {
            self.irq_counter    = self.irq_reset;
        } 
        else {
            self.irq_counter    -= 1;
        }

        self.irq_reload     = false;
        self.irq_pending    = self.irq_pending || (self.irq_counter == 0 && self.irq_enabled);
    }

    fn update_a12(&mut self, addr: u16) {
        if bit_set(addr, 12) {
            if self.num_low_clocks >= 1 {
                self.clock_irq();
            }

            self.num_low_clocks = 0;
        } 
        else {
            self.num_low_clocks += 1;
        }
    }
}

impl Mapper for MMC3 {
    fn prg_ram_read(&mut self, addr: u16) -> u8 {
        if self.prg_ram_enabled { self.prg_ram[addr as usize] }
        else { 0 }
    }

    fn prg_ram_write(&mut self, addr: u16, value: u8) {
        if self.prg_ram_enabled && !self.prg_ram_protected { self.prg_ram[addr as usize] = value; }
    }

    fn prg_read(&mut self, addr: u16) -> u8 {
        let effective_address = self.prg_effective_address(addr);

        self.prg_rom[effective_address]
    }

    fn prg_write(&mut self, addr: u16, value: u8) {
        let is_even = (addr & 1) == 0;

        match addr {
            0x0000..0x2000 =>
                if is_even { 
                    self.next_write_dest    = get_bits(value, 0..3);
                    self.prg_mode           = bit_set(value, 6);
                    self.chr_mode           = bit_set(value, 7);
                }
                else {
                    self.write_bank(value);
                }

            0x2000..0x4000 =>
                if is_even {
                    if !matches!(self.mirroring, NametableMirroring::FourScreen) {
                        self.mirroring = 
                            if !bit_set(value, 0) { NametableMirroring::Vertical } 
                            else { NametableMirroring::Horizontal };
                    }
                }
                else {
                    self.prg_ram_protected  = bit_set(value, 6);
                    self.prg_ram_enabled    = bit_set(value, 7);
                }

            0x4000..0x6000 => 
                if is_even {
                    self.irq_reset      = value;
                }
                else {
                    self.irq_counter    = 0;
                    self.irq_reload     = true;
                }

            0x6000..0x8000 => {
                self.irq_enabled = !is_even;
                // self.irq_pending = self.irq_pending && self.irq_enabled;
            }

            _ => unreachable!()
        }
    }

    fn chr_read(&mut self, addr: u16) -> u8 {
        self.update_a12(addr);
        let effective_address = self.chr_effective_address(addr);

        self.chr_rom[effective_address]
    }

    fn chr_write(&mut self, addr: u16, value: u8) {
        self.update_a12(addr);
        let effective_address = self.chr_effective_address(addr);

        self.chr_rom[effective_address] = value;

        // (Most) MMC3 games don't use CHR RAM, so this is pretty useless.
    }

    fn vram_read(&mut self, addr: u16) -> u8 {
        self.update_a12(0x2000 | addr);

        let address = mirrored_address(self.mirroring, addr) as usize;
        self.vram[address % 0x0800]
    }

    fn vram_write(&mut self, addr: u16, value: u8) {
        self.update_a12(0x2000 | addr);

        let address = mirrored_address(self.mirroring, addr) as usize;
        self.vram[address % 0x0800] = value;
    }

    fn signaling_irq(&self) -> bool {
        self.irq_pending
    }

    fn irq_acknowledged(&mut self) {
        self.irq_pending = false;
    }
}