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

pub const DISPLAY_WIDTH: u32 = 256;
pub const DISPLAY_HEIGHT: u32 = 240;
pub const FRAME_SIZE_BYTES: usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT * 3) as usize;
const RAM_SIZE: usize = 0x800;
const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;
const CPU_CART_START: u16 = 0x4020;
const CPU_CART_END: u16 = 0xFFFF;
const PPU_CART_START: u16 = 0x0000;
const PPU_CART_END: u16 = 0x3EFF;
const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3FFF;

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

    pub fn frame(&mut self, frame: &mut [u8; FRAME_SIZE_BYTES]) {
        for _ in 0..ppu::CYCLES_PER_FRAME {
            self.tick(frame);
        }
    }

    fn tick(&mut self, frame: &mut [u8; FRAME_SIZE_BYTES]) {
        self.ppu.tick(ppu_bus!(*self.cartridge), frame);

        if self.ppu.cycles() % 3 == 0 {
            self.cpu.tick(cpu_bus!(self.ram, self.ppu, *self.cartridge));
        }
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
            PPU_REG_START..=PPU_REG_END => self.ppu.cpu_read(addr, ppu_bus!(*self.cartridge)),
            CPU_CART_START..=CPU_CART_END => self.cartridge.cpu_read(addr),
            _ => 0x00,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_START..=RAM_END => self.ram[addr as usize % RAM_SIZE] = data,
            PPU_REG_START..=PPU_REG_END => {
                self.ppu.cpu_write(addr, data, ppu_bus!(*self.cartridge))
            }
            CPU_CART_START..=CPU_CART_END => self.cartridge.cpu_write(addr, data),
            _ => (),
        };
    }
}

struct NesPpuBus<'a> {
    cartridge: &'a mut dyn Cartridge,
}

impl PpuBus for NesPpuBus<'_> {
    fn ppu_read(&mut self, addr: u16) -> u8 {
        match addr {
            PPU_CART_START..=PPU_CART_END => self.cartridge.ppu_read(addr),
            _ => 0x00,
        }
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        match addr {
            PPU_CART_START..=PPU_CART_END => self.cartridge.ppu_write(addr, data),
            _ => (),
        }
    }
}
