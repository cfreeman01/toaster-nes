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

#[path = "controller/controller.rs"]
mod controller;

use cartridge::Cartridge;
pub use controller::Button;
use controller::Controller;
use cpu::{Cpu, CpuBus};
use ppu::{Ppu, PpuBus, OAM_ADDR, OAM_DATA};
use rom::Rom;

pub const DISPLAY_WIDTH: u32 = 256;
pub const DISPLAY_HEIGHT: u32 = 240;
pub const FRAME_SIZE_BYTES: usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT * 3) as usize;
pub const KB_8: usize = 8192;
pub const KB_16: usize = KB_8 * 2;
const RAM_SIZE: usize = 0x800;
const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;
const CPU_CART_START: u16 = 0x4020;
const CPU_CART_END: u16 = 0xFFFF;
const PPU_CART_START: u16 = 0x0000;
const PPU_CART_END: u16 = 0x3EFF;
const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3FFF;
const BUTTON_REG: u16 = 0x4016;
const DMA_REG: u16 = 0x4014;

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    ram: [u8; RAM_SIZE],
    cartridge: Cartridge,
    controller: Controller,
    dma_flag: bool,
    dma_addr: u16,
    dma_data: u8,
    dma_write_toggle: bool,
    cpu_bus_val: u8,
}

macro_rules! cpu_bus {
    ($nes:expr) => {
        &mut NesCpuBus {
            ram: &mut $nes.ram,
            ppu: &mut $nes.ppu,
            cartridge: &mut $nes.cartridge,
            controller: &mut $nes.controller,
            dma_flag: &mut $nes.dma_flag,
            dma_addr: &mut $nes.dma_addr,
            cpu_bus_val: &mut $nes.cpu_bus_val,
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
            cartridge: Cartridge::init(rom),
            controller: Controller::default(),
            dma_flag: false,
            dma_addr: 0,
            dma_data: 0,
            dma_write_toggle: false,
            cpu_bus_val: 0,
        };

        nes.cpu.reset = true;
        nes.cpu.step(cpu_bus!(nes));
        nes.cpu.reset = false;

        nes
    }

    pub fn frame(&mut self, frame: &mut [u8; FRAME_SIZE_BYTES]) {
        for _ in 0..ppu::CYCLES_PER_FRAME {
            self.tick(frame);
        }
    }

    pub fn set_button_state(&mut self, button: Button, pressed: bool) {
        self.controller.set_button_state(button, pressed);
    }

    fn tick(&mut self, frame: &mut [u8; FRAME_SIZE_BYTES]) {
        self.ppu.tick(ppu_bus!(self.cartridge), frame);

        self.cpu.nmi = self.ppu.nmi();

        if self.ppu.cycles() % 3 == 0 {
            if !self.dma_flag {
                self.cpu.tick(cpu_bus!(self));
            } else {
                self.dma_tick();
            }

            self.controller.update();
        }
    }

    fn dma_tick(&mut self) {
        if !self.dma_write_toggle {
            let dma_addr = self.dma_addr;
            self.dma_data = cpu_bus!(self).cpu_read(dma_addr);
            self.dma_addr += 1;
        } else {
            self.ppu.cpu_write(
                PPU_REG_START + OAM_DATA,
                self.dma_data,
                ppu_bus!(self.cartridge),
            );

            if self.dma_addr & 0x00FF == 0 {
                self.dma_flag = false;
            }
        }

        self.dma_write_toggle = !self.dma_write_toggle;
    }
}

struct NesCpuBus<'a> {
    ram: &'a mut [u8; RAM_SIZE],
    ppu: &'a mut Ppu,
    cartridge: &'a mut Cartridge,
    controller: &'a mut Controller,
    dma_flag: &'a mut bool,
    dma_addr: &'a mut u16,
    cpu_bus_val: &'a mut u8,
}

impl CpuBus for NesCpuBus<'_> {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        *self.cpu_bus_val = match addr {
            RAM_START..=RAM_END => self.ram[addr as usize % RAM_SIZE],
            PPU_REG_START..=PPU_REG_END => self.ppu.cpu_read(addr, ppu_bus!(self.cartridge)),
            CPU_CART_START..=CPU_CART_END => self.cartridge.cpu_read(addr),
            BUTTON_REG => self.controller.read() | (*self.cpu_bus_val & 0xF0),
            _ => *self.cpu_bus_val,
        };

        *self.cpu_bus_val
    }

    fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_START..=RAM_END => self.ram[addr as usize % RAM_SIZE] = data,
            PPU_REG_START..=PPU_REG_END => self.ppu.cpu_write(addr, data, ppu_bus!(self.cartridge)),
            CPU_CART_START..=CPU_CART_END => self.cartridge.cpu_write(addr, data),
            DMA_REG => {
                *self.dma_addr = (data as u16) << 8;
                *self.dma_flag = true
            }
            BUTTON_REG => self.controller.strobe = ((data & 0x1) == 1),
            _ => (),
        };
    }
}

struct NesPpuBus<'a> {
    cartridge: &'a mut Cartridge,
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
