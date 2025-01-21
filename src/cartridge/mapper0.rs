use std::cell::RefCell;

use crate::cartridge::Cartridge;
use crate::rom::Rom;

const PRG_RAM_START: u16 = 0x6000;
const PRG_RAM_END: u16 = 0x7FFF;
const PRG_ROM_START: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xFFFF;
const PATTERN_START: u16 = 0x0000;
const PATTERN_END: u16 = 0x1FFF;
const VRAM_SIZE: usize = 0x800;

pub struct Mapper0 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr_ram: Vec<u8>,
}

impl Mapper0 {
    pub fn init(rom: &Rom) -> Mapper0 {
        Mapper0 {
            prg_rom: rom.prg_rom.clone(),
            chr_rom: rom.chr_rom.clone(),
            prg_ram: vec![0; rom.prg_ram_size as usize],
            chr_ram: vec![0; rom.chr_ram_size as usize],
        }
    }
}

impl Cartridge for Mapper0 {
    fn cpu_read(&mut self, addr: u16) -> Option<u8> {
        let (prg_ram_size, prg_rom_size) = (self.prg_ram.len(), self.prg_rom.len());

        match addr {
            PRG_RAM_START..=PRG_RAM_END => {
                if prg_ram_size == 0 {
                    None
                } else {
                    Some(self.prg_ram[(addr - PRG_RAM_START) as usize % prg_ram_size])
                }
            }
            PRG_ROM_START..=PRG_ROM_END => {
                Some(self.prg_rom[(addr - PRG_ROM_START) as usize % prg_rom_size])
            }
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8) {
        let prg_ram_size = self.prg_ram.len();

        match addr {
            PRG_RAM_START..=PRG_RAM_END => {
                if prg_ram_size != 0 {
                    self.prg_ram[(addr - PRG_RAM_START) as usize % prg_ram_size] = data;
                }
            }
            _ => (),
        }
    }

    fn ppu_read(&mut self, addr: u16) -> Option<u8> {
        match addr {
            PATTERN_START..=PATTERN_END => Some(self.chr_rom[addr as usize]),
            _ => None,
        }
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {}
}
