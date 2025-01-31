#[path = "media/media.rs"]
pub mod media;

use media::{Media, MediaEvent::*};
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

    let mut media = Media::init(WINDOW_TITLE, DISPLAY_WIDTH, DISPLAY_HEIGHT, WINDOW_SCALE);

    let mut frame = [0; FRAME_SIZE_BYTES];

    loop {
        if let Some(event) = media.poll_event() {
            match event {
                Quit => break,
                _ => (),
            }
        }

        nes.step(&mut frame);
        media.render(&frame);
    }
}
