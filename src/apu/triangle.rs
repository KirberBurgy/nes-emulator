use crate::{apu::LENGTH_TABLE, bit_utils::{bit_set, get_bits, set_bits}};

const SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15
];

pub struct APUTriangle {
    pub linear_counter: u8,
    pub linear_reset:   u8,
    pub linear_reload:  bool,

    pub sequence_step:  u8,

    pub control:        bool,

    pub timer_reset:    u16,
    pub timer:          u16,

    pub length_counter: u8,
}

impl APUTriangle {
    pub fn new() -> APUTriangle {
        APUTriangle
        {
            linear_counter: 0,
            linear_reset:   0,
            linear_reload:  false,

            sequence_step:  0,

            control:        false,

            timer_reset:    0,
            timer:          0,

            length_counter: 0
        }
    }

    pub fn write_setup(&mut self, to: u8) {
        self.linear_reset   = get_bits(to, 0..7);
        self.control        = bit_set(to, 7);
    }

    pub fn write_timer_lo(&mut self, to: u8) {
        self.timer_reset = set_bits(self.timer_reset, 0..8, to as u16);
    }
    
    pub fn write_timer_hi(&mut self, to: u8, lc_enabled: bool) {
        self.timer_reset = set_bits(self.timer_reset, 8..11, get_bits(to, 0..3) as u16);
        
        if lc_enabled {
            self.length_counter = LENGTH_TABLE[get_bits(to, 3..8) as usize];
        }

        self.linear_reload = true;
    }

    pub fn tick(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_reset;

            if self.linear_counter != 0 && self.length_counter != 0 {
                self.sequence_step = (self.sequence_step + 1) % 32;
            }
        } 
        else {
            self.timer -= 1;
        }
    }

    pub fn tick_linear_counter(&mut self) {
        if self.linear_reload {
            self.linear_counter = self.linear_reset;
        } 
        else if self.linear_counter != 0 {
            self.linear_counter -= 1;
        }

        if !self.control {
            self.linear_reload = false;
        }
    }

    pub fn tick_length_counter(&mut self) {
        if !self.control && self.length_counter != 0 {
            self.length_counter -= 1;
        }
    }

    pub fn volume(&self) -> u8 {
        SEQUENCE[self.sequence_step as usize]
    }
}