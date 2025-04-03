use super::*;
use crate::{bitfield::*, KB_1, KB_16, KB_2, KB_32, KB_4, KB_8};

#[derive(Copy, Clone, Default)]
pub struct BankSelect {
    pub data: u8,
}

impl BankSelect {
    get_set_field!(next_bank, set_next_bank, 0, 3, u8);
    get_set_field!(prg_mode, set_prg_mode, 6, 1, u8);
    get_set_field!(chr_mode, set_chr_mode, 7, 1, u8);
}

pub struct Mapper4 {
    bank_select: BankSelect,
    chr_1kb_bank_0: u8,
    chr_1kb_bank_1: u8,
    chr_1kb_bank_2: u8,
    chr_1kb_bank_3: u8,
    chr_2kb_bank_0: u8,
    chr_2kb_bank_1: u8,
    prg_bank_0: u8,
    prg_bank_1: u8,
    irq_latch: u8,
    irq_counter: u8,
    irq_reset: bool,
    irq_enable: bool,
    a12_prev: bool,
    irq_delay_counter: u32,
}

impl Mapper for Mapper4 {
    fn write_reg(&mut self, addr: u16, data: u8, cart: &mut CartData) {
        match ((addr % PRG_ROM_START) / KB_8 as u16, addr % 2) {
            (0, 0) => self.bank_select.data = data,
            (0, 1) => match self.bank_select.next_bank() {
                0 => self.chr_2kb_bank_0 = data,
                1 => self.chr_2kb_bank_1 = data,
                2 => self.chr_1kb_bank_0 = data,
                3 => self.chr_1kb_bank_1 = data,
                4 => self.chr_1kb_bank_2 = data,
                5 => self.chr_1kb_bank_3 = data,
                6 => self.prg_bank_0 = data,
                7 => self.prg_bank_1 = data,
                _ => {}
            },
            (1, 0) => {
                *cart.nt_conf = if data & 0x1 == 0x1 {
                    Horizontal
                } else {
                    Vertical
                }
            }
            (2, 0) => self.irq_latch = data,
            (2, 1) => self.irq_reset = true,
            (3, 0) => {
                self.irq_enable = false;
                *cart.irq = false;
            }
            (3, 1) => self.irq_enable = true,
            _ => {}
        }
    }

    fn map_prg(&mut self, addr: u16, cart: &mut CartData) -> usize {
        let prg_bank_0 = self.prg_bank_0 as usize;
        let prg_bank_1 = self.prg_bank_1 as usize;
        let prg_bank_last = (cart.prg_rom_size / KB_8) - 1;
        let prg_bank_second_last = prg_bank_last - 1;
        let offset = (addr as usize) % KB_8;
        let base = match (addr % PRG_ROM_START) / KB_8 as u16 {
            0 => {
                if self.bank_select.prg_mode() == 0 {
                    prg_bank_0
                } else {
                    prg_bank_second_last
                }
            }
            1 => prg_bank_1,
            2 => {
                if self.bank_select.prg_mode() == 0 {
                    prg_bank_second_last
                } else {
                    prg_bank_0
                }
            }
            3 => prg_bank_last,
            _ => panic!(),
        } * KB_8;

        (base + offset) % cart.prg_rom_size
    }

    fn map_chr(&mut self, addr: u16, cart: &mut CartData) -> usize {
        self.update_irq(addr, cart.irq);
        let addr = addr as usize;
        let offset = addr % KB_4;
        let chr_1kb_bank_0 = (self.chr_1kb_bank_0 as usize, KB_1);
        let chr_1kb_bank_1 = (self.chr_1kb_bank_1 as usize, KB_1);
        let chr_1kb_bank_2 = (self.chr_1kb_bank_2 as usize, KB_1);
        let chr_1kb_bank_3 = (self.chr_1kb_bank_3 as usize, KB_1);
        let chr_2kb_bank_0 = ((self.chr_2kb_bank_0 >> 1) as usize, KB_2);
        let chr_2kb_bank_1 = ((self.chr_2kb_bank_1 >> 1) as usize, KB_2);

        let (base, size) = if ((addr / KB_4) == 0 && self.bank_select.chr_mode() == 0)
            || ((addr / KB_4) == 1 && self.bank_select.chr_mode() == 1)
        {
            if (offset / KB_2) == 0 {
                chr_2kb_bank_0
            } else {
                chr_2kb_bank_1
            }
        } else if ((addr / KB_4) == 0 && self.bank_select.chr_mode() == 1)
            || ((addr / KB_4) == 1 && self.bank_select.chr_mode() == 0)
        {
            match offset / KB_1 {
                0 => chr_1kb_bank_0,
                1 => chr_1kb_bank_1,
                2 => chr_1kb_bank_2,
                3 => chr_1kb_bank_3,
                _ => panic!(),
            }
        } else {
            panic!()
        };

        ((base * size) + (addr % size)) % cart.chr_size
    }

    fn tick(&mut self) {
        self.irq_delay_counter += 1
    }
}

impl Mapper4 {
    pub fn init() -> Mapper4 {
        Mapper4 {
            bank_select: BankSelect::default(),
            chr_1kb_bank_0: 0,
            chr_1kb_bank_1: 0,
            chr_1kb_bank_2: 0,
            chr_1kb_bank_3: 0,
            chr_2kb_bank_0: 0,
            chr_2kb_bank_1: 0,
            prg_bank_0: 0,
            prg_bank_1: 0,
            irq_latch: 0,
            irq_counter: 0,
            irq_reset: false,
            irq_enable: false,
            a12_prev: false,
            irq_delay_counter: 0,
        }
    }

    fn update_irq(&mut self, addr: u16, irq: &mut bool) {
        let a12 = addr & 0x1000 == 0x1000;

        if !a12 && self.a12_prev {
            self.irq_delay_counter = 0;
        } else if a12 && !self.a12_prev && self.irq_delay_counter > 3 {
            if self.irq_reset {
                self.irq_reset = false;
                self.irq_counter = self.irq_latch;
            } else {
                if self.irq_counter == 0 {
                    self.irq_counter = self.irq_latch;
                } else {
                    self.irq_counter -= 1;
                }

                if self.irq_counter == 0 && self.irq_enable {
                    *irq = true;
                }
            }
        }

        self.a12_prev = a12;
    }
}
