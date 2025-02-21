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
const OAM2_SIZE: usize = 32;
const SPRITE_EVAL: u32 = 65;
const SPIRTES_PER_ROW: usize = 8;

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
    oam2: [u8; OAM2_SIZE],
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
            oam: [0xff; OAM_SIZE],
            oam2: [0xff; OAM2_SIZE],
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
                self.update_shift_regs();

                match (col - 1) % 8 {
                    0 => {
                        self.load_shift_regs();
                        self.fetch_tile_num(bus);
                    }
                    2 => self.fetch_attr_byte(bus),
                    4 => self.fetch_bg(bus, 0),
                    6 => self.fetch_bg(bus, 1),
                    7 => self.inc_v_hor(),
                    _ => (),
                };
            };

            if col == DISPLAY_WIDTH {
                self.inc_v_ver();
            }

            if col == DISPLAY_WIDTH + 1 && self.rendering_enabled() {
                self.load_shift_regs();
                self.v.set_coarse_x(self.t.coarse_x());
                self.v.set_nx(self.t.nx());
            }

            if row < DISPLAY_HEIGHT && col == SPRITE_EVAL {
                self.sprite_eval();
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
            let priority = self.get_priority();
            if priority == -1 {
                self.draw_bg_pixel(frame, row, col - 1);
            } else {
                self.draw_sprite_pixel(frame, row, col - 1, priority as u32);
            }
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
                    if [0x0014, 0x0018, 0x001C].contains(&(self.v.addr() - PALETTE_START)) {
                        return;
                    }
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

    fn fetch_tile_num(&mut self, bus: &mut impl PpuBus) {
        self.tile_num = bus.ppu_read(NAMETABLE_0_START | self.v.nt_addr());
    }

    fn fetch_attr_byte(&mut self, bus: &mut impl PpuBus) {
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

    fn fetch_bg(&mut self, bus: &mut impl PpuBus, plane: u8) {
        let mut pattern_addr = PatternAddr::default();
        pattern_addr.set_fine_y(self.v.fine_y());
        pattern_addr.set_p(plane as u16);
        pattern_addr.set_tile(self.tile_num as u16);
        pattern_addr.set_h(self.ctrl.b() as u16);

        if plane == 0 {
            self.bg_byte_0 = bus.ppu_read(pattern_addr.data);
        } else if plane == 1 {
            self.bg_byte_1 = bus.ppu_read(pattern_addr.data);
        } else {
            panic!("Invalid plane.")
        };
    }

    fn update_shift_regs(&mut self) {
        if self.mask.b() == 1 {
            self.bg_shift_reg_0 <<= 1;
            self.bg_shift_reg_1 <<= 1;
            self.attr_shift_reg_0 <<= 1;
            self.attr_shift_reg_1 <<= 1;
        }
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
        if self.rendering_enabled() {
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
    }

    fn inc_v_ver(&mut self) {
        if self.rendering_enabled() {
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
    }

    fn draw_bg_pixel(&mut self, frame: &mut [u8; FRAME_SIZE_BYTES], row: u32, col: u32) {
        let frame_idx = ((row * DISPLAY_WIDTH + col) * 3) as usize;

        let mut palette_addr = PaletteAddr::default();
        palette_addr.set_p0((self.bg_shift_reg_0 << self.x) >> 15);
        palette_addr.set_p1((self.bg_shift_reg_1 << self.x) >> 15);
        palette_addr.set_a0((self.attr_shift_reg_0 << self.x) >> 15);
        palette_addr.set_a1((self.attr_shift_reg_1 << self.x) >> 15);

        let color = PPU_PALETTE
            [self.palette_ram[get_palette_addr(palette_addr.data)] as usize % PALETTE_SIZE];

        Rgb(frame[frame_idx], frame[frame_idx + 1], frame[frame_idx + 2]) = color;
    }

    fn draw_sprite_pixel(
        &mut self,
        frame: &mut [u8; FRAME_SIZE_BYTES],
        row: u32,
        col: u32,
        sprite_num: u32,
    ) {
        let frame_idx = ((row * DISPLAY_WIDTH + col) * 3) as usize;
        Rgb(frame[frame_idx], frame[frame_idx + 1], frame[frame_idx + 2]) = Rgb(100, 100, 100);
    }

    fn rendering_enabled(&self) -> bool {
        self.mask.s() == 1 || self.mask.b() == 1
    }

    fn sprite_eval(&mut self) {
        let mut oam_idx = 0;
        let mut sprites_found = 0;
        let row = ((self.frame_cycle / ROW_SIZE) + 1) % DISPLAY_HEIGHT;
        self.oam2.fill(0xFF);

        while oam_idx < OAM_SIZE {
            let sprite_y = self.oam[oam_idx] as u32;

            if (0..8).contains(&(row - sprite_y)) && sprites_found < SPIRTES_PER_ROW {
                let oam2_idx = sprites_found * 4;
                self.oam2[oam2_idx] = self.oam[oam_idx];
                self.oam2[oam2_idx + 1] = self.oam[oam_idx + 1];
                self.oam2[oam2_idx + 2] = self.oam[oam_idx + 2];
                self.oam2[oam2_idx + 3] = self.oam[oam_idx + 3];
                sprites_found += 1;
            } else if sprites_found == SPIRTES_PER_ROW {
                self.status.set_o(1);
                break;
            }

            oam_idx += 4;
        }
    }

    fn get_priority(&mut self) -> i32 {
        let mut oam2_idx = 0;
        let mut sprite_found = false;
        let mut sprite_attr = SpriteAttr::default();
        let col = self.frame_cycle % ROW_SIZE;

        while oam2_idx < OAM2_SIZE {
            let sprite_x = self.oam2[oam2_idx + 3] as u32;

            if (0..8).contains(&(col - sprite_x)) {
                sprite_attr.data = self.oam2[oam2_idx + 2];

                if sprite_attr.priority() == 0 {
                    sprite_found = true;
                }

                break;
            }

            oam2_idx += 4;
        }

        if sprite_found {
            (oam2_idx / 4) as i32
        } else {
            -1
        }
    }
}

fn load_shift_reg(reg: &mut u16, val: u8) {
    *reg &= 0xFF00;
    *reg |= val as u16;
}

fn get_palette_addr(addr: u16) -> usize {
    let mut addr = addr as usize % PALETTE_RAM_SIZE;

    if addr % 4 == 0 {
        addr = 0;
    };

    addr
}
