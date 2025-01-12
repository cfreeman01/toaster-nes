use std::env;
use toaster_nes::*;

const WINDOW_TITLE: &str = "ToasterNES";
const WINDOW_SCALE: u32 = 3;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut nes = Nes::init();

    nes.step();
}
