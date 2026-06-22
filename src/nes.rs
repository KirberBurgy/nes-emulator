use crate::{cartridge::Cartridge, controller::Controller, cpu::CPU, memory_bus::MemoryBus};

pub const NES_PALETTE: [(u8, u8, u8); 64] = [
    (98u8, 98u8, 98u8), (  0,  28, 149), ( 25,   4, 172), ( 66,   0, 157), ( 97,   0, 107), (101,   5,   0), (110,   0,  37), ( 73,  30,   0),
    (  34,   55,    0), (  0,  73,   0), (  0,  79,   0), (  0,  72,  22), (  0,  53,  94), (  0,   0,   0), (  0,   0,   0), (  0,   0,   0),

    ( 171,  171,  171), ( 12,  78, 219), ( 61,  46, 255), (113,  21, 243), (155,  11, 185), (176,  18,  98), (169,  39,   4), (137,  70,   0),
    (  87,  102,    0), ( 35, 127,   0), (  0, 137,   0), (  0, 131,  50), (  0, 109, 144), (  0,   0,   0), (  0,   0,   0), (  0,   0,   0),

    ( 255,  255,  255), ( 87, 165, 255), (130, 135, 255), (180, 109, 255), (223,  96, 255), (248,  99, 198), (248, 116, 109), (222, 144,  32),
    ( 179,  174,    0), (129, 200,   0), ( 86, 213,  34), ( 61, 211, 111), ( 62, 193, 200), ( 78,  78,  78), (  0,   0,   0), (  0,   0,   0),

    ( 255,  255,  255), (190, 224, 255), (205, 212, 255), (224, 202, 255), (241, 196, 255), (252, 196, 239), (253, 202, 206), (245, 212, 175),
    ( 230,  223,  156), (211, 233, 154), (194, 239, 168), (183, 239, 196), (182, 234, 229), (184, 184, 184), (  0,   0,   0), (  0,   0,   0),
];

pub struct NES<'a> {
    pub cpu:            CPU,
    pub bus:            MemoryBus<'a>,

    pub framebuffer:    Box<[(u8, u8, u8); 256 * 240]>
}

impl<'a> NES<'a> {
    pub fn new(cart: Cartridge, player_1: Option<Controller<'a>>, player_2: Option<Controller<'a>>) -> Self {
        Self { 
            cpu: CPU::new(), 

            bus: MemoryBus::new(cart, player_1, player_2), 
            
            framebuffer: Box::new([(0, 0, 0); 256 * 240]) 
        }
    }

    pub fn reset(&mut self) {
        self.cpu.jump_to_startup(&mut self.bus);
    }

    pub fn tick(&mut self) {
        if self.cpu.delay == 0 && self.bus.ppu.signaling_nmi() {
            self.cpu.jump_to_nmi_handler(&mut self.bus);

            // Hack: ensures that an NMI handler can't be jumped to
            // more than once per frame.
            self.bus.ppu.disable_nmi_this_frame();
        }

        self.cpu.tick(&mut self.bus);

        if self.bus.dma_signal {
            // Hack: to get the cycle which the DMA transfer began on
            // we just minus one from the cycles (since tick advances one cycle always).
            if (self.cpu.cycles - 1) % 2 == 0 {
                self.cpu.delay += 513;
            }
            else {
                self.cpu.delay += 514;
            }

            self.bus.stop_dma_signal();
        }


        for _ in 0..3 {
            let scanline = self.bus.ppu.scanline;
            let cycle = self.bus.ppu.cycle;

            self.bus.ppu.tick();

            if scanline < 240 && (1..=256).contains(&cycle) {
                let x = (cycle - 1) as usize; 
                let y = scanline as usize;
                
                let index = x + y * 256;

                self.framebuffer[index] = NES_PALETTE[self.bus.ppu.pal_read(self.bus.ppu.palette_index) as usize];
            }
        }
    }
}