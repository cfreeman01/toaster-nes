use std::env;
use std::fs;
use toaster_nes::rom::{rom_get_info, rom_parse};
use toaster_nes::*;

const WINDOW_TITLE: &str = "ToasterNES";
const WINDOW_SCALE: u32 = 3;

fn main() {
    let args: Vec<String> = env::args().collect();

    let rom_data = fs::read(&args[1]).unwrap();
    let rom = rom_parse(&rom_data).unwrap();

    println!("{}", rom_get_info(&rom));

    let mut nes = Nes::init(&rom);

    loop {
        nes.step();
    }
}
