use super::*;
use crate::{KB_16, KB_8};

pub struct Mapper3 {
    chr_offset: usize,
}

impl Mapper for Mapper3 {
    fn write_reg(&mut self, addr: u16, data: u8, cart: &mut CartData) {
        self.chr_offset = ((data as usize) * KB_8) % cart.chr_size
    }

    fn map_chr(&self, addr: u16, cart: &mut CartData) -> usize {
        (addr as usize) + self.chr_offset
    }
}

impl Mapper3 {
    pub fn init() -> Mapper3 {
        Mapper3 { chr_offset: 0 }
    }
}
