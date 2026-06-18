use std::{cell::RefCell, rc::Rc};

use crate::{bit_utils::{bit_set, get_bits, set_bit, set_bits, toggle_bit}, cartridge::Cartridge, memory_bus::MemoryBus};


#[repr(usize)]
#[derive(Copy, Clone, Debug)]
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
#[derive(Copy, Clone, Debug)]
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
#[derive(Copy, Clone, Debug)]
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

    pub palette:    [u8; 0x020],

    pub cycle:      usize,
    pub scanline:   usize,

    pub odd_frame:  bool,

    pub cart:       Rc<RefCell<Cartridge>>
}

impl PPU {
    pub fn new(cart: Rc<RefCell<Cartridge>>) -> PPU {
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

            palette:    [0; 0x020],

            cycle:      0,
            scanline:   261,

            odd_frame:  false,

            cart
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

    fn rendering(&self) -> bool {
        bit_set(self.mask, PPUMaskFlags::EnableBg as usize) &&
        bit_set(self.mask, PPUMaskFlags::EnableSprites as usize)
    }

    fn increment_x(&mut self) {
        let mut coarse_x = get_bits(self.v, 0..5);

        if coarse_x < 31 {
            coarse_x += 1;
        }
        else {
            coarse_x = 0;
            self.v = toggle_bit(self.v, 10);
        }

        self.v = set_bits(self.v, 0..5, coarse_x);
    }

    fn increment_y(&mut self) {
        let fine_y = get_bits(self.v, 12..15);

        if fine_y < 7 {
            self.v = set_bits(self.v, 12..15, fine_y + 1);

            return;
        }

        self.v = set_bits(self.v, 12..15, 0);
        let mut coarse_y = get_bits(self.v, 5..10);

        match coarse_y {
            29 => {
                coarse_y = 0;
                self.v ^= 0x0800;
            }
            31 => coarse_y = 0,
            _ => coarse_y += 1,
        }

        self.v = set_bits(self.v, 5..10, coarse_y);
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


    fn palette_address(addr: u16) -> usize {
        let mut addr = (addr - 0x3F00) % 0x20;

        if matches!(addr, 0x10 | 0x14 | 0x18 | 0x1C) {
            addr -= 0x10;
        }

        addr as usize
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..0x2000 => self.cart.borrow_mut().chr_read(addr),
            0x2000..0x3F00 => self.cart.borrow_mut().vram_read(addr),

            _ => 0
        }
    }


    pub fn ppudata_read(&mut self) -> u8 {
        let byte = match self.v {
            0x0000..0x3F00 => {
                let ret = self.buffer;

                self.buffer = self.ppu_read(self.v);

                ret
            }

            0x3F00..0x4000 => {
                let ret = self.palette[Self::palette_address(self.v)];

                self.buffer = self.ppu_read(self.v - 0x1000);

                ret
            }

            _ => unreachable!(),
        };

        self.v = self.v.wrapping_add(self.increment());

        byte
    }

    pub fn ppudata_write(&mut self, to: u8) {
        match self.v {
            0x0000..0x2000 => self.cart.borrow_mut().chr_write(self.v, to),
            0x2000..0x3F00 => self.cart.borrow_mut().vram_write(self.v, to),
            0x3F00..0x4000 => self.palette[Self::palette_address(self.v)] = to,

            _ => unreachable!()
        };

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