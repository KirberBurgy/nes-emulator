use crate::{apu::LENGTH_TABLE, bit_utils::{bit_set, get_bits, set_bits}};

const DUTY_CYCLE_SEQUENCES: [[bool; 8]; 4] = [
    [false,  true, false, false, false, false, false, false],
    [false,  true,  true, false, false, false, false, false],
    [false,  true,  true,  true,  true, false, false, false],
    [ true, false, false,  true,  true,  true,  true,  true],
];

pub struct APUPulse {
    pub duty_cycle:         u8,
    pub sequence_step:      u8,
    pub envelope_period:    u8,

    pub sweep_enable:       bool,
    pub divide_period:      u8,
    pub sweep_negate:       bool,
    pub shift_count:        u8,
    pub sweep_divider:      u8,
    pub sweep_reload:       bool,

    pub halt:               bool,
    pub constant_vol:       bool,

    pub timer_reset:        u16,
    pub timer:              u16,

    pub length_counter:     u8,

    pub envelope_start:     bool,
    pub decay_divider:      u8,
    pub decay_level:        u8,

    pub is_2nd:             bool
}

impl APUPulse {
    pub fn new(is_2nd: bool) -> APUPulse {
        APUPulse
        {
            duty_cycle:         0,
            sequence_step:      0,
            envelope_period:    0,

            sweep_enable:       false,
            divide_period:      0,
            sweep_negate:       false,
            shift_count:        0,
            sweep_divider:      0,
            sweep_reload:       false,

            halt:               false,
            constant_vol:       false,

            timer_reset:        0,
            timer:              0,

            length_counter:     0,

            envelope_start:     false,
            decay_divider:      0,
            decay_level:        0,

            is_2nd
        }
    }

    #[inline(always)]
    fn is_silent(&self) -> bool {
        self.target_period() > 0x07FF ||
        self.length_counter == 0      ||
        self.timer_reset < 8
    }

    fn target_period(&self) -> u16 {
        let delta = self.timer_reset >> self.shift_count;
        
        if self.sweep_negate {
            if self.is_2nd { self.timer_reset.saturating_sub(delta) } 
            else { self.timer_reset.saturating_sub(delta).saturating_sub(1) }
        } 
        else {
            self.timer_reset + delta
        }
    }

    pub fn write_misc(&mut self, to: u8) {
        self.envelope_period    = get_bits(to, 0..4);
        self.constant_vol       = bit_set(to, 4);
        self.halt               = bit_set(to, 5);
        self.duty_cycle         = get_bits(to, 6..8);
    }

    pub fn write_sweep(&mut self, to: u8) {
        self.shift_count    = get_bits(to, 0..3);
        self.sweep_negate   = bit_set(to, 3);
        self.divide_period  = get_bits(to, 4..7);
        self.sweep_enable   = bit_set(to, 7);

        self.sweep_reload   = true;
    }

    pub fn write_timer_lo(&mut self, to: u8) {
        self.timer_reset = set_bits(self.timer_reset, 0..8, to as u16);
    }

    pub fn write_timer_hi(&mut self, to: u8, lc_enabled: bool) {
        self.timer_reset    = set_bits(self.timer_reset, 8..11, get_bits(to, 0..3) as u16);
        
        if lc_enabled {
            self.length_counter = LENGTH_TABLE[get_bits(to, 3..8) as usize];
        }

        self.sequence_step = 0;
        self.envelope_start = true;
    }

    pub fn tick(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_reset;
            self.sequence_step = (self.sequence_step + 1) % 8;
        } 
        else {
            self.timer -= 1;
        }
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

    pub fn tick_sweep(&mut self) {
        let target_period = self.target_period();

        if self.sweep_divider == 0 
            && self.sweep_enable 
            && self.shift_count > 0 
            && self.timer_reset >= 8 
            && target_period <= 0x07FF
        {
            self.timer_reset = target_period;
        }

        if self.sweep_divider == 0 || self.sweep_reload {
            self.sweep_divider = self.divide_period;
            self.sweep_reload = false;
        } 
        else {
            self.sweep_divider -= 1;
        }
    }

    // Only runs if the APU length counter flag is not clear.
    // However it's silly to store this state in the pulse unit!
    pub fn tick_length_counter(&mut self) {
        if !self.halt && self.length_counter != 0 {
            self.length_counter -= 1;
        }
    }

    pub fn volume(&self) -> u8 {
        if self.is_silent() || !DUTY_CYCLE_SEQUENCES[self.duty_cycle as usize][self.sequence_step as usize] {
            return 0;
        }

        if self.constant_vol {
            self.envelope_period
        }
        else {
            self.decay_level
        }
    }
}