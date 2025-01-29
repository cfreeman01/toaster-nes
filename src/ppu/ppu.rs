mod ppu_regs;

#[cfg(test)]
mod test;

use ppu_regs::*;

#[derive(Default)]
pub struct Ppu {
    ppu_ctrl: PpuCtrl,
    ppu_mask: PpuMask,
    v: VramAddr,
    t: VramAddr,
    w: bool,
    x: u8,
}

pub trait PpuBus {
    fn ppu_read(&mut self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, data: u8);
}

impl Ppu {
    pub fn step(&mut self, bus: &mut impl PpuBus) {}
}
