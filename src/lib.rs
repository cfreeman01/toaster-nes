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
}

macro_rules! cpu_bus {
    ($ram:expr, $ppu:expr, $cart:expr) => {
        &mut NesCpuBus {
            ram: &mut $ram,
            ppu: &mut $ppu,
            cartridge: &mut $cart,
        }
    };
}

macro_rules! ppu_bus {
    ( $cart:expr ) => {
        &mut NesPpuBus {
            cartridge: &mut $cart,
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
        };

        nes.cpu.reset = true;
        nes.cpu.step(cpu_bus!(nes.ram, nes.ppu, *nes.cartridge));
        nes.cpu.reset = false;
        
        nes
    }

    pub fn step(&mut self) {
        let (cpu, ppu) = (&mut self.cpu, &mut self.ppu);

        cpu.step(cpu_bus!(self.ram, self.ppu, *self.cartridge));
    }
}

struct NesCpuBus<'a> {
    ram: &'a mut [u8; RAM_SIZE],
    ppu: &'a mut Ppu,
    cartridge: &'a mut dyn Cartridge,
}

impl CpuBus for NesCpuBus<'_> {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => self.ram[addr as usize % RAM_SIZE],
            CPU_CART_START..=CPU_CART_END => self
                .cartridge
                .cpu_read(addr)
                .unwrap_or_else(|| 0x00),
            0x01 => ppu_bus!(*self.cartridge).ppu_read(0), //proof of concept: can instantiate and pass in a PPU bus here when accessing PPU bus via CPU bus
            _ => 0x00,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8) {
    }
}

struct NesPpuBus<'a> {
    cartridge: &'a mut dyn Cartridge,
}

impl PpuBus for NesPpuBus<'_> {
    fn ppu_read(&mut self, addr: u16) -> u8 {
        0x00
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
    }
}
