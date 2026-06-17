pub trait Mapper {
    fn prg_read(&mut self, addr: u16) -> u8;
    fn prg_write(&mut self, _addr: u16, _value: u8) {}

    fn chr_read(&mut self, addr: u16) -> u8;
    fn chr_write(&mut self, _addr: u16, _value: u8) {}
}

pub enum NametableMirroring {
    Horizontal,
    Vertical,
    FourScreen
}