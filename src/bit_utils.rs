use num_traits::PrimInt;

pub const fn binary_bit(bit: usize) -> usize {
    1 << bit
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