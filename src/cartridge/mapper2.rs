use super::*;
use crate::{KB_16, KB_8};

pub struct Mapper2 {
    prg_offset: usize,
}

impl Mapper for Mapper2 {
    fn write_reg(&mut self, addr: u16, data: u8, cart: &mut CartData) {
        self.prg_offset = ((data as usize) * KB_16) % cart.prg_rom_size
    }

    fn map_prg(&mut self, addr: u16, cart: &mut CartData) -> usize {
        let bank = (addr - PRG_ROM_START) as usize / KB_16;
        let mut offset = (addr - PRG_ROM_START) as usize % KB_16;

        offset += if bank == 0 {
            self.prg_offset
        } else if bank == 1 {
            cart.prg_rom_size - KB_16
        } else {
            panic!("mapper 2: invalid address {:04X}", bank);
        };

        offset
    }
}

impl Mapper2 {
    pub fn init() -> Mapper2 {
        Mapper2 { prg_offset: 0 }
    }
}
