use std::ops::Range;

use num_traits::PrimInt;

pub const fn address_from_bytes(lsb: u8, msb: u8) -> u16 {
    u16::from_le_bytes([lsb, msb])
}

pub fn nth_bit<T: PrimInt>(bit: usize) -> T {
    T::one() << bit
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

pub fn toggle_bit<T: PrimInt>(val: T, bit: usize) -> T {
    val ^ nth_bit(bit)
}

pub fn get_bits<T: PrimInt>(val: T, range: Range<usize>) -> T {
    let width = range.end - range.start;
    let mask = (T::one() << width) - T::one();

    (val >> range.start) & mask
}

pub fn set_bits<T: PrimInt>(val: T, range: Range<usize>, to: T) -> T {
    let width = range.end - range.start;
    let mask = ((T::one() << width) - T::one()) << range.start;

    (val & !mask) | (get_bits(to, 0..width) << range.start)
}
