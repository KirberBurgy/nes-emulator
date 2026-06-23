use crate::bit_utils::{bit_set};

pub struct Controller {
    pub a:      bool,
    pub b:      bool,
    pub left:   bool,
    pub right:  bool,
    pub up:     bool,
    pub down:   bool,
    pub start:  bool,
    pub select: bool,

    pub output: u8,
    pub latch:  bool
}

impl Controller {
    pub fn new() -> Self {
        Self { a: false, b: false, left: false, right: false, up: false, down: false, start: false, select: false, output: 0, latch: false }
    }

    pub fn write(&mut self, to: u8) {
        let bit = bit_set(to, 0);

        // Fill the output shifter with controller data.
        if !self.latch && bit {
            self.output =
                ((self.a      as u8) << 7) |
                ((self.b      as u8) << 6) |
                ((self.select as u8) << 5) |
                ((self.start  as u8) << 4) |
                ((self.up     as u8) << 3) |
                ((self.down   as u8) << 2) |
                ((self.left   as u8) << 1) |
                ( self.right  as u8      );
        }

        self.latch = bit;
    }

    pub fn read(&mut self) -> u8 {
        if self.latch { 
            return self.a as u8;
        }

        let bit = bit_set(self.output, 7) as u8;
        self.output <<= 1;

        bit
    }
}