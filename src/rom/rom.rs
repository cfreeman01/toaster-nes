const HDR_SIZE: usize = 16;
const TRAINER_SIZE: usize = 512;
use crate::{KB_16, KB_8};

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub prg_ram_size: u16,
    pub chr_ram_size: u16,
    pub vert_mirrored: bool,
}

pub fn rom_parse(data: &[u8]) -> Result<Rom, String> {
    let ines_2 = (data[7] & 0x0C) == 0x08;
    let prg_rom_size = data[4] as usize * KB_16;
    let chr_rom_size = data[5] as usize * KB_8;
    let trainer_present = (data[6] & 0x04) == 0x04;
    let prg_rom_offset = HDR_SIZE + if trainer_present { TRAINER_SIZE } else { 0 };
    let chr_rom_offset = prg_rom_offset + prg_rom_size;
    let mapper: u8 = (data[6] >> 4) | (data[7] & 0xF0);
    let vert_mirrored = (data[6] & 0x01) == 1;

    let prg_ram_size: u16 = if ines_2 {
        let shift_count = data[10] & 0x0F;
        if shift_count == 0 {
            0
        } else {
            0x40 << shift_count
        }
    } else {
        KB_8 as u16
    };

    let chr_ram_size: u16 = if ines_2 {
        let shift_count = data[11] & 0x0F;
        if shift_count == 0 {
            0
        } else {
            0x40 << shift_count
        }
    } else if chr_rom_size == 0 {
        KB_8 as u16
    } else {
        0
    };

    let mut prg_rom: Vec<u8> = vec![0; prg_rom_size];
    let mut chr_rom: Vec<u8> = vec![0; chr_rom_size];
    prg_rom.copy_from_slice(&data[prg_rom_offset..prg_rom_offset + prg_rom_size]);
    chr_rom.copy_from_slice(&data[chr_rom_offset..chr_rom_offset + chr_rom_size]);

    Ok(Rom {
        prg_rom,
        chr_rom,
        mapper,
        prg_ram_size,
        chr_ram_size,
        vert_mirrored,
    })
}

pub fn rom_get_info(rom: &Rom) -> String {
    format!(
        "PRG ROM Size: {}\
        \nCHR ROM Size: {}\
        \nPRG RAM Size: {}\
        \nCHR RAM Size: {}\
        \nMapper:       {}",
        rom.prg_rom.len(),
        rom.chr_rom.len(),
        rom.prg_ram_size,
        rom.chr_ram_size,
        rom.mapper
    )
}
