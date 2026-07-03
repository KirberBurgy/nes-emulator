use std::{cell::RefCell, rc::Rc};

use crate::{bit_utils::{bit_set, get_bits}, cartridge::Cartridge};

const RATE_TABLE: [u16; 0x10] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54
];

pub struct APUDMC {
    pub timer_reset:        u16,
    pub timer:              u16,

    pub sample_address:     u16,
    pub current_address:    u16,
    pub sample_length:      u16,
    pub remaining_samples:  u16,
    pub bits_remaining:     u8,

    pub sample_buffer:      u8,
    pub output_shifter:     u8,
    pub output_level:       u8,
    pub buffer_empty:       bool,

    pub irq_enabled:        bool,
    pub irq_pending:        bool,
    pub will_loop:          bool,
    pub silence:            bool,

    pub transferring:       bool,

    pub cartridge:          Rc<RefCell<Cartridge>>
}

impl APUDMC {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> APUDMC {
        APUDMC
        {
            timer_reset:        0,
            timer:              0,

            sample_address:     0,
            current_address:    0,
            sample_length:      0,
            remaining_samples:  0,
            bits_remaining:     8,

            sample_buffer:      0,
            output_shifter:     0,
            output_level:       0,
            buffer_empty:       true,

            irq_enabled:        false,
            irq_pending:        false,
            will_loop:          false,
            silence:            false,
            
            transferring:       false,

            cartridge
        }
    }

    pub fn write_flags_and_rate(&mut self, value: u8) {
        self.timer_reset    = RATE_TABLE[get_bits(value, 0..4) as usize];

        self.will_loop      = bit_set(value, 6);
        self.irq_enabled    = bit_set(value, 7);

        if !self.irq_enabled { self.irq_pending = false; }
    }

    pub fn write_load(&mut self, value: u8) {
        self.output_level = get_bits(value, 0..7);
    }

    pub fn write_sample_address(&mut self, value: u8) {
        self.sample_address = 0xC000 + (value as u16) * 0x40;
        self.current_address = self.sample_address;
    }

    pub fn write_sample_length(&mut self, value: u8) {
        self.sample_length      = ((value as u16) << 4) + 1;
        self.remaining_samples  = self.sample_length;
    }

    pub fn tick(&mut self) {
        if self.buffer_empty && self.remaining_samples > 0 {
            self.sample_buffer = self.cartridge.borrow_mut().prg_read(self.current_address);
            self.transferring = true;
            self.buffer_empty = false;
            
            if self.current_address == 0xFFFF   { self.current_address = 0x8000;    } 
            else                                { self.current_address += 1;        }

            self.remaining_samples -= 1;

            if self.remaining_samples == 0 {
                if self.will_loop {
                    self.remaining_samples = self.sample_length;
                    self.current_address = self.sample_address;
                } 
                else { 
                    self.irq_pending = self.irq_enabled; 
                }
            }
        }

        if self.timer != 0 {
            self.timer -= 1;

            return;
        }

        self.timer = self.timer_reset;

        if !self.silence {
            let will_add = bit_set(self.output_shifter, 0);

            if will_add {
                if self.output_level <= 125 { self.output_level += 2; }
            } else {
                if self.output_level >= 2 { self.output_level -= 2; }
            }
        }

        self.output_shifter >>= 1;
        self.bits_remaining -= 1;

        if self.bits_remaining == 0 {
            self.bits_remaining = 8;
            
            if self.buffer_empty {
                self.silence = true;
            } 
            else {
                self.silence = false;
                self.output_shifter = self.sample_buffer;
                self.buffer_empty = true;
            }
        }
    }

    pub fn volume(&self) -> u8 {
        self.output_level
    }

    pub fn signaling_irq(&self) -> bool {
        self.irq_pending
    }

    pub fn disable_irq_until_next(&mut self) {
        self.irq_pending = false;
    }

    pub fn done_transferring(&mut self) {
        self.transferring = false;
    }
}