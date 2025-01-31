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
const NAMETABLE_SIZE: u16 = (VRAM_SIZE / 2) as u16;
const NAMETABLE_0_START: u16 = 0x2000;
const NAMETABLE_0_END: u16 = NAMETABLE_0_START + (NAMETABLE_SIZE - 1);
const NAMETABLE_1_START: u16 = NAMETABLE_0_END + 1;
const NAMETABLE_1_END: u16 = NAMETABLE_1_START + (NAMETABLE_SIZE - 1);
const NAMETABLE_2_START: u16 = NAMETABLE_1_END + 1;
const NAMETABLE_2_END: u16 = NAMETABLE_2_START + (NAMETABLE_SIZE - 1);
const NAMETABLE_3_START: u16 = NAMETABLE_2_END + 1;
const NAMETABLE_3_END: u16 = NAMETABLE_3_START + (NAMETABLE_SIZE - 1);

pub struct Mapper0 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr_ram: Vec<u8>,
    vert_mirrored: bool,
    vram: [u8; VRAM_SIZE],
}

impl Mapper0 {
    pub fn init(rom: &Rom) -> Mapper0 {
        Mapper0 {
            prg_rom: rom.prg_rom.clone(),
            chr_rom: rom.chr_rom.clone(),
            prg_ram: vec![0; rom.prg_ram_size as usize],
            chr_ram: vec![0; rom.chr_ram_size as usize],
            vert_mirrored: rom.vert_mirrored,
            vram: [0x00; VRAM_SIZE],
        }
    }
}

impl Cartridge for Mapper0 {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        let (prg_ram_size, prg_rom_size) = (self.prg_ram.len(), self.prg_rom.len());

        match addr {
            PRG_RAM_START..=PRG_RAM_END => {
                if prg_ram_size == 0 {
                    0x00
                } else {
                    self.prg_ram[(addr - PRG_RAM_START) as usize % prg_ram_size]
                }
            }
            PRG_ROM_START..=PRG_ROM_END => {
                self.prg_rom[(addr - PRG_ROM_START) as usize % prg_rom_size]
            }
            _ => 0x00,
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

    fn ppu_read(&mut self, addr: u16) -> u8 {
        match addr {
            PATTERN_START..=PATTERN_END => self.chr_rom[addr as usize],
            NAMETABLE_0_START..=NAMETABLE_3_END => self.vram[vram_idx(addr, self.vert_mirrored)],
            _ => 0x00,
        }
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
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
