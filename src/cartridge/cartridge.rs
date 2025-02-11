#[path = "mapper0.rs"]
pub mod mapper0;

use crate::rom::Rom;
use mapper0::Mapper0;

pub const PRG_RAM_START: u16 = 0x6000;
pub const PRG_RAM_END: u16 = 0x7FFF;
pub const PRG_ROM_START: u16 = 0x8000;
pub const PRG_ROM_END: u16 = 0xFFFF;
pub const PATTERN_START: u16 = 0x0000;
pub const PATTERN_END: u16 = 0x1FFF;
pub const VRAM_SIZE: usize = 0x800;
pub const NAMETABLE_SIZE: u16 = (VRAM_SIZE / 2) as u16;
pub const NAMETABLE_0_START: u16 = 0x2000;
pub const NAMETABLE_0_END: u16 = NAMETABLE_0_START + (NAMETABLE_SIZE - 1);
pub const NAMETABLE_1_START: u16 = NAMETABLE_0_END + 1;
pub const NAMETABLE_1_END: u16 = NAMETABLE_1_START + (NAMETABLE_SIZE - 1);
pub const NAMETABLE_2_START: u16 = NAMETABLE_1_END + 1;
pub const NAMETABLE_2_END: u16 = NAMETABLE_2_START + (NAMETABLE_SIZE - 1);
pub const NAMETABLE_3_START: u16 = NAMETABLE_2_END + 1;
pub const NAMETABLE_3_END: u16 = NAMETABLE_3_START + (NAMETABLE_SIZE - 1);

pub trait Cartridge {
    fn cpu_read(&mut self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, data: u8);
    fn ppu_read(&mut self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, data: u8);
}

pub fn cart_init(rom: &Rom) -> Box<dyn Cartridge> {
    match rom.mapper {
        0 => Box::new(Mapper0::init(rom)),
        _ => panic!("Invalid or unsupported mapper: {}", rom.mapper),
    }
}
