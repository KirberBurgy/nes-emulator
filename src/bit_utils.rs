use std::ops::Range;

use num_traits::PrimInt;

pub const fn address_from_bytes(lsb: u8, msb: u8) -> u16 {
    u16::from_le_bytes([lsb, msb])
}

pub fn nth_bit<T: PrimInt>(bit: usize) -> T {
    T::one() << bit
}

pub fn bits_mask<T: PrimInt>(bits: Range<usize>) -> T {
    let width = bits.end - bits.start;
    let mask = (T::one() << width) - T::one();
    
    mask << bits.start
}

pub fn bits_ranges_mask<T: PrimInt>(ranges: &[Range<usize>]) -> T {
    ranges.iter().map(|range| bits_mask::<T>(range.clone())).fold(T::zero(), |a, b| a | b)
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
    val & bits_mask(range.clone())
}

pub fn extract_bits_ranges<T: PrimInt>(val: T, ranges: &[Range<usize>]) -> T {
    val & bits_ranges_mask(ranges)
}

pub fn set_bits<T: PrimInt>(val: T, range: Range<usize>, to: T) -> T {
    let width = range.end - range.start;

    (val & !bits_mask::<T>(range.clone())) | (get_bits(to, 0..width) << range.start)
}

pub fn copy_bit<T: PrimInt>(to: T, from: T, bit: usize) -> T {
    set_bit(to, bit, bit_set(from, bit))
}

pub fn copy_bits<T: PrimInt>(to: T, from: T, range: Range<usize>) -> T {
    set_bits(to, range.clone(), get_bits(from, range))
}

pub fn copy_bit_ranges<T: PrimInt>(to: T, from: T, ranges: &[Range<usize>]) -> T {
    let mask = bits_ranges_mask::<T>(ranges);

    (to & !mask) | (from & mask)
}