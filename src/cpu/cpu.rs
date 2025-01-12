#[cfg(test)]
mod test;

pub const VEC_NMI: u16 = 0xFFFA;
pub const VEC_RESET: u16 = 0xFFFC;
pub const VEC_IRQ: u16 = 0xFFFE;
pub const STACK_BASE: u16 = 0x0100;

#[derive(Default)]
pub struct Cpu {
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    pc: u16,
    n: bool,
    v: bool,
    d: bool,
    i: bool,
    z: bool,
    c: bool,
    pub reset: bool,
    pub irq: bool,
    pub nmi: bool,
    prev_nmi: bool,
}

pub trait CpuBus {
    fn cpu_read(&self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, data: u8);
}

impl Cpu {
    pub fn step(&mut self, bus: &mut impl CpuBus) -> u32 {
        1
    }
}
