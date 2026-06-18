use crate::{cartridge::Cartridge, cpu::CPU, memory_bus::MemoryBus};

pub struct NES {
    pub cpu: CPU,
    pub bus: MemoryBus
}

impl NES {
    pub fn new(cart: Cartridge) -> NES {
        NES { cpu: CPU::new(), bus: MemoryBus::new(cart) }
    }

    pub fn tick(&mut self) {
        if self.cpu.delay == 0 && self.bus.ppu.signaling_nmi() {
            self.cpu.delay = 7;

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
            self.bus.ppu.tick();
        }
    }
}