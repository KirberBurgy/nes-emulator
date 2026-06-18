use std::{cell::RefCell, rc::Rc};

use crate::{cartridge::Cartridge, ppu::PPU};

pub struct MemoryBus {
    pub ram:        Box<[u8; 0x0800]>,
    pub cartridge:  Rc<RefCell<Cartridge>>,
    pub ppu:        PPU
}

impl MemoryBus {
    pub fn new(cartridge: Cartridge) -> MemoryBus {
        let cell = Rc::new(RefCell::new(cartridge));

        MemoryBus { ram: Box::new([0; 0x0800]), ppu: PPU::new(cell.clone()), cartridge: cell }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..0x2000  => self.ram[(addr % 0x0800) as usize],

            0x2000..0x4000  => match addr % 0x0008 {
                0x0002 => self.ppu.ppustatus_read(),
                0x0004 => self.ppu.oamdata_read(),
                0x0007 => self.ppu.ppudata_read(),

                _ => unreachable!()
            }

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
            }

            0x8000..        => self.cartridge.borrow_mut().prg_write(addr, value),

            _ => {}
        }
    }
}