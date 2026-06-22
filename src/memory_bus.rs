use std::{cell::RefCell, rc::Rc};

use crate::{cartridge::Cartridge, controller::Controller, ppu::PPU};

pub struct MemoryBus<'a> {
    pub ram:        Box<[u8; 0x0800]>,
    pub cartridge:  Rc<RefCell<Cartridge>>,
    
    pub ppu:        PPU,
    
    pub player_1:   Option<Controller<'a>>,
    pub player_2:   Option<Controller<'a>>,
    
    pub dma_signal: bool
}

impl<'a> MemoryBus<'a> {
    pub fn new(cartridge: Cartridge, player_1: Option<Controller<'a>>, player_2: Option<Controller<'a>>) -> Self {
        let cell = Rc::new(RefCell::new(cartridge));

        Self { ram: Box::new([0; 0x0800]), ppu: PPU::new(cell.clone()), player_1, player_2, cartridge: cell, dma_signal: false }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..0x2000  => self.ram[(addr % 0x0800) as usize],

            0x2000..0x4000  => match addr % 0x0008 {
                0x0002 => self.ppu.ppustatus_read(),
                0x0004 => self.ppu.oamdata_read(),
                0x0007 => self.ppu.ppudata_read(),

                _ => unreachable!()
            },

            0x4016          => match &mut self.player_1 {
                Some(controller)    => controller.read(),
                None                => 0
            },

            0x4017          => match &mut self.player_2 {
                Some(controller)    => controller.read(),
                None                => 0
            },

            0x8000..        => self.cartridge.borrow_mut().prg_read(addr),

            _ => 0 
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..0x2000  => self.ram[(addr % 0x0800) as usize] = value,

            0x2000..0x4000  => match addr % 0x0008 {
                0x0000 => self.ppu.ppuctrl_write(value),
                0x0001 => self.ppu.ppumask_write(value),
                0x0003 => self.ppu.oamaddr_write(value),
                0x0004 => self.ppu.oamdata_write(value),
                0x0005 => self.ppu.ppuscroll_write(value),
                0x0006 => self.ppu.ppuaddr_write(value),
                0x0007 => self.ppu.ppudata_write(value),

                _ => unreachable!()
            },

            0x4014          => {
                let mut new_oam = [0; 0x100];
                let base_address = (value as u16) << 8;

                for i in 0x000..0x100 {
                    new_oam[i as usize] = self.read(base_address + i);
                }

                self.ppu.overwrite_oam(new_oam);
                self.dma_signal = true;
            },

            0x4016          => {
                if let Some(controller) = &mut self.player_1 {
                    controller.write(value);
                }

                if let Some(controller) = &mut self.player_2 {
                    controller.write(value);
                }
            }

            0x8000..        => self.cartridge.borrow_mut().prg_write(addr, value),

            _ => {}
        }
    }

    pub fn stop_dma_signal(&mut self) {
        self.dma_signal = false;
    }
}