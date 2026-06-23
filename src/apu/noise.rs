use crate::{apu::LENGTH_TABLE, bit_utils::{bit_set, get_bits, set_bit}};

const PERIOD_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068
];

pub struct APUNoise {
    pub halt:               bool,
    pub constant_volume:    bool,
    pub envelope_period:    u8,

    pub mode:               bool,
    pub period:             u8,
    pub delay:              u16,

    pub length_counter:     u8,

    pub envelope_start:     bool,
    pub decay_divider:      u8,
    pub decay_level:        u8,

    pub shift:              u16
}

impl APUNoise {
    pub fn new() -> APUNoise {
        APUNoise
        {
            halt:               false,
            constant_volume:    false,
            envelope_period:    0,

            mode:               false,
            delay:              0,
            period:             0,

            length_counter:     0,

            envelope_start:     false,
            decay_divider:      0,
            decay_level:        0,

            shift:              1,
        }
    }

    pub fn write_envelope(&mut self, to: u8) {
        self.envelope_period    = get_bits(to, 0..4);
        self.constant_volume    = bit_set(to, 4);
        self.halt               = bit_set(to, 5);
    }

    pub fn write_period(&mut self, to: u8) {
        self.period = get_bits(to, 0..4);
        self.mode   = bit_set(to, 7);
    }

    pub fn write_length_counter(&mut self, to: u8, lc_enabled: bool) {
        if lc_enabled {
            self.length_counter = LENGTH_TABLE[get_bits(to, 3..8) as usize];
        }

        self.envelope_start = true;
    }

    pub fn tick(&mut self) {
        if self.delay != 0 {
            self.delay -= 1;

            return;
        }

        self.delay = PERIOD_TABLE[self.period as usize];
        
        let feedback = bit_set(self.shift, 0) ^
            if self.mode { bit_set(self.shift, 6) }
            else { bit_set(self.shift, 1) };
        
        self.shift >>= 1;
        self.shift = set_bit(self.shift, 14, feedback);
    }

    pub fn tick_envelope(&mut self) {
        if self.envelope_start {
            self.envelope_start = false;
            self.decay_level = 15;
            self.decay_divider = self.envelope_period;

            return;
        }

        if self.decay_divider == 0 {
            self.decay_divider = self.envelope_period;
                
            if self.decay_level > 0 {
                self.decay_level -= 1;
            } 
            else if self.halt {
                self.decay_level = 15;
            }
        } 
        else {
            self.decay_divider -= 1;
        }
    }

    pub fn tick_length_counter(&mut self) {
        if !self.halt && self.length_counter != 0 {
            self.length_counter -= 1;
        }
    }

    pub fn volume(&self) -> u8 {
        if bit_set(self.shift, 0) || self.length_counter == 0 {
            return 0;
        }

        if self.constant_volume {
            self.envelope_period
        }
        else {
            self.decay_level
        }
    }
}