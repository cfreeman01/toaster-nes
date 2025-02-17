#[path = "window/window.rs"]
pub mod window;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::{env, fs};
use toaster_nes::rom::{rom_get_info, rom_parse};
use toaster_nes::*;
use window::*;

const WINDOW_TITLE: &str = "ToasterNES";
const WINDOW_SCALE: u32 = 3;

lazy_static! {
    static ref KEY_BINDS: HashMap<Key, Button> = [
        (Key::W, Button::Up),
        (Key::A, Button::Left),
        (Key::S, Button::Down),
        (Key::D, Button::Right),
        (Key::Q, Button::Select),
        (Key::E, Button::Start),
        (Key::L, Button::A),
        (Key::K, Button::B)
    ]
    .iter()
    .cloned()
    .collect();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let rom_data = fs::read(&args[1]).unwrap();
    let rom = rom_parse(&rom_data).unwrap();

    println!("{}", rom_get_info(&rom));

    let mut nes = Nes::init(&rom);

    let mut window = Window::init(WINDOW_TITLE, DISPLAY_WIDTH, DISPLAY_HEIGHT, WINDOW_SCALE);

    let mut frame = [0; FRAME_SIZE_BYTES];

    while !window.closed() {
        nes.frame(&mut frame);

        window.poll_events();

        for (key, pressed) in window.get_key_events() {
            if let Some(&button) = KEY_BINDS.get(&key) {
                nes.set_button_state(button, pressed)
            }
        }

        window.render(&frame);
    }
}
