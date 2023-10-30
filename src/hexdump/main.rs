use std::env;
use std::fs::File;
use std::io::Read;

const BYTES_PER_LINE: usize = 16;
fn main() {
    let arg = env::args().nth(1);
    let filename = arg.expect("Usage: hexdump <filename>");

    let mut file = File::open(&filename).expect("Error opening file");
    let mut pos = 0;
    let mut buf = [0u8; BYTES_PER_LINE];

    while let Ok(bytes_read) = file.read_exact(&mut buf) {
        print!("[0x{:08x}] ", pos);
        for byte in &buf {
            match *byte {
                0x00 => print!(".  "),
                0xff => print!("## "),
                _ => print!("{:02x} ", byte),
            }
        }

        println!("");
        pos += BYTES_PER_LINE;
    }
}