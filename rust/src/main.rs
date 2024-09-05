use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{self, Read};

fn main() {
    println!("kilo start");
    enable_raw_mode().unwrap();

    for b in io::stdin().bytes() {
        match b {
            Ok(b) => {
                let c = b as char;
                if c.is_control() {
                    println!("Binary: {0:08b} ASCII: {0:#03} \r", b);
                } else {
                    println!("Binary: {0:08b} ASCII: {0:#03} Character: {1:#?}\r", b, c);
                }
                if c == 'q' {
                    break;
                }
            }
            Err(err) => println!("{}", err),
        }
    }

    disable_raw_mode().unwrap();
    println!("kilo end");
}
