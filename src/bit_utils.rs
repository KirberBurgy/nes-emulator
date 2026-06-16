use num_traits::PrimInt;

pub const fn address_from_bytes(lsb: u8, msb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

pub fn bit_set<T: PrimInt>(val: T, bit: usize) -> bool {
    ((val >> bit) & T::one()) != T::zero()
}

pub fn set_bit<T: PrimInt>(val: T, bit: usize, to: bool) -> T {
    let b = T::one() << bit;

    if to {
        val | b
    }
    else {
        val & !b
    }
}