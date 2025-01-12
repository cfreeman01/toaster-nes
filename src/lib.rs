#[path = "assemble/assemble.rs"]
pub mod assemble;

#[path = "cpu/cpu.rs"]
mod cpu;

#[path = "ppu/ppu.rs"]
mod ppu;

use crate::cpu::{Cpu, CpuBus};
use crate::ppu::{Ppu, PpuBus};

const RAM_SIZE: usize = 0x800;

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    ram: [u8; RAM_SIZE],
}

macro_rules! cpu_bus {
    ( $x:expr ) => {
        &mut NesCpuBus { ram: &mut $x.ram }
    };
}

macro_rules! ppu_bus {
    ( $x:expr ) => {
        &mut NesPpuBus { ram: &mut $x.ram }
    };
}

impl Nes {
    pub fn init() -> Self {
        Self {
            cpu: Cpu::default(),
            ppu: Ppu::default(),
            ram: [0xff; RAM_SIZE],
        }
    }

    pub fn step(&mut self) {
        let (cpu, ppu) = (&mut self.cpu, &mut self.ppu);

        let cycles = cpu.step(cpu_bus!(self));

        for _ in 0..cycles * 3 {
            ppu.step(ppu_bus!(self));
        }
    }
}

struct NesCpuBus<'a> {
    ram: &'a mut [u8; RAM_SIZE],
}

impl CpuBus for NesCpuBus<'_> {
    fn cpu_read(&self, addr: u16) -> u8 {
        self.ram[0]
    }

    fn cpu_write(&mut self, addr: u16, data: u8) {}
}

struct NesPpuBus<'a> {
    ram: &'a mut [u8; RAM_SIZE],
}

impl PpuBus for NesPpuBus<'_> {
    fn ppu_read(&self, addr: u16) -> u8 {
        self.ram[0]
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {}
}
