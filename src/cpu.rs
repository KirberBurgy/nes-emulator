use crate::bit_utils::binary_bit;


#[repr(usize)]
pub enum CPUFlags {
    Carry               = binary_bit(0),
    Zero                = binary_bit(1),
    InterruptDisable    = binary_bit(2),
    Decimal             = binary_bit(3),
    //                  B
    //                  1
    Overflow            = binary_bit(6),
    Negative            = binary_bit(7),
}

pub struct CPU {
    pub pc:     u16,

    pub sp:     u8,
    pub p:      u8,

    pub a:      u8,

    pub x:      u8,
    pub y:      u8,

    pub ram:    Box<[u8; 0x8000]>,
}