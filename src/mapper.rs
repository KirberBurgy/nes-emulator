pub trait Mapper {
    fn prg_ram_read(&mut self, _addr: u16) -> u8 { 0 }
    fn prg_ram_write(&mut self, _addr: u16, _value: u8) {}

    fn prg_read(&mut self, addr: u16) -> u8;
    fn prg_write(&mut self, _addr: u16, _value: u8) {}

    fn chr_read(&mut self, addr: u16) -> u8;
    fn chr_write(&mut self, _addr: u16, _value: u8) {}

    fn vram_read(&mut self, addr: u16) -> u8;
    fn vram_write(&mut self, addr: u16, value: u8);
}

#[derive(Copy, Clone, Debug)]
pub enum NametableMirroring {
    Horizontal,
    Vertical,
    FourScreen
}

