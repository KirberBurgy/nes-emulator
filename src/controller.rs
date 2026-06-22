use crate::bit_utils::{bit_set};

pub trait ControllerInput {
    fn right    (&self) -> bool;
    fn left     (&self) -> bool;
    fn down     (&self) -> bool;
    fn up       (&self) -> bool;
    fn start    (&self) -> bool;
    fn select   (&self) -> bool;
    fn b        (&self) -> bool;
    fn a        (&self) -> bool;
}

pub struct Controller<'a> {
    pub input:      &'a dyn ControllerInput,

    pub output:     u8,
    pub latch:      bool
}

impl<'a> Controller<'a> {
    pub fn new(input: &'a impl ControllerInput) -> Self {
        Self { input, output: 0, latch: false }
    }

    pub fn write(&mut self, to: u8) {
        let bit = bit_set(to, 0);

        // Fill the output shifter with controller data.
        if !self.latch && bit {
            self.output =
                ((self.input.a      () as u8) << 7) |
                ((self.input.b      () as u8) << 6) |
                ((self.input.select () as u8) << 5) |
                ((self.input.start  () as u8) << 4) |
                ((self.input.up     () as u8) << 3) |
                ((self.input.down   () as u8) << 2) |
                ((self.input.left   () as u8) << 1) |
                ( self.input.right  () as u8      );
        }

        self.latch = bit;
    }

    pub fn read(&mut self) -> u8 {
        if self.latch { 
            return self.input.a() as u8;
        }

        let bit = bit_set(self.output, 7) as u8;
        self.output <<= 1;

        bit
    }
}