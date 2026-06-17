pub struct MemoryBus {
    pub ram: Box<[u8; 0x10000]>
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus { ram: Box::new([0; 0x10000]) }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000.. => self.ram[addr as usize]
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000.. => self.ram[addr as usize] = value
        }
    }
}