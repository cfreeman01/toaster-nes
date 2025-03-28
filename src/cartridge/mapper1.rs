use super::*;
use crate::{bitfield::*, KB_16, KB_32, KB_4, KB_8};

const MSB: u8 = 0x80;
const LSB: u8 = 0x01;
const REG_WIDTH: u8 = 5;

#[derive(Copy, Clone)]
pub struct Ctrl {
    pub data: u8,
}

impl Ctrl {
    get_set_field!(nt_conf, set_nt_conf, 0, 2, u8);
    get_set_field!(prg_bank_mode, set_prg_bank_mode, 2, 2, u8);
    get_set_field!(chr_bank_mode, set_chr_bank_mode, 4, 1, u8);
}

impl Default for Ctrl {
    fn default() -> Self {
        Self { data: 0x0C }
    }
}

pub struct Mapper1 {
    shift_reg: u8,
    write_count: u8,
    ctrl: Ctrl,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,
}

impl Mapper for Mapper1 {
    fn write_reg(&mut self, addr: u16, data: u8, cart: &mut CartData) {
        if (data & MSB == MSB) {
            self.shift_reg = 0;
            self.write_count = 0;
            self.ctrl = Ctrl::default();
        } else {
            self.shift_reg >>= 1;
            self.shift_reg |= ((data & LSB) << (REG_WIDTH - 1));
            self.write_count += 1;

            if self.write_count == REG_WIDTH {
                match (addr >> 13) & 0x3 {
                    0 => self.ctrl.data = self.shift_reg,
                    1 => self.chr_bank_0 = self.shift_reg,
                    2 => self.chr_bank_1 = self.shift_reg,
                    3 => self.prg_bank = self.shift_reg,
                    _ => {}
                }

                *cart.nt_conf = match self.ctrl.nt_conf() {
                    0 => OneScreenLower,
                    1 => OneScreenUpper,
                    2 => Horizontal,
                    3 => Vertical,
                    _ => panic!(),
                };

                self.shift_reg = 0;
                self.write_count = 0;
            }
        }
    }

    fn map_prg(&self, addr: u16, cart: &mut CartData) -> usize {
        let prg_bank = self.prg_bank as usize;
        let offset = (addr as usize) % KB_32;

        (match self.ctrl.prg_bank_mode() {
            0 | 1 => ((prg_bank >> 1) * KB_32) + offset,
            2 => {
                if offset < KB_16 {
                    offset
                } else {
                    (prg_bank * KB_16) + (offset % KB_16)
                }
            }
            3 => {
                if offset < KB_16 {
                    (prg_bank * KB_16) + offset
                } else {
                    (cart.prg_rom_size - KB_16) + (offset % KB_16)
                }
            }
            _ => panic!(),
        }) % cart.prg_rom_size
    }

    fn map_chr(&self, addr: u16, cart: &mut CartData) -> usize {
        let chr_bank_0 = self.chr_bank_0 as usize;
        let chr_bank_1 = self.chr_bank_1 as usize;
        let offset = (addr as usize) % KB_8;

        (if self.ctrl.chr_bank_mode() == 0 {
            ((chr_bank_0 >> 1) * KB_8) + offset
        } else {
            if offset < KB_4 {
                (chr_bank_0 * KB_4) + offset
            } else {
                (chr_bank_1 * KB_4) + (offset % KB_4)
            }
        }) % cart.chr_size
    }
}

impl Mapper1 {
    pub fn init() -> Mapper1 {
        Mapper1 {
            shift_reg: 0,
            write_count: 0,
            ctrl: Ctrl::default(),
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
        }
    }
}
