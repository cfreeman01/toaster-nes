#[path = "mapper0.rs"]
pub mod mapper0;

#[path = "mapper2.rs"]
pub mod mapper2;

#[path = "mapper3.rs"]
pub mod mapper3;

use crate::rom::Rom;
use mapper0::Mapper0;
use mapper2::Mapper2;
use mapper3::Mapper3;

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

struct CartData<'a> {
    prg_rom_size: usize,
    chr_size: usize,
    vert_mirrored: &'a mut bool,
}

macro_rules! cart_data {
    ($cart:expr) => {
        &mut CartData {
            prg_rom_size: $cart.prg_rom.len(),
            chr_size: $cart.chr.len(),
            vert_mirrored: &mut $cart.vert_mirrored,
        }
    };
}

trait Mapper {
    fn write_reg(&mut self, addr: u16, data: u8, cart: &mut CartData) {}

    fn map_prg(&self, addr: u16, cart: &mut CartData) -> usize {
        (addr - PRG_ROM_START) as usize % cart.prg_rom_size
    }

    fn map_chr(&self, addr: u16, cart: &mut CartData) -> usize {
        addr as usize
    }
}

pub struct Cartridge {
    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr: Vec<u8>,
    chr_ram: bool,
    vert_mirrored: bool,
    vram: [u8; VRAM_SIZE],
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn init(rom: &Rom) -> Self {
        Self {
            prg_rom: rom.prg_rom.clone(),
            prg_ram: vec![0; rom.prg_ram_size as usize],
            chr: if rom.chr_ram_size == 0 {
                rom.chr_rom.clone()
            } else {
                vec![0; rom.chr_ram_size as usize]
            },
            chr_ram: rom.chr_ram_size > 0,
            vert_mirrored: rom.vert_mirrored,
            vram: [0x00; VRAM_SIZE],
            mapper: match rom.mapper {
                0 => Box::new(Mapper0::init()),
                2 => Box::new(Mapper2::init()),
                3 => Box::new(Mapper3::init()),
                _ => panic!("Invalid or unsupported mapper: {}", rom.mapper),
            },
        }
    }

    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            PRG_RAM_START..=PRG_RAM_END => {
                if self.prg_ram.len() == 0 {
                    0x00
                } else {
                    self.prg_ram[(addr - PRG_RAM_START) as usize % self.prg_ram.len()]
                }
            }
            PRG_ROM_START..=PRG_ROM_END => {
                self.prg_rom[self.mapper.map_prg(addr, cart_data!(self))]
            }
            _ => 0x00,
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        let prg_ram_size = self.prg_ram.len();

        match addr {
            PRG_RAM_START..=PRG_RAM_END => {
                if prg_ram_size != 0 {
                    self.prg_ram[(addr - PRG_RAM_START) as usize % prg_ram_size] = data;
                }
            }
            PRG_ROM_START..=PRG_ROM_END => self.mapper.write_reg(addr, data, cart_data!(self)),
            _ => (),
        }
    }

    pub fn ppu_read(&mut self, addr: u16) -> u8 {
        match addr {
            PATTERN_START..=PATTERN_END => self.chr[self.mapper.map_chr(addr, cart_data!(self))],
            NAMETABLE_0_START..=NAMETABLE_3_END => self.vram[vram_idx(addr, self.vert_mirrored)],
            _ => 0x00,
        }
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) {
        match addr {
            PATTERN_START..=PATTERN_END => {
                if self.chr_ram {
                    let cart_data = cart_data!(self);
                    self.chr[self.mapper.map_chr(addr, cart_data)] = data;
                }
            }
            NAMETABLE_0_START..=NAMETABLE_3_END => {
                self.vram[vram_idx(addr, self.vert_mirrored)] = data
            }
            _ => (),
        }
    }
}

fn vram_idx(addr: u16, vert_mirrored: bool) -> usize {
    let addr = (addr - NAMETABLE_0_START) as usize;

    if vert_mirrored {
        addr % VRAM_SIZE
    } else {
        (addr % NAMETABLE_SIZE as usize) + ((addr / VRAM_SIZE) * NAMETABLE_SIZE as usize)
    }
}
