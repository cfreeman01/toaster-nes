#[cfg(test)]
mod test;

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
    cycles: u32,
}

pub trait CpuBus {
    fn cpu_read(&self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, data: u8);
}

pub const VEC_NMI: u16 = 0xFFFA;
pub const VEC_RESET: u16 = 0xFFFC;
pub const VEC_IRQ: u16 = 0xFFFE;
pub const STACK_BASE: u16 = 0x0100;

const NUM_CYCLES_INT: u32 = 7;
const NUM_CYCLES_IMM: u32 = 2;

type Ins = fn(&mut Cpu);
type InsR = fn(&mut Cpu, val: u8);
type InsW = fn(&mut Cpu) -> u8;
type InsRW = fn(&mut Cpu, val: u8) -> u8;
type InsBr = fn(&mut Cpu) -> bool;

impl Cpu {
    pub fn step(&mut self, bus: &mut impl CpuBus) -> u32 {
        let num_cycles = if self.reset {
            self.int(bus, VEC_RESET, false)
        } else if self.nmi && !self.prev_nmi {
            self.int(bus, VEC_NMI, false)
        } else if self.irq && !self.i {
            self.int(bus, VEC_IRQ, false)
        } else {
            let opcode = bus.cpu_read(self.pc());

            match opcode {
                0x69 => self.imm(bus, Cpu::adc),
                _ => panic!("Invalid opcode: {:02X}", opcode),
            }
        };

        self.prev_nmi = self.nmi;

        self.cycles += num_cycles;
        num_cycles
    }

    fn int(&mut self, bus: &mut impl CpuBus, vec: u16, brk: bool) -> u32 {
        if brk {
            self.pc += 1;
        }

        bus.cpu_write(stack(self.s), self.pch());
        self.s -= 1;
        bus.cpu_write(stack(self.s), self.pcl());
        self.s -= 1;

        bus.cpu_write(
            stack(self.s),
            if brk {
                self.get_flags() | (1 << 4)
            } else {
                self.get_flags()
            },
        );
        self.s -= 1;

        let vec_low = bus.cpu_read(vec);
        let vec_high = bus.cpu_read(vec + 1);

        self.pc = u8_to_u16(vec_low, vec_high);

        self.i = true;

        NUM_CYCLES_INT
    }

    fn imm(&mut self, bus: &mut impl CpuBus, ins: InsR) -> u32 {
        let val = bus.cpu_read(self.pc());
        ins(self, val);
        NUM_CYCLES_IMM
    }

    fn pc(&mut self) -> u16 {
        self.pc += 1;
        self.pc - 1
    }

    fn pcl(&self) -> u8 {
        self.pc as u8
    }

    fn pch(&self) -> u8 {
        (self.pc >> 8) as u8
    }

    fn set_zn(&mut self, val: u8) {
        self.z = val == 0;
        self.n = (val & 0x80) != 0;
    }

    fn set_flags(&mut self, flags: u8) {
        self.n = (flags & (1 << 7)) != 0;
        self.v = (flags & (1 << 6)) != 0;
        self.d = (flags & (1 << 3)) != 0;
        self.i = (flags & (1 << 2)) != 0;
        self.z = (flags & (1 << 1)) != 0;
        self.c = (flags & (1 << 0)) != 0;
    }

    fn get_flags(&self) -> u8 {
        let mut flags: u8 = 0;
        if self.n {
            flags |= 1 << 7;
        }
        if self.v {
            flags |= 1 << 6;
        }
        flags |= 1 << 5;
        if self.d {
            flags |= 1 << 3;
        }
        if self.i {
            flags |= 1 << 2;
        }
        if self.z {
            flags |= 1 << 1;
        }
        if self.c {
            flags |= 1 << 0;
        }
        return flags;
    }

    fn adc(&mut self, val: u8) {
        let sum: u32 = (self.a as u32) + (val as u32) + (self.c as u32);
        self.c = sum > 0xff;
        self.v = (!(self.a ^ val) & (self.a ^ (sum as u8)) & 0x80) != 0;
        self.a = sum as u8;
        self.set_zn(self.a);
    }
}

fn u8_to_u16(low: u8, high: u8) -> u16 {
    ((high as u16) << 8) | (low as u16)
}

fn stack(s: u8) -> u16 {
    0x0100 + (s as u16)
}
