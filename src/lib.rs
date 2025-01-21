#[path = "assemble/assemble.rs"]
pub mod assemble;

#[path = "rom/rom.rs"]
pub mod rom;

#[path = "cpu/cpu.rs"]
mod cpu;

#[path = "ppu/ppu.rs"]
mod ppu;

#[path = "cartridge/cartridge.rs"]
mod cartridge;

use cartridge::{cart_init, Cartridge};
use cpu::{Cpu, CpuBus};
use ppu::{Ppu, PpuBus};
use rom::Rom;

const RAM_SIZE: usize = 0x800;
const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;
const CPU_CART_START: u16 = 0x4020;
const CPU_CART_END: u16 = 0xFFFF;
const PPU_CART_START: u16 = 0x0000;
const PPU_CART_END: u16 = 0x3EFF;

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    ram: [u8; RAM_SIZE],
    cartridge: Box<dyn Cartridge>,
    cpu_bus_val: u8,
    ppu_bus_val: u8,
}

macro_rules! cpu_bus {
    ( $x:expr ) => {
        &mut NesCpuBus {
            ram: &mut $x.ram,
            ppu: &mut $x.ppu,
            cartridge: &mut *$x.cartridge,
            cpu_bus_val: &mut $x.cpu_bus_val,
        }
    };
}

macro_rules! ppu_bus {
    ( $x:expr ) => {
        &mut NesPpuBus {
            cartridge: &mut *$x.cartridge,
            ppu_bus_val: &mut $x.ppu_bus_val,
        }
    };
}

impl Nes {
    pub fn init(rom: &Rom) -> Self {
        let mut nes = Self {
            cpu: Cpu::default(),
            ppu: Ppu::default(),
            ram: [0xff; RAM_SIZE],
            cartridge: cart_init(rom),
            cpu_bus_val: 0,
            ppu_bus_val: 0,
        };

        nes.cpu.reset = true;
        nes.cpu.step(cpu_bus!(nes));
        nes.cpu.reset = false;
        
        nes
    }

    pub fn step(&mut self) {
        let (cpu, ppu) = (&mut self.cpu, &mut self.ppu);

        cpu.step(cpu_bus!(self));
    }
}

struct NesCpuBus<'a> {
    ram: &'a mut [u8; RAM_SIZE],
    ppu: &'a mut Ppu,
    cartridge: &'a mut dyn Cartridge,
    cpu_bus_val: &'a mut u8,
}

impl CpuBus for NesCpuBus<'_> {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        *self.cpu_bus_val = match addr {
            RAM_START..=RAM_END => self.ram[addr as usize % RAM_SIZE],
            CPU_CART_START..=CPU_CART_END => self
                .cartridge
                .cpu_read(addr)
                .unwrap_or_else(|| *self.cpu_bus_val),
            _ => *self.cpu_bus_val,
        };

        *self.cpu_bus_val
    }

    fn cpu_write(&mut self, addr: u16, data: u8) {}
}

struct NesPpuBus<'a> {
    cartridge: &'a mut dyn Cartridge,
    ppu_bus_val: &'a mut u8,
}

impl PpuBus for NesPpuBus<'_> {
    fn ppu_read(&mut self, addr: u16) -> u8 {
        self.cartridge.ppu_read(0).unwrap()
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {}
}
