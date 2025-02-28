mod ppu_regs;

mod ppu_palette;

#[cfg(test)]
mod test;

use crate::cartridge::NAMETABLE_0_START;
use crate::{DISPLAY_HEIGHT, DISPLAY_WIDTH, FRAME_SIZE_BYTES, PPU_REG_END, PPU_REG_START};
use ppu_palette::*;
use ppu_regs::*;

pub const ROW_SIZE: u32 = 341;
pub const NUM_ROWS: u32 = 262;
pub const CYCLES_PER_FRAME: u32 = ROW_SIZE * NUM_ROWS;
const PPU_CTRL: u16 = 0;
const PPU_MASK: u16 = 1;
const PPU_STATUS: u16 = 2;
pub const OAM_ADDR: u16 = 3;
pub const OAM_DATA: u16 = 4;
const PPU_SCROLL: u16 = 5;
const PPU_ADDR: u16 = 6;
const PPU_DATA: u16 = 7;
const PALETTE_RAM_SIZE: usize = 32;
const PRE_FETCH_START: u32 = 321;
const PRE_FETCH_END: u32 = 336;
const ATTR_TABLE_OFFSET: u32 = 0x23C0;
const PALETTE_START: u16 = 0x3F00;
pub const OAM_SIZE: usize = 256;
const SPRITES_PER_ROW: usize = 8;

macro_rules! field {
    ($val:expr, $pos:expr, $width:expr) => {{
        let mut mask = 0;
        for _ in 0..$width {
            mask <<= 1;
            mask |= 0x1;
        }
        ($val >> $pos) & mask
    }};
}

#[derive(Copy, Clone)]
struct SpriteInfo {
    x_pos: u8,
    y_pos: u8,
    pattern_0: u8,
    pattern_1: u8,
    attr: SpriteAttr,
    sprite_0: bool,
}

impl Default for SpriteInfo {
    fn default() -> Self {
        Self {
            x_pos: 0xFF,
            y_pos: 0xFF,
            pattern_0: 0xFF,
            pattern_1: 0xFF,
            attr: SpriteAttr { data: 0xff },
            sprite_0: false,
        }
    }
}

pub struct Ppu {
    ctrl: PpuCtrl,
    mask: PpuMask,
    status: PpuStatus,
    v: VramAddr,
    t: VramAddr,
    w: bool,
    x: u8,
    read_buf: u8,
    palette_ram: [u8; PALETTE_RAM_SIZE],
    nmi: bool,
    bg_shift_reg_0: u16,
    bg_shift_reg_1: u16,
    attr_shift_reg_0: u16,
    attr_shift_reg_1: u16,
    tile_num: u8,
    attr_byte: u8,
    bg_byte_0: u8,
    bg_byte_1: u8,
    oam: [u8; OAM_SIZE],
    sprites: Vec<SpriteInfo>,
    oam_addr: u8,
    cycles: u32,
    frame_cycle: u32,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            ctrl: Default::default(),
            mask: Default::default(),
            status: Default::default(),
            v: Default::default(),
            t: Default::default(),
            w: Default::default(),
            x: Default::default(),
            read_buf: Default::default(),
            palette_ram: Default::default(),
            nmi: Default::default(),
            bg_shift_reg_0: Default::default(),
            bg_shift_reg_1: Default::default(),
            attr_shift_reg_0: Default::default(),
            attr_shift_reg_1: Default::default(),
            tile_num: Default::default(),
            attr_byte: Default::default(),
            bg_byte_0: Default::default(),
            bg_byte_1: Default::default(),
            oam: [0xFF; OAM_SIZE],
            sprites: vec![],
            oam_addr: Default::default(),
            cycles: Default::default(),
            frame_cycle: Default::default(),
        }
    }
}

pub trait PpuBus {
    fn ppu_read(&mut self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, data: u8);
}

impl Ppu {
    pub fn tick(&mut self, bus: &mut impl PpuBus, frame: &mut [u8; FRAME_SIZE_BYTES]) {
        let (row, col) = (self.frame_cycle / ROW_SIZE, self.frame_cycle % ROW_SIZE);

        if row < DISPLAY_HEIGHT || row == NUM_ROWS - 1 {
            if (col >= 1 && col <= DISPLAY_WIDTH)
                || (col >= PRE_FETCH_START && col <= PRE_FETCH_END)
            {
                if self.bg_enabled() {
                    self.update_shift_regs();
                }

                match (col - 1) % 8 {
                    0 => {
                        self.load_shift_regs();
                        self.fetch_bg_tile_num(bus);
                    }
                    2 => self.fetch_bg_attr(bus),
                    4 => self.fetch_bg_pattern(bus, 0),
                    6 => self.fetch_bg_pattern(bus, 1),
                    7 => {
                        if self.bg_enabled() {
                            self.inc_v_hor()
                        }
                    }
                    _ => (),
                };
            };

            if col == DISPLAY_WIDTH && self.rendering_enabled() {
                self.inc_v_ver();
            }

            if col == DISPLAY_WIDTH + 1 && self.rendering_enabled() {
                self.load_shift_regs();
                self.v.set_coarse_x(self.t.coarse_x());
                self.v.set_nx(self.t.nx());
            }
        }

        if col == DISPLAY_WIDTH + 1 {
            self.sprites.clear();

            if row < DISPLAY_HEIGHT {
                self.sprite_eval(row, bus);
            }
        }

        if row == DISPLAY_HEIGHT + 1 && col == 1 {
            self.status.set_v(1)
        }

        if row == NUM_ROWS - 1 && col == 1 {
            self.status.set_v(0);
            self.status.set_s(0);
            self.status.set_o(0);
        }

        if row == NUM_ROWS - 1 && self.rendering_enabled() {
            self.v.set_coarse_y(self.t.coarse_y());
            self.v.set_fine_y(self.t.fine_y());
            self.v.set_ny(self.t.ny());
        }

        if row < DISPLAY_HEIGHT && (col - 1) < DISPLAY_WIDTH {
            self.draw_pixel(col - 1, row, frame)
        }

        self.nmi = ((self.status.v() & self.ctrl.v()) == 1);
        self.frame_cycle = (self.frame_cycle + 1) % CYCLES_PER_FRAME;
        self.cycles += 1;
    }

    pub fn cpu_read(&mut self, addr: u16, bus: &mut impl PpuBus) -> u8 {
        match addr % 8 {
            PPU_STATUS => {
                let val = self.status.data;
                self.status.set_v(0);
                self.w = false;
                val
            }
            OAM_DATA => self.oam[self.oam_addr as usize],
            PPU_DATA => {
                if self.v.addr() >= PALETTE_START {
                    self.read_buf = self.palette_ram[get_palette_addr(self.v.addr())];
                    self.read_buf
                } else {
                    let val = self.read_buf;
                    self.read_buf = bus.ppu_read(self.v.addr());
                    self.v.data += if self.ctrl.i() == 1 { 32 } else { 1 };
                    val
                }
            }
            _ => 0x0,
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8, bus: &mut impl PpuBus) {
        match addr % 8 {
            PPU_CTRL => {
                self.ctrl.data = data;
                self.t.set_n(field!(data, 0, 2) as u16);
            }
            PPU_MASK => {
                self.mask.data = data;
            }
            OAM_ADDR => {
                self.oam_addr = data;
            }
            OAM_DATA => {
                self.oam[self.oam_addr as usize] = data;
                self.oam_addr += 1;
            }
            PPU_SCROLL => {
                if !self.w {
                    self.x = field!(data, 0, 3);
                    self.t.set_coarse_x(field!(data, 3, 5) as u16);
                    self.w = true;
                } else {
                    self.t.set_fine_y(field!(data, 0, 3) as u16);
                    self.t.set_coarse_y(field!(data, 3, 5) as u16);
                    self.w = false;
                }
            }
            PPU_ADDR => {
                if !self.w {
                    self.t.set_addr_hi(field!(data, 0, 6) as u16);
                    self.w = true;
                } else {
                    self.t.set_addr_low(data as u16);
                    self.v.data = self.t.data;
                    self.w = false;
                }
            }
            PPU_DATA => {
                if self.v.addr() >= PALETTE_START {
                    self.palette_ram[get_palette_addr(self.v.addr())] = data;
                } else {
                    bus.ppu_write(self.v.addr(), data);
                }
                self.v.data += if self.ctrl.i() == 1 { 32 } else { 1 }
            }
            _ => (),
        }
    }

    pub fn cycles(&self) -> u32 {
        self.cycles
    }

    pub fn nmi(&self) -> bool {
        self.nmi
    }

    fn fetch_bg_tile_num(&mut self, bus: &mut impl PpuBus) {
        self.tile_num = bus.ppu_read(NAMETABLE_0_START | self.v.nt_addr());
    }

    fn fetch_bg_attr(&mut self, bus: &mut impl PpuBus) {
        let mut attr_addr = AttrAddr::default();
        attr_addr.set_tile_group_x(self.v.coarse_x() / 4);
        attr_addr.set_tile_group_y(self.v.coarse_y() / 4);
        attr_addr.set_n(self.v.n());
        self.attr_byte = bus.ppu_read(attr_addr.data());

        let tile_group_right = (self.v.coarse_x() % 4 > 1) as u8;
        let tile_group_bottom = (self.v.coarse_y() % 4 > 1) as u8;
        let shift_amt = ((tile_group_bottom << 1) | tile_group_right) * 2;
        self.attr_byte >>= shift_amt;
    }

    fn fetch_bg_pattern(&mut self, bus: &mut impl PpuBus, plane: u8) {
        let mut pattern_addr = PatternAddr::default();
        pattern_addr.set_fine_y(self.v.fine_y());
        pattern_addr.set_p(plane as u16);
        pattern_addr.set_tile(self.tile_num as u16);
        pattern_addr.set_h(self.ctrl.b() as u16);

        if plane & 0x1 == 0 {
            self.bg_byte_0 = bus.ppu_read(pattern_addr.data);
        } else {
            self.bg_byte_1 = bus.ppu_read(pattern_addr.data);
        }
    }

    fn update_shift_regs(&mut self) {
        self.bg_shift_reg_0 <<= 1;
        self.bg_shift_reg_1 <<= 1;
        self.attr_shift_reg_0 <<= 1;
        self.attr_shift_reg_1 <<= 1;
    }

    fn load_shift_regs(&mut self) {
        load_shift_reg(&mut self.bg_shift_reg_0, self.bg_byte_0);
        load_shift_reg(&mut self.bg_shift_reg_1, self.bg_byte_1);
        load_shift_reg(
            &mut self.attr_shift_reg_0,
            if field!(self.attr_byte, 0, 1) == 1 {
                0xFF
            } else {
                0x00
            },
        );
        load_shift_reg(
            &mut self.attr_shift_reg_1,
            if field!(self.attr_byte, 1, 1) == 1 {
                0xFF
            } else {
                0x00
            },
        );
    }

    fn inc_v_hor(&mut self) {
        let mut coarse_x = self.v.coarse_x();
        let mut nx = self.v.nx();

        if coarse_x == 31 {
            coarse_x = 0;
            nx = !nx;
        } else {
            coarse_x += 1;
        }

        self.v.set_coarse_x(coarse_x);
        self.v.set_nx(nx);
    }

    fn inc_v_ver(&mut self) {
        let mut coarse_y = self.v.coarse_y();
        let mut fine_y = self.v.fine_y();
        let mut ny = self.v.ny();

        if (fine_y < 7) {
            fine_y += 1;
        } else {
            fine_y = 0;
            if coarse_y == 29 {
                coarse_y = 0;
                ny = !ny;
            } else if coarse_y == 31 {
                coarse_y = 0;
            } else {
                coarse_y += 1;
            }
        }

        self.v.set_coarse_y(coarse_y);
        self.v.set_fine_y(fine_y);
        self.v.set_ny(ny);
    }

    fn draw_pixel(&mut self, x: u32, y: u32, frame: &mut [u8; FRAME_SIZE_BYTES]) {
        let frame_idx = ((y * DISPLAY_WIDTH + x) * 3) as usize;
        let bg_addr = self.get_bg_pixel_info();
        let bg_pixel = self.get_color(bg_addr);
        let (sprite_info, sprite_addr) = self.get_sprite_pixel_info(x);
        let sprite_pixel = self.get_color(sprite_addr);
        let transparent_pixel = self.get_color(PaletteAddr { data: 0 });

        if is_opaque(bg_addr)
            && is_opaque(sprite_addr)
            && sprite_info.sprite_0
            && self.rendering_enabled()
        {
            self.status.set_s(1)
        }

        Rgb(frame[frame_idx], frame[frame_idx + 1], frame[frame_idx + 2]) = match (
            sprite_info.attr.priority() == 0,
            is_opaque(sprite_addr),
            is_opaque(bg_addr),
        ) {
            (false, false, false) => transparent_pixel,
            (false, false, true) => bg_pixel,
            (false, true, false) => sprite_pixel,
            (false, true, true) => bg_pixel,
            (true, false, false) => transparent_pixel,
            (true, false, true) => bg_pixel,
            (true, true, _) => sprite_pixel,
        }
    }

    fn get_bg_pixel_info(&mut self) -> PaletteAddr {
        let mut palette_addr = PaletteAddr::default();
        palette_addr.set_p0(self.fine_x(self.bg_shift_reg_0));
        palette_addr.set_p1(self.fine_x(self.bg_shift_reg_1));
        palette_addr.set_a0(self.fine_x(self.attr_shift_reg_0));
        palette_addr.set_a1(self.fine_x(self.attr_shift_reg_1));

        palette_addr
    }

    fn get_sprite_pixel_info(&mut self, x: u32) -> (SpriteInfo, PaletteAddr) {
        for sprite in self.sprites.iter() {
            let mut fine_x = x - sprite.x_pos as u32;
            if sprite.attr.flip_hor() == 1 {
                fine_x = 7 - fine_x;
            }

            let pattern_0 = sprite.pattern_0 >> (7 - fine_x);
            let pattern_1 = sprite.pattern_1 >> (7 - fine_x);

            if (0..8).contains(&fine_x) && ((pattern_0 | pattern_1) & 0x1 != 0) {
                let mut palette_addr = PaletteAddr::default();
                palette_addr.set_p0(pattern_0 as u16);
                palette_addr.set_p1(pattern_1 as u16);
                palette_addr.set_a0(sprite.attr.palette0() as u16);
                palette_addr.set_a1(sprite.attr.palette1() as u16);
                palette_addr.set_s(1);
                return (*sprite, palette_addr);
            }
        }

        (SpriteInfo::default(), PaletteAddr { data: 0 })
    }

    fn sprite_eval(&mut self, row: u32, bus: &mut impl PpuBus) {
        let mut oam_idx = 0;
        let mut sprites_found = 0;

        while oam_idx < OAM_SIZE {
            let y = self.oam[oam_idx];
            let tile = self.oam[oam_idx + 1];
            let attr = SpriteAttr {
                data: self.oam[oam_idx + 2],
            };
            let x = self.oam[oam_idx + 3];
            let mut fine_y = row - (y as u32);
            if attr.flip_ver() == 1 {
                fine_y = 7 - fine_y;
            }

            if (0..8).contains(&fine_y) {
                if sprites_found < SPRITES_PER_ROW {
                    let mut fetch_pattern = |plane| -> u8 {
                        let mut pattern_addr = PatternAddr::default();
                        pattern_addr.set_fine_y(fine_y as u16);
                        pattern_addr.set_p(plane as u16);
                        pattern_addr.set_tile(tile as u16);
                        pattern_addr.set_h(self.ctrl.s() as u16);

                        bus.ppu_read(pattern_addr.data)
                    };

                    self.sprites.push(SpriteInfo {
                        x_pos: x,
                        y_pos: y,
                        pattern_0: fetch_pattern(0),
                        pattern_1: fetch_pattern(1),
                        attr: attr,
                        sprite_0: oam_idx == 0,
                    });

                    sprites_found += 1;
                } else {
                    self.status.set_o(1);
                    break;
                }
            }

            oam_idx += 4;
        }
    }

    fn get_color(&self, palette_addr: PaletteAddr) -> Rgb {
        PPU_PALETTE[self.palette_ram[get_palette_addr(palette_addr.data)] as usize % PALETTE_SIZE]
    }

    fn fine_x(&self, val: u16) -> u16 {
        (val << self.x) >> 15
    }

    fn bg_enabled(&self) -> bool {
        self.mask.bg_enabled() == 1
    }

    fn sprites_enabled(&self) -> bool {
        self.mask.sprites_enabled() == 1
    }

    fn rendering_enabled(&self) -> bool {
        self.bg_enabled() || self.sprites_enabled()
    }
}

fn load_shift_reg(reg: &mut u16, val: u8) {
    *reg &= 0xFF00;
    *reg |= val as u16;
}

fn get_palette_addr(addr: u16) -> usize {
    let mut addr = addr as usize % PALETTE_RAM_SIZE;

    if addr == 0x0010 {
        addr = 0x0000;
    }
    if addr == 0x0014 {
        addr = 0x0004;
    }
    if addr == 0x0018 {
        addr = 0x0008;
    }
    if addr == 0x001C {
        addr = 0x000C;
    }

    addr
}

fn is_opaque(palette_addr: PaletteAddr) -> bool {
    !(palette_addr.p0() == 0 && palette_addr.p1() == 0)
}
