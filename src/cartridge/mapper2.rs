use super::*;
use crate::{KB_16, KB_8};

pub struct Mapper2 {
    bank_0_offset: usize,
}

impl Mapper for Mapper2 {
    fn write_reg(&mut self, addr: u16, data: u8, cart: &mut CartData) {
        self.bank_0_offset = (data as usize) * KB_16
    }

    fn map_prg(&self, addr: u16, cart: &mut CartData) -> usize {
        let bank = (addr - PRG_ROM_START) as usize / KB_16;
        let mut offset = (addr - PRG_ROM_START) as usize % KB_16;

        offset += if bank == 0 {
            self.bank_0_offset
        } else if bank == 1 {
            cart.prg_rom_size - KB_16
        } else {
            panic!("mapper 2: invalid address {:04X}", bank);
        };

        offset
    }

    fn map_chr(&self, addr: u16, cart: &mut CartData) -> usize {
        addr as usize
    }
}

impl Mapper2 {
    pub fn init() -> Mapper2 {
        Mapper2 { bank_0_offset: 0 }
    }
}
