use std::{cell::RefCell, rc::Rc};

use crate::{bit_utils::{bit_set, copy_bit_ranges, get_bits, set_bit, set_bits, set_bits_all_to, toggle_bit}, cartridge::Cartridge};


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


#[repr(usize)]
#[derive(Copy, Clone, Debug)]
pub enum SpriteAttributes {
    BehindBg            = 5,
    FlipHorizontally    = 6,
    FlipVertically      = 7
}

pub struct PPU {    
    pub control:        u8,
    pub mask:           u8,
    pub status:         u8,
    pub nmi_done:       bool,

    pub v:              u16,
    pub t:              u16,
    pub x:              u8,
    pub w:              bool,

    pub buffer:         u8,

    pub nt_byte:        u8,
    pub at_byte:        u8,
    pub pt_low:         u8,
    pub pt_high:        u8,

    pub p_shift_lo:     u16,
    pub p_shift_hi:     u16,
    
    pub a_latch_lo:     bool,
    pub a_latch_hi:     bool,

    pub a_shift_lo:     u16,
    pub a_shift_hi:     u16,

    pub oam_addr:       u8,
    pub oam:            [u8; 0x100],
    pub secondary_oam:  [u8; 0x020],

    pub sprite_count:   usize,
    pub can_hit_sp0:    bool,

    pub sprite_pt_lo:   [u8; 0x008],
    pub sprite_pt_hi:   [u8; 0x008],
    pub sprite_attr:    [u8; 0x008],
    pub sprite_xs:      [u8; 0x008],

    pub palette_ram:    [u8; 0x020],

    // This is a silly name, but it means the palette index
    // of the latest pixel which was shifted out.
    pub palette_index:  u16,

    pub cycle:          usize,
    pub scanline:       usize,
    pub frame:          usize,

    pub cart:           Rc<RefCell<Cartridge>>,
}

impl PPU {
    pub fn new(cart: Rc<RefCell<Cartridge>>) -> PPU {
        PPU
        {
            control:        0,
            mask:           0,
            status:         0,

            nmi_done:       false,

            v:              0,
            t:              0,
            x:              0,
            w:              false,

            nt_byte:        0,
            at_byte:        0,
            pt_low:         0,
            pt_high:        0,
            
            p_shift_lo:     0,
            p_shift_hi:     0,
            
            a_latch_lo:     false,
            a_latch_hi:     false,

            a_shift_lo:     0,
            a_shift_hi:     0,

            buffer:         0,

            oam_addr:       0,
            oam:            [0; 0x100],
            secondary_oam:  [0; 0x020],

            sprite_count:   0,
            can_hit_sp0:    false,

            sprite_pt_lo:   [0; 0x008],
            sprite_pt_hi:   [0; 0x008],
            sprite_attr:    [0; 0x008],
            sprite_xs:      [0; 0x008],

            palette_ram:    [0; 0x020],

            palette_index:  0x3F00,

            cycle:          0,
            scanline:       261,

            frame:          0,

            cart,
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
        bit_set(self.mask, PPUMaskFlags::EnableBg as usize) ||
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
                self.v = toggle_bit(self.v, 11);
            }
            31 => coarse_y = 0,
            _ => coarse_y += 1,
        }

        self.v = set_bits(self.v, 5..10, coarse_y);
    }

    //                           LO    HI
    fn attribute_bits(&self) -> (bool, bool) {
        let coarse_x = get_bits(self.v, 0..5);
        let coarse_y = get_bits(self.v, 5..10);

        let quadrant_x = bit_set(coarse_x, 1) as u8;
        let quadrant_y = bit_set(coarse_y, 1) as u8;

        // If x, shift += 2 (x controls bit 1).
        // if y, shift += 4 (y controls bit 2).
        // In other words, shift = (quadrant_y << 2) | (quadrant_x << 1)

        let shift = (quadrant_y << 2) | (quadrant_x << 1);
        let shifted = self.at_byte >> shift;

        (bit_set(shifted, 0), bit_set(shifted, 1))
    }

    fn shift_shifters(&mut self) {
        self.p_shift_lo = self.p_shift_lo.wrapping_shl(1);
        self.p_shift_hi = self.p_shift_hi.wrapping_shl(1);

        self.a_shift_lo = self.a_shift_lo.wrapping_shl(1);
        self.a_shift_hi = self.a_shift_hi.wrapping_shl(1);
    }

    fn reload_shifters(&mut self) {
        self.p_shift_lo = set_bits(self.p_shift_lo, 0..8, self.pt_low  as u16);
        self.p_shift_hi = set_bits(self.p_shift_hi, 0..8, self.pt_high as u16);

        self.a_shift_lo = set_bits_all_to(self.a_shift_lo, 0..8, self.a_latch_lo);
        self.a_shift_hi = set_bits_all_to(self.a_shift_hi, 0..8, self.a_latch_hi);
    }

    fn odd_frame(&self) -> bool {
        self.frame % 2 != 0
    }

    pub fn outputting_pixels(&self) -> bool {
        self.scanline <= 239 && self.cycle <= 256
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
            self.t = set_bits(self.t, 12..15, get_bits(to, 0..3) as u16);
            self.t = set_bits(self.t, 5..10,  get_bits(to, 3..8) as u16);

            self.w = false;
        }
        else {
            self.x = get_bits(to, 0..3);
            self.t = set_bits(self.t, 0..5, get_bits(to, 3..8) as u16);

            self.w = true;
        }
    }

    pub fn ppuaddr_write(&mut self, to: u8) {
        if self.w {
            self.t = set_bits(self.t, 0..8, to as u16);
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


    pub fn pal_read(&self, addr: u16) -> u8 {
        get_bits(self.palette_ram[Self::palette_address(addr)], 0..6)
    }

    pub fn ppudata_read(&mut self) -> u8 {
        let mirrored_v = self.v & 0x3FFF;

        let byte = match mirrored_v {
            0x0000..0x3F00 => {
                let ret = self.buffer;

                self.buffer = self.ppu_read(mirrored_v);

                ret
            }

            0x3F00..0x4000 => {
                let ret = self.pal_read(mirrored_v);

                self.buffer = self.ppu_read(mirrored_v - 0x1000);

                ret
            }

            _ => {
                println!("Attempted to read at {:04X}!", mirrored_v);

                unreachable!()
            },
        };

        self.v = self.v.wrapping_add(self.increment());

        byte
    }

    pub fn ppudata_write(&mut self, to: u8) {
        let mirrored_v = self.v & 0x3FFF;
        
        match mirrored_v {
            0x0000..0x2000 => self.cart.borrow_mut().chr_write(mirrored_v, to),
            0x2000..0x3F00 => self.cart.borrow_mut().vram_write(mirrored_v, to),
            0x3F00..0x4000 => self.palette_ram[Self::palette_address(mirrored_v)] = get_bits(to, 0..6),
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


    fn fetch_nametable_byte(&mut self) -> u8 {
        let nametable_address = 0x2000 | get_bits(self.v, 0..12);
        self.cart.borrow_mut().vram_read(nametable_address)
    }


    pub fn signaling_nmi(&self) -> bool {
        !self.nmi_done                                              && 
        bit_set(self.control, PPUControlFlags::NMIEnabled as usize) && 
        bit_set(self.status,  PPUStatusFlags::VBlank as usize)
    }

    pub fn disable_nmi_this_frame(&mut self) {
        self.nmi_done = true;
    }


    fn perform_fetches(&mut self) {
        match self.cycle % 8 {
            1 => {
                self.nt_byte = self.fetch_nametable_byte();
            }
            3 => {
                let attribute_address = 
                    0x23C0                  |
                    (self.v & 0x0C00)       |
                    ((self.v >> 4) & 0x38)  | 
                    ((self.v >> 2) & 0x07);

                self.at_byte = self.cart.borrow_mut().vram_read(attribute_address);
                (self.a_latch_lo, self.a_latch_hi) = self.attribute_bits();
            }
            5 => {
                let base_address =
                    if bit_set(self.control, PPUControlFlags::BgTableAddress as usize) { 0x1000 }
                    else { 0x0000 };

                let bg_pattern_lo_addr = 
                    base_address +
                    (self.nt_byte as u16) * 16 +
                    get_bits(self.v, 12..15);

                self.pt_low = self.cart.borrow_mut().chr_read(bg_pattern_lo_addr);
            }
            7 => {
                let base_address =
                    if bit_set(self.control, PPUControlFlags::BgTableAddress as usize) { 0x1000 }
                    else { 0x0000 };

                let bg_pattern_lo_addr = 
                    base_address +
                    (self.nt_byte as u16) * 16 +
                    get_bits(self.v, 12..15);

                self.pt_high = self.cart.borrow_mut().chr_read(bg_pattern_lo_addr + 8);
            }
            0 => {
                self.reload_shifters();
            }
            _ => {}
        }
    }

    fn sprite_evaluation(&mut self) {
        self.sprite_count = 0;
        self.can_hit_sp0 = false;

        let next_scanline   = self.scanline + 1;
        let tall_sprites    = bit_set(self.control, PPUControlFlags::SpriteSize as usize);
        let sprite_height   = if tall_sprites { 16 } else { 8 };

        for i in 0..64 {
            let y = self.oam[i * 4] as usize;
            let sprite_top = y + 1;

            if next_scanline >= sprite_top && next_scanline < sprite_top + sprite_height {
                if self.sprite_count < 8 {
                    // Copy this sprite to secondary OAM.
                    for j in 0..4 {
                        self.secondary_oam[self.sprite_count * 4 + j] = self.oam[i * 4 + j];
                    }

                    if i == 0 {
                        self.can_hit_sp0 = true;
                    }

                    self.sprite_count += 1;
                }
                else {
                    self.status = set_bit(self.status, PPUStatusFlags::SpriteOverflow as usize, true);

                    break;
                }
            }
        }

        for i in 0..self.sprite_count {
            let y           = self.secondary_oam[i * 4    ] as usize;
            let tile        = self.secondary_oam[i * 4 + 1];
            let attribute   = self.secondary_oam[i * 4 + 2];
            let x           = self.secondary_oam[i * 4 + 3];

            let sprite_top  = y + 1;

            let row = next_scanline - sprite_top;

            let rel_y = 
                if bit_set(attribute, SpriteAttributes::FlipVertically as usize) { sprite_height - 1 - row }
                else { row };

            let (bank, tile) = 
                if tall_sprites { 
                    let bank = if bit_set(tile, 0) { 0x1000 } else { 0x0000 };
                    let tile = set_bit(tile as u16, 0, false);

                    (bank, tile)
                }
                else { 
                    let bank = if bit_set(self.control, PPUControlFlags::SpriteTableAddress as usize) { 0x1000 } else { 0x0000 };

                    (bank, tile as u16)
                };

            let address = if sprite_height == 8 {
                bank + tile * 16 + rel_y as u16
            }
            else {
                let address =
                    if rel_y >= 8 { bank + (tile + 1) * 16 + (rel_y - 8) as u16 }
                    else { bank + tile * 16 + rel_y as u16 };

                address
            };

            let lo = self.cart.borrow_mut().chr_read(address);
            let hi = self.cart.borrow_mut().chr_read(address + 8);

            self.sprite_pt_lo[i] = lo;
            self.sprite_pt_hi[i] = hi;

            self.sprite_attr[i] = attribute;
            self.sprite_xs[i] = x;
        }

        for _ in self.sprite_count..8 {
            let dummy_addr = if tall_sprites {
                0x1000 + (0xFF * 16)
            } else {
                let bank = if bit_set(self.control, PPUControlFlags::SpriteTableAddress as usize) { 0x1000 } else { 0x0000 };
                bank + (0xFF * 16)
            };

            self.cart.borrow_mut().chr_read(dummy_addr);
            self.cart.borrow_mut().chr_read(dummy_addr + 8);
        }
    }

    fn render_sprites(&mut self, bg_opaque: bool) {
        let current_x = self.cycle as usize;



        for i in 0..self.sprite_count {
            let x = self.sprite_xs[i] as usize;
            
            if current_x < x || current_x >= x + 8 { continue; }

            if x < 8 && !bit_set(self.mask, PPUMaskFlags::ShowBorderSprites as usize) {
                continue;
            }

            let offset = current_x - x;
            let attr = self.sprite_attr[i];

            let bit_index = 
                if bit_set(attr, SpriteAttributes::FlipHorizontally as usize) { offset } 
                else { 7 - offset };

            let pal_lo = bit_set(self.sprite_pt_lo[i], bit_index) as u16;
            let pal_hi = bit_set(self.sprite_pt_hi[i], bit_index) as u16;

            let pattern_bits = (pal_hi << 1) | pal_lo;

            // If the pattern bits are zero, this sprite pixel is translucent anyways.
            if pattern_bits == 0 {
                continue;
            }

            if bg_opaque && self.can_hit_sp0 && i == 0 {
                if current_x < 255 {
                    self.status = set_bit(self.status, PPUStatusFlags::Sprite0Hit as usize, true);
                }
            }

            let attr_lo = bit_set(attr, 0) as u16;
            let attr_hi = bit_set(attr, 1) as u16;
            let attribute_bits = (attr_hi << 1) | attr_lo;

            let palette_index = 0x3F10 | (attribute_bits << 2) | pattern_bits;

            let behind_bg = bit_set(attr, SpriteAttributes::BehindBg as usize);
            if !bg_opaque || !behind_bg {
                self.palette_index = palette_index;
            }

            return;
        }
    }

    pub fn tick(&mut self) {
        if self.cycle == 0 {
            self.cycle += 1;

            return;
        }

        match self.scanline {
            0..=239 | 261 => {
                let rendering = self.rendering();
                
                if (1..=256).contains(&self.cycle) || (321..=336).contains(&self.cycle) {
                    self.shift_shifters();
                    self.perform_fetches();
                    
                    if self.cycle % 8 == 0 && rendering {
                        self.increment_x();
                    }
                }

                if self.cycle == 256 && rendering {
                    self.increment_y();
                }

                if self.cycle == 257 && rendering {
                    self.v = copy_bit_ranges(self.v, self.t, &[0..5, 10..11]);

                    if self.scanline == 261 {
                        self.sprite_count = 0;
                        self.can_hit_sp0 = false;
                    }
                    else {
                        self.sprite_evaluation();
                    }
                }

                if self.cycle == 338 || self.cycle == 340 {
                    self.fetch_nametable_byte();
                }
                
                let mut bg_opaque = false;

                if self.scanline == 261 {
                    if self.cycle == 1 {
                        self.status = set_bit(self.status, PPUStatusFlags::Sprite0Hit as usize, false);
                        self.status = set_bit(self.status, PPUStatusFlags::SpriteOverflow as usize, false);
                        self.status = set_bit(self.status, PPUStatusFlags::VBlank as usize, false);
                    }
                    
                    if rendering && self.cycle >= 280 && self.cycle <= 304 {
                        self.v = copy_bit_ranges(self.v, self.t, &[5..10, 11..15]);
                    }
                    
                    if self.cycle == 339 && self.odd_frame() && rendering {
                        self.cycle = 0;
                        self.scanline = 0;

                        return;
                    }
                } 
                else if bit_set(self.mask, PPUMaskFlags::EnableBg as usize) {
                    if self.cycle <= 8 && !bit_set(self.mask, PPUMaskFlags::ShowBorderBg as usize) {
                        self.palette_index = 0x3F00;
                    }
                    else {
                        let bit_pos = 15 - self.x as usize;

                        let pal_lo = bit_set(self.p_shift_lo, bit_pos) as u16;
                        let pal_hi = bit_set(self.p_shift_hi, bit_pos) as u16;

                        let attr_lo = bit_set(self.a_shift_lo, bit_pos) as u16;
                        let attr_hi = bit_set(self.a_shift_hi, bit_pos) as u16;

                        let pattern_bits = (pal_hi << 1) | pal_lo;
                        let attribute_bits = (attr_hi << 1) | attr_lo;

                        self.palette_index =
                            0x3F00 |
                            if pattern_bits == 0 { 0 }
                            else { (attribute_bits << 2) | pattern_bits };
                    }

                    bg_opaque = self.palette_index != 0x3F00;
                }
                else {
                    self.palette_index = 0x3F00;
                }

                if (1..=256).contains(&self.cycle) && bit_set(self.mask, PPUMaskFlags::EnableSprites as usize) && self.scanline != 261 {
                    self.render_sprites(bg_opaque);
                }
            }

            241 => {
                if self.cycle == 1 {
                    self.nmi_done = false;
                    self.status = set_bit(self.status, PPUStatusFlags::VBlank as usize, true);
                }
            }
            
            260 if self.cycle == 340 => {
                self.frame += 1;
            }

            _ => {}
        }

        self.cycle += 1;
        
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;
        }

        if self.scanline >= 262 {
            self.scanline = 0;
        }
    }
}