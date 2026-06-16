use crate::bit_utils::bit_set;

pub mod bit_utils;

pub mod cpu;

fn main() {
    let x = 0b1010101010;

    println!("{:b}", x);
    for i in 0..10 {
        // Print MSB first.
        print!("{}", if bit_set(x, 9 - i) { "1" } else { "0" });
    }
}
