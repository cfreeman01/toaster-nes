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
const OAM_ADDR: u16 = 3;
const OAM_DATA: u16 = 4;
const PPU_SCROLL: u16 = 5;
const PPU_ADDR: u16 = 6;
const PPU_DATA: u16 = 7;
const PALETTE_RAM_SIZE: usize = 32;
const PRE_FETCH_START: u32 = 321;
const PRE_FETCH_END: u32 = 336;
const ATTR_TABLE_OFFSET: u32 = 0x3C0;
const PALETTE_START: u16 = 0x3F00;

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

#[derive(Default)]
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
    attr_latch_0: bool,
    attr_latch_1: bool,
    tile_num: u8,
    attr_byte: u8,
    bg_byte_0: u8,
    bg_byte_1: u8,
    cycles: u32,
    frame_cycle: u32,
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
        }

        if row == DISPLAY_HEIGHT + 1 && col == 1 {
            self.status.set_v(1)
        }
        if row == NUM_ROWS - 1 && col == 1 {
            self.status.set_v(0)
        }

        if row == NUM_ROWS - 1 && self.rendering_enabled() {
            self.v.set_coarse_y(self.t.coarse_y());
            self.v.set_fine_y(self.t.fine_y());
            self.v.set_ny(self.t.ny());
        }

        if row < DISPLAY_HEIGHT && (col - 1) < DISPLAY_WIDTH {
            self.draw_pixel(frame, row, col - 1);
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
            PPU_DATA => {
                if self.v.addr() >= PALETTE_START {
                    self.read_buf = self.palette_ram[self.v.addr() as usize % PALETTE_RAM_SIZE];
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
                    self.palette_ram[self.v.addr() as usize % PALETTE_RAM_SIZE] = data;
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
        attr_addr.set_tile_group_x(self.v.coarse_x() >> 2);
        attr_addr.set_tile_group_y(self.v.coarse_y() >> 2);
        attr_addr.set_n(self.v.n());
        self.attr_byte = bus.ppu_read(attr_addr.data());
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
        }
    }

    fn load_shift_regs(&mut self) {
        load_shift_reg(&mut self.bg_shift_reg_0, self.bg_byte_0);
        load_shift_reg(&mut self.bg_shift_reg_1, self.bg_byte_1);

        let tile_group_right = (self.v.coarse_x() % 4 > 1) as u8;
        let tile_group_bottom = (self.v.coarse_y() % 4 > 1) as u8;
        let shift_amt = (tile_group_right | (tile_group_bottom << 1)) * 2;
        let attr = self.attr_byte >> shift_amt;
        self.attr_latch_0 = field!(attr, 0, 1) == 1;
        self.attr_latch_1 = field!(attr, 1, 1) == 1;
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

    fn draw_pixel(&mut self, frame: &mut [u8; FRAME_SIZE_BYTES], row: u32, col: u32) {
        let frame_idx = ((row * DISPLAY_WIDTH + col) * 3) as usize;

        let mut palette_addr = PaletteAddr::default();
        palette_addr.set_p0((self.bg_shift_reg_0 << self.x) >> 15);
        palette_addr.set_p1((self.bg_shift_reg_1 << self.x) >> 15);
        palette_addr.set_a0(self.attr_latch_0 as u16);
        palette_addr.set_a1(self.attr_latch_1 as u16);

        let color = self.get_color(palette_addr.data as u8);

        Rgb(frame[frame_idx], frame[frame_idx + 1], frame[frame_idx + 2]) = color;
    }

    fn get_color(&self, palette_ram_addr: u8) -> Rgb {
        let mut addr = palette_ram_addr as usize % PALETTE_RAM_SIZE;

        if addr % 4 == 0 {
            addr = 0;
        };

        PPU_PALETTE[self.palette_ram[addr] as usize % PALETTE_SIZE]
    }

    fn rendering_enabled(&self) -> bool {
        self.mask.s() == 1 || self.mask.b() == 1
    }
}

fn load_shift_reg(reg: &mut u16, val: u8) {
    *reg &= 0xFF00;
    *reg |= val as u16;
}
