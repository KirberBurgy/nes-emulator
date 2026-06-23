use crate::{apu::{noise::APUNoise, pulse::APUPulse, triangle::APUTriangle}, bit_utils::{bit_set, get_bits}};

pub mod pulse;
pub mod triangle;
pub mod noise;
pub mod dmc;

pub const LENGTH_TABLE: [u8; 0x20] = [
    10, 254,  20,   2,  40,   4,  80,   6, 160,   8,  60,  10,  14,  12,  26,  14,
    12,  16,  24,  18,  48,  20,  96,  22, 192,  24,  72,  26,  16,  28,  32,  30    
];



#[repr(usize)]
#[derive(Copy, Clone, Debug)]
enum APUStatus {
    Pulse1Enabled   = 0,
    Pulse2Enabled   = 1,
    TriangleEnabled = 2,
    NoiseEnabled    = 3,
    DMCEnabled      = 4,
}

pub struct APU {
    pub pulse_1:    APUPulse,
    pub pulse_2:    APUPulse,
    pub triangle:   APUTriangle,
    pub noise:      APUNoise,
    //  dmc:        APUDMC

    pub status:     u8,

    pub mode:       bool,
    pub irq:        bool,
    pub irq_done:   bool,
    pub step_num:   usize
}

impl APU {
    pub fn new() -> APU {
        APU
        {
            pulse_1:    APUPulse::new(false),
            pulse_2:    APUPulse::new(true),
            triangle:   APUTriangle::new(),
            noise:      APUNoise::new(),
            
            status:     0b11111,
            mode:       false,
            irq:        false,
            irq_done:   false,
            step_num:   0
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        if addr == 0x4015 {
            let value = 
                ((self.pulse_1.length_counter   > 0)    as u8)                      |
                (((self.pulse_2.length_counter  > 0)    as u8) << 1)                |
                (((self.triangle.length_counter > 0)    as u8) << 2)                |
                (((self.noise.length_counter    > 0)    as u8) << 3)                |
                ((bit_set(self.status, APUStatus::DMCEnabled as usize) as u8) << 4);

            self.irq_done = true;

            value
        }
        else {
            0
        }
    }

    pub fn write(&mut self, addr: u16, to: u8) {
        match addr {
            0x4000 => self.pulse_1.write_misc(to),
            0x4001 => self.pulse_1.write_sweep(to),
            0x4002 => self.pulse_1.write_timer_lo(to),
            0x4003 => self.pulse_1.write_timer_hi(to, bit_set(self.status, APUStatus::Pulse1Enabled as usize)),
            
            0x4004 => self.pulse_2.write_misc(to),
            0x4005 => self.pulse_2.write_sweep(to),
            0x4006 => self.pulse_2.write_timer_lo(to),
            0x4007 => self.pulse_2.write_timer_hi(to, bit_set(self.status, APUStatus::Pulse2Enabled as usize)),

            0x4008 => self.triangle.write_setup(to),
            0x400A => self.triangle.write_timer_lo(to),
            0x400B => self.triangle.write_timer_hi(to, bit_set(self.status, APUStatus::TriangleEnabled as usize)),

            0x400C => self.noise.write_envelope(to),
            0x400E => self.noise.write_period(to),
            0x400F => self.noise.write_length_counter(to, bit_set(self.status, APUStatus::NoiseEnabled as usize)),

            0x4015 => {
                self.status = get_bits(to, 0..5);

                if !bit_set(self.status, APUStatus::Pulse1Enabled as usize)   { self.pulse_1.length_counter = 0; }
                if !bit_set(self.status, APUStatus::Pulse2Enabled as usize)   { self.pulse_2.length_counter = 0; }
                if !bit_set(self.status, APUStatus::TriangleEnabled as usize) { self.triangle.length_counter = 0; }
                if !bit_set(self.status, APUStatus::NoiseEnabled as usize)    { self.noise.length_counter = 0; }
            }

            0x4017 => {
                self.irq    = !bit_set(to, 6);
                self.mode   = bit_set(to, 7);
                
                if !self.irq { 
                    self.irq_done = true;
                }
                
                self.step_num = 0;

                if self.mode {
                    self.tick_half_frame();
                }
            }

            _ => ()
        }
    }

    pub fn tick(&mut self) {
        self.triangle.tick();

        if self.step_num % 2 == 0 {
            self.pulse_1.tick();
            self.pulse_2.tick();
            self.noise.tick();
        }

        if self.mode {
            match self.step_num {
                3729  => self.tick_quarter_frame(),
                7457  => self.tick_half_frame(),
                11186 => self.tick_quarter_frame(),
                18641 => {
                    self.tick_half_frame();
                    self.step_num = 0;
                }
                _ => {}
            }
        } 
        else {
            match self.step_num {
                3729  => self.tick_quarter_frame(),
                7457  => self.tick_half_frame(),
                11186 => self.tick_quarter_frame(),
                14915 => {
                    self.tick_half_frame();

                    if self.irq {
                        self.irq_done = false;
                    }

                    self.step_num = 0;
                }
                _ => {}
            }
        }

        self.step_num += 1;
    }

    fn tick_quarter_frame(&mut self) {
        self.pulse_1.tick_envelope();
        self.pulse_2.tick_envelope();
        self.noise.tick_envelope();
        self.triangle.tick_linear_counter();
    }

    fn tick_half_frame(&mut self) {
        self.tick_quarter_frame();

        self.pulse_1.tick_length_counter();
        self.pulse_2.tick_length_counter();
        self.noise.tick_length_counter();
        self.triangle.tick_length_counter();

        self.pulse_1.tick_sweep();
        self.pulse_2.tick_sweep();
    }

    fn signaling_irq(&self) -> bool {
        // DMC signaling an IRQ, AND:
        !self.irq_done

        // Make sure to change this after adding DMC!
    }

    fn disable_irq_until_next(&mut self) {
        self.irq_done = true;
    }
}