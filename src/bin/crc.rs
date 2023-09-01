use std::io::{stdin, BufRead};

use ebus::Crc;

fn main() {
    let mut stdin = stdin().lock();
    let mut s = String::new();
    println!("Generator polynom: ");
    stdin.read_line(&mut s).unwrap();
    let polynom: u8 = u8::from_str_radix(s.trim().strip_prefix("0x").unwrap(), 16).unwrap();
    println!("Using generator polynom: 0x{polynom:X}");

    s.clear();

    println!("Bytes (whitespace separated): ");
    stdin.read_line(&mut s).unwrap();

    let mut crc = Crc::new(polynom);
    s.split_ascii_whitespace()
        .map(|s| u8::from_str_radix(s.trim().strip_prefix("0x").unwrap(), 16).unwrap())
        .for_each(|b| {
            println!("byte 0x{b:X}");

            crc.add(b)
        });

    println!("0x{:X}", crc.calc_crc());
}
