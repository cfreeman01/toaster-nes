use super::*;

pub struct Mapper0 {}

impl Mapper for Mapper0 {
    fn write_reg(&mut self, addr: u16, data: u8, cart: &mut CartData) {}

    fn map_prg(&self, addr: u16, cart: &mut CartData) -> usize {
        (addr - PRG_ROM_START) as usize % cart.prg_rom_size
    }

    fn map_chr(&self, addr: u16, cart: &mut CartData) -> usize {
        addr as usize
    }
}

impl Mapper0 {
    pub fn init() -> Mapper0 {
        Mapper0 {}
    }
}
