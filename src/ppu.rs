use crate::bit_utils::{bit_set, get_bits, set_bit, set_bits};


#[repr(usize)]
pub enum PPUControlFlags {
    X9thBit                 = 0,
    Y9thBit                 = 1,
    VRAMIncrement           = 2,
    SpriteTableAddress      = 3,
    BgTableAddress          = 4,
    SpriteSize              = 5,
    MasterSlave             = 6,
    NMIEnabled              = 7
}

#[repr(usize)]
pub enum PPUMaskFlags {
    Greyscale           = 0,
    ShowBorderBg        = 1,
    ShowBorderSprites   = 2,
    EnableBg            = 3,
    EnableSprites       = 4,
    EmphasizeRed        = 5,
    EmphasizeGreen      = 6,
    EmphasizeBlue       = 7
}


#[repr(usize)]
pub enum PPUStatusFlags {
    SpriteOverflow  = 5,
    Sprite0Hit      = 6,
    VBlank          = 7
}

pub struct PPU {    
    pub control:    u8,
    pub mask:       u8,
    pub status:     u8,

    pub v:          u16,
    pub t:          u16,
    pub x:          u8,
    pub w:          bool,

    pub buffer:     u8,

    pub oam_addr:   u8,
    pub oam:        [u8; 0x100],

    pub vram:       Box<[u8; 0x0800]>
}

impl PPU {
    pub fn new() -> PPU {
        PPU
        {
            control:    0,
            mask:       0,
            status:     0,
            v:          0,
            t:          0,
            x:          0,
            w:          false,
            buffer:     0,
            oam_addr:   0,
            oam:        [0; 0x100],
            vram:       Box::new([0; 0x0800])
        }
    }

    fn increment(&self) -> u16 {
        if bit_set(self.control, PPUControlFlags::VRAMIncrement as usize) {
            32
        }
        else { 
            1 
        }
    }

    pub fn ppustatus_read(&mut self) -> u8 {
        self.w = false;
        
        let result = self.status;
        self.status = set_bit(self.status, PPUStatusFlags::VBlank as usize, false);

        result
    }

    pub fn ppuctrl_write(&mut self, to: u8) {
        self.control = to;
        self.t = set_bits(self.t, 10..12, get_bits(to, 0..2) as u16);
    }

    pub fn ppumask_write(&mut self, to: u8) {
        self.mask = to;
    }


    pub fn ppuscroll_write(&mut self, to: u8) {
        if self.w {
            self.t = set_bits(self.t, 12..15, get_bits(to, 0..2) as u16);
            self.t = set_bits(self.t, 5..10,  get_bits(to, 3..8) as u16);

            self.w = false;
        }
        else {
            self.x = get_bits(to, 0..2);
            self.t = set_bits(self.t, 0..5, get_bits(to, 3..8) as u16);

            self.w = true;
        }
    }

    pub fn ppuaddr_write(&mut self, to: u8) {
        if self.w {
            self.t = set_bits(self.t, 0..8, get_bits(to, 0..8) as u16);
            self.v = self.t;

            self.w = false;
        }
        else {
            self.t = set_bits(self.t, 8..14, get_bits(to, 0..6) as u16);
            self.t = set_bit(self.t, 15, false);

            self.w = true;
        }
    }

    // TO DO: Should be memory mapped.
    pub fn ppudata_read(&mut self) -> u8 {
        let byte = self.buffer;
        self.buffer = self.vram[self.v as usize];

        self.v = self.v.wrapping_add(self.increment());

        byte
    }

    pub fn ppudata_write(&mut self, to: u8) {
        self.vram[self.v as usize] = to;

        self.v = self.v.wrapping_add(self.increment());
    }

    pub fn oamaddr_write(&mut self, to: u8) {
        self.oam_addr = to;
    }

    pub fn oamdata_read(&mut self) -> u8 {
        self.oam[self.oam_addr as usize]
    }

    pub fn oamdata_write(&mut self, to: u8) {
        self.oam[self.oam_addr as usize] = to;
        
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    pub fn overwrite_oam(&mut self, new: [u8; 0x100]) {
        self.oam = new;
    }
}