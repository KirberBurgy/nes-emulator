use std::{fs::File, io::{Write}};

use crate::{cartridge::Cartridge, nes::{NES, NES_PALETTE}};

pub mod bit_utils;

pub mod memory_bus;

pub mod cpu;
pub mod instructions;

pub mod ppu;

pub mod controller;

pub mod mapper;
pub mod mappers;

pub mod cartridge;

pub mod nes;

use std::io::{self};

pub fn dump_chr_to_ppm(cart: &mut Cartridge, filename: &str) -> io::Result<()> {
    let mut file = File::create(filename)?;
    
    writeln!(file, "P3")?;
    writeln!(file, "{} {}", 256, 240)?;
    writeln!(file, "255")?;
    
    let palette = [
        (0, 0, 0),
        (85, 85, 85),
        (170, 170, 170),
        (255, 255, 255),
    ];
    
    for py in 0..240 {
        for px in 0..256 {
            let tile_x = px / 8;
            let tile_y = py / 8;
            
            let pixel_x = px % 8;
            let pixel_y = py % 8;
            
            let tile_index = tile_y * 32 + tile_x;
            let tile_offset = tile_index * 16;
            
            let plane0 = cart.chr_read((tile_offset + pixel_y)     % 0x2000);
            let plane1 = cart.chr_read((tile_offset + pixel_y + 8) % 0x2000);
                
            let bit0 = (plane0 >> (7 - pixel_x)) & 1;
            let bit1 = (plane1 >> (7 - pixel_x)) & 1;
                
            let color_index = (bit1 << 1) | bit0;
            let color = palette[color_index as usize];
                
            writeln!(file, "{} {} {}", color.0, color.1, color.2)?;
        }
    }
    
    Ok(())
}

fn capture(nes: &mut NES, name: &str) {
    let mut file = File::create(&format!("screenshot/{}.ppm", name)).unwrap();
    writeln!(file, "P3").unwrap();
    writeln!(file, "256 240").unwrap();
    writeln!(file, "255").unwrap();

    for y in 0..240 {
        for x in 0..256 {
            let (r, g, b) = nes.framebuffer[x + y * 256];

            writeln!(file, "{} {} {}", r, g, b).unwrap();    
        }
    }
}

fn record(nes: &mut NES, frames: usize, name: &str) {
    for i in 0..frames {
        let old = nes.bus.ppu.frame;

        while nes.bus.ppu.frame < old + 60 {
            nes.tick();
        }

        capture(nes, &format!("{}{}", name, i));
    }
}

fn main() {
    unsafe { std::env::set_var("RUST_BACKTRACE", "full"); }
    let donkey_kong = Cartridge::load("tests/roms/Donkey Kong.nes").unwrap();
    let mut nes = NES::new(donkey_kong);

    nes.reset();

    {
        let now = std::time::Instant::now();
        while nes.bus.ppu.frame != 1 {
            nes.tick();
        }
        println!("Time taken for one frame: {:?}", now.elapsed());
    }

    while nes.bus.ppu.frame < 300 {
        nes.tick();
    }

    {
        let mut cart = nes.bus.cartridge.borrow_mut();
        dump_chr_to_ppm(&mut cart, "screenshot/chr.ppm").unwrap();
    }
    

    // PAL
    {
        let mut file = File::create("screenshot/pal.ppm").unwrap();
        writeln!(file, "P3").unwrap();
        
        let w = 16;
        let h = 16;

        let rows = 8;
        let columns = 32 / rows;
        
        writeln!(file, "{} {}", columns * w, rows * h).unwrap();
        writeln!(file, "255").unwrap();

        for y in 0..(rows * h) {
            for x in 0..(columns * w) {
                let i = x / w;
                let j = y / h;

                let (r, g, b) = NES_PALETTE[nes.bus.ppu.pal_read(0x3F00 + (i + j * columns)) as usize];

                writeln!(file, "{} {} {}", r, g, b).unwrap();    
            }
        }
    }

    record(&mut nes, 10, "game");
}
