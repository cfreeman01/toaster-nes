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

trait Mapper {
    fn write_reg(&mut self, addr: u16, cart: &mut CartData);
    fn map_prg_rom(&self, addr: u16, cart: &mut CartData) -> usize;
    fn map_chr_rom(&self, addr: u16, cart: &mut CartData) -> usize;
}

struct CartData<'a> {
    prg_rom_size: usize,
    chr_rom_size: usize,
    vert_mirrored: &'a mut bool,
}

macro_rules! cart_data {
    ($cart:expr) => {
        &mut CartData {
            prg_rom_size: $cart.prg_rom.len(),
            chr_rom_size: $cart.chr_rom.len(),
            vert_mirrored: &mut $cart.vert_mirrored,
        }
    };
}

pub struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    vert_mirrored: bool,
    prg_ram: Vec<u8>,
    chr_ram: Vec<u8>,
    vram: [u8; VRAM_SIZE],
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn init(rom: &Rom) -> Self {
        Self {
            prg_rom: rom.prg_rom.clone(),
            chr_rom: rom.chr_rom.clone(),
            vert_mirrored: rom.vert_mirrored,
            prg_ram: vec![0; rom.prg_ram_size as usize],
            chr_ram: vec![0; rom.chr_ram_size as usize],
            vram: [0x00; VRAM_SIZE],
            mapper: Box::new(match rom.mapper {
                0 => Mapper0::init(),
                _ => panic!("Invalid or unsupported mapper: {}", rom.mapper),
            }),
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
                self.prg_rom[self.mapper.map_prg_rom(addr, cart_data!(self))]
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
            PRG_ROM_START..=PRG_ROM_END => self.mapper.write_reg(addr, cart_data!(self)),
            _ => (),
        }
    }

    pub fn ppu_read(&mut self, addr: u16) -> u8 {
        match addr {
            PATTERN_START..=PATTERN_END => {
                self.chr_rom[self.mapper.map_chr_rom(addr, cart_data!(self))]
            }
            NAMETABLE_0_START..=NAMETABLE_3_END => self.vram[vram_idx(addr, self.vert_mirrored)],
            _ => 0x00,
        }
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) {
        match addr {
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
