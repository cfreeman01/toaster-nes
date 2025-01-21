#[path = "mapper0.rs"]
pub mod mapper0;

use crate::rom::Rom;
use mapper0::Mapper0;

pub trait Cartridge {
    fn cpu_read(&mut self, addr: u16) -> Option<u8>;
    fn cpu_write(&mut self, addr: u16, data: u8);
    fn ppu_read(&mut self, addr: u16) -> Option<u8>;
    fn ppu_write(&mut self, addr: u16, data: u8);
}

pub fn cart_init(rom: &Rom) -> Box<dyn Cartridge> {
    match rom.mapper {
        0 => Box::new(Mapper0::init(rom)),
        _ => panic!("Invalid or unsupported mapper: {}", rom.mapper),
    }
}
