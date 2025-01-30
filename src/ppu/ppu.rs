mod ppu_regs;

#[cfg(test)]
mod test;

use crate::{PPU_REG_END, PPU_REG_START};
use ppu_regs::*;

const PPU_CTRL: u16 = 0;
const PPU_MASK: u16 = 1;
const PPU_STATUS: u16 = 2;
const OAM_ADDR: u16 = 3;
const OAM_DATA: u16 = 4;
const PPU_SCROLL: u16 = 5;
const PPU_ADDR: u16 = 6;
const PPU_DATA: u16 = 7;

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
    ppu_ctrl: PpuCtrl,
    ppu_mask: PpuMask,
    ppu_status: PpuStatus,
    v: VramAddr,
    t: VramAddr,
    w: bool,
    x: u8,
    ppu_read_buf: u8,
}

pub trait PpuBus {
    fn ppu_read(&mut self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, data: u8);
}

impl Ppu {
    pub fn step(&mut self, ppu_bus: &mut impl PpuBus) {}

    pub fn cpu_read(&mut self, addr: u16, bus: &mut impl PpuBus) -> u8 {
        match addr % 8 {
            PPU_STATUS => {
                self.w = false;
                self.ppu_status.data
            }
            PPU_DATA => {
                let val = self.ppu_read_buf;
                self.ppu_read_buf = bus.ppu_read(self.v.addr());
                self.v.data += if self.ppu_ctrl.i() == 1 { 32 } else { 1 };
                val
            }
            _ => 0x0,
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8, ppu_bus: &mut impl PpuBus) {
        match addr % 8 {
            PPU_CTRL => {
                self.ppu_ctrl.data = data;
                self.t.set_n(field!(data, 0, 2) as u16);
            }
            PPU_MASK => {
                self.ppu_mask.data = data;
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
                    self.t.data &= 0x3FFF;
                    self.w = true;
                } else {
                    self.t.set_addr_low(data as u16);
                    self.v.data = self.t.data;
                    self.w = false;
                }
            }
            PPU_DATA => {
                ppu_bus.ppu_write(self.v.addr(), data);
                self.v.data += if self.ppu_ctrl.i() == 1 { 32 } else { 1 }
            }
            _ => (),
        }
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
}
