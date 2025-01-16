#[cfg(test)]
mod test;

use std::collections::HashMap;

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
const NUM_CYCLES_IMP: u32 = 2;
const NUM_CYCLES_ZPR: u32 = 3;
const NUM_CYCLES_ZPIR: u32 = 4;
const NUM_CYCLES_ABSR: u32 = 4;
const NUM_CYCLES_ABSIR: u32 = 4;
const NUM_CYCLES_INDXR: u32 = 6;
const NUM_CYCLES_INDYR: u32 = 5;
const NUM_CYCLES_ACC: u32 = 2;
const NUM_CYCLES_ZPRW: u32 = 5;
const NUM_CYCLES_ZPXRW: u32 = 6;
const NUM_CYCLES_ABSRW: u32 = 6;
const NUM_CYCLES_ABSXRW: u32 = 7;
const NUM_CYCLES_ZPW: u32 = 4;
const NUM_CYCLES_ZPIW: u32 = 4;
const NUM_CYCLES_ABSW: u32 = 4;
const NUM_CYCLES_ABSIW: u32 = 5;
const NUM_CYCLES_INDXW: u32 = 6;
const NUM_CYCLES_INDYW: u32 = 6;

type InsR = fn(&mut Cpu, val: u8);
type InsW = fn(&Cpu) -> u8;
type InsRW = fn(&mut Cpu, val: u8) -> u8;
type InsBr = fn(&mut Cpu) -> bool;
type InsImp = fn(&mut Cpu);

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
            self.exec(bus, opcode)
        };

        self.prev_nmi = self.nmi;

        self.cycles += num_cycles;
        num_cycles
    }

    fn exec(&mut self, bus: &mut impl CpuBus, opcode: u8) -> u32 {
        match opcode {
            0x69 => self.imm(bus, Cpu::adc),
            0x65 => self.zp_r(bus, Cpu::adc),
            0x75 => self.zpi_r(bus, Cpu::adc, self.x),
            0x6D => self.abs_r(bus, Cpu::adc),
            0x7D => self.absi_r(bus, Cpu::adc, self.x),
            0x79 => self.absi_r(bus, Cpu::adc, self.y),
            0x61 => self.indx_r(bus, Cpu::adc),
            0x71 => self.indy_r(bus, Cpu::adc),
            0x29 => self.imm(bus, Cpu::and),
            0x25 => self.zp_r(bus, Cpu::and),
            0x35 => self.zpi_r(bus, Cpu::and, self.x),
            0x2D => self.abs_r(bus, Cpu::and),
            0x3D => self.absi_r(bus, Cpu::and, self.x),
            0x39 => self.absi_r(bus, Cpu::and, self.y),
            0x21 => self.indx_r(bus, Cpu::and),
            0x31 => self.indy_r(bus, Cpu::and),
            0x0A => self.acc(bus, Cpu::asl),
            0x06 => self.zp_rw(bus, Cpu::asl),
            0x16 => self.zpx_rw(bus, Cpu::asl),
            0x0E => self.abs_rw(bus, Cpu::asl),
            0x1E => self.absx_rw(bus, Cpu::asl),
            // (0x90, Br(Cpu::bcc)),
            // (0xB0, Br(Cpu::bcs)),
            // (0xD0, Br(Cpu::bne)),
            // (0xF0, Br(Cpu::beq)),
            // (0x10, Br(Cpu::bpl)),
            // (0x30, Br(Cpu::bmi)),
            // (0x50, Br(Cpu::bvc)),
            // (0x70, Br(Cpu::bvs)),
            0x24 => self.zp_r(bus, Cpu::bit),
            0x2C => self.abs_r(bus, Cpu::bit),
            // (0x00, Brk),
            0x18 => self.imp(bus, Cpu::clc),
            0xD8 => self.imp(bus, Cpu::cld),
            0x58 => self.imp(bus, Cpu::cli),
            0xB8 => self.imp(bus, Cpu::clv),
            0xC9 => self.imm(bus, Cpu::cmp),
            0xC5 => self.zp_r(bus, Cpu::cmp),
            0xD5 => self.zpi_r(bus, Cpu::cmp, self.x),
            0xCD => self.abs_r(bus, Cpu::cmp),
            0xDD => self.absi_r(bus, Cpu::cmp, self.x),
            0xD9 => self.absi_r(bus, Cpu::cmp, self.y),
            0xC1 => self.indx_r(bus, Cpu::cmp),
            0xD1 => self.indy_r(bus, Cpu::cmp),
            0xE0 => self.imm(bus, Cpu::cpx),
            0xE4 => self.zp_r(bus, Cpu::cpx),
            0xEC => self.abs_r(bus, Cpu::cpx),
            0xC0 => self.imm(bus, Cpu::cpy),
            0xC4 => self.zp_r(bus, Cpu::cpy),
            0xCC => self.abs_r(bus, Cpu::cpy),
            0xC6 => self.zp_rw(bus, Cpu::dec),
            0xD6 => self.zpx_rw(bus, Cpu::dec),
            0xCE => self.abs_rw(bus, Cpu::dec),
            0xDE => self.absx_rw(bus, Cpu::dec),
            0xCA => self.imp(bus, Cpu::dex),
            0x88 => self.imp(bus, Cpu::dey),
            0x49 => self.imm(bus, Cpu::eor),
            0x45 => self.zp_r(bus, Cpu::eor),
            0x55 => self.zpi_r(bus, Cpu::eor, self.x),
            0x4D => self.abs_r(bus, Cpu::eor),
            0x5D => self.absi_r(bus, Cpu::eor, self.x),
            0x59 => self.absi_r(bus, Cpu::eor, self.y),
            0x41 => self.indx_r(bus, Cpu::eor),
            0x51 => self.indy_r(bus, Cpu::eor),
            0xE6 => self.zp_rw(bus, Cpu::inc),
            0xF6 => self.zpx_rw(bus, Cpu::inc),
            0xEE => self.abs_rw(bus, Cpu::inc),
            0xFE => self.absx_rw(bus, Cpu::inc),
            0xE8 => self.imp(bus, Cpu::inx),
            0xC8 => self.imp(bus, Cpu::iny),
            // (0x4C, JmpAbs),
            // (0x6C, JmpInd),
            // (0x20, Jsr),
            0xA9 => self.imm(bus, Cpu::lda),
            0xA5 => self.zp_r(bus, Cpu::lda),
            0xB5 => self.zpi_r(bus, Cpu::lda, self.x),
            0xAD => self.abs_r(bus, Cpu::lda),
            0xBD => self.absi_r(bus, Cpu::lda, self.x),
            0xB9 => self.absi_r(bus, Cpu::lda, self.y),
            0xA1 => self.indx_r(bus, Cpu::lda),
            0xB1 => self.indy_r(bus, Cpu::lda),
            0xA2 => self.imm(bus, Cpu::ldx),
            0xA6 => self.zp_r(bus, Cpu::ldx),
            0xB6 => self.zpi_r(bus, Cpu::ldx, self.y),
            0xAE => self.abs_r(bus, Cpu::ldx),
            0xBE => self.absi_r(bus, Cpu::ldx, self.y),
            0xA0 => self.imm(bus, Cpu::ldy),
            0xA4 => self.zp_r(bus, Cpu::ldy),
            0xB4 => self.zpi_r(bus, Cpu::ldy, self.x),
            0xAC => self.abs_r(bus, Cpu::ldy),
            0xBC => self.absi_r(bus, Cpu::ldy, self.x),
            0x4A => self.acc(bus, Cpu::lsr),
            0x46 => self.zp_rw(bus, Cpu::lsr),
            0x56 => self.zpx_rw(bus, Cpu::lsr),
            0x4E => self.abs_rw(bus, Cpu::lsr),
            0x5E => self.absx_rw(bus, Cpu::lsr),
            0xEA => self.imp(bus, Cpu::nop),
            0x09 => self.imm(bus, Cpu::ora),
            0x05 => self.zp_r(bus, Cpu::ora),
            0x15 => self.zpi_r(bus, Cpu::ora, self.x),
            0x0D => self.abs_r(bus, Cpu::ora),
            0x1D => self.absi_r(bus, Cpu::ora, self.x),
            0x19 => self.absi_r(bus, Cpu::ora, self.y),
            0x01 => self.indx_r(bus, Cpu::ora),
            0x11 => self.indy_r(bus, Cpu::ora),
            // (0x48, Push(Cpu::pha),
            // (0x08, Push(Cpu::php),
            // (0x68, Pull(Cpu::pla),
            // (0x28, Pull(Cpu::plp),
            0x2A => self.acc(bus, Cpu::rol),
            0x26 => self.zp_rw(bus, Cpu::rol),
            0x36 => self.zpx_rw(bus, Cpu::rol),
            0x2E => self.abs_rw(bus, Cpu::rol),
            0x3E => self.absx_rw(bus, Cpu::rol),
            0x6A => self.acc(bus, Cpu::ror),
            0x66 => self.zp_rw(bus, Cpu::ror),
            0x76 => self.zpx_rw(bus, Cpu::ror),
            0x6E => self.abs_rw(bus, Cpu::ror),
            0x7E => self.absx_rw(bus, Cpu::ror),
            // (0x40, Rti),
            // (0x60, Rts),
            0xE9 => self.imm(bus, Cpu::sbc),
            0xE5 => self.zp_r(bus, Cpu::sbc),
            0xF5 => self.zpi_r(bus, Cpu::sbc, self.x),
            0xED => self.abs_r(bus, Cpu::sbc),
            0xFD => self.absi_r(bus, Cpu::sbc, self.x),
            0xF9 => self.absi_r(bus, Cpu::sbc, self.y),
            0xE1 => self.indx_r(bus, Cpu::sbc),
            0xF1 => self.indy_r(bus, Cpu::sbc),
            0x38 => self.imp(bus, Cpu::sec),
            0xF8 => self.imp(bus, Cpu::sed),
            0x78 => self.imp(bus, Cpu::sei),
            0x85 => self.zp_w(bus, Cpu::sta),
            0x95 => self.zpi_w(bus, Cpu::sta, self.x),
            0x8D => self.abs_w(bus, Cpu::sta),
            0x9D => self.absi_w(bus, Cpu::sta, self.x),
            0x99 => self.absi_w(bus, Cpu::sta, self.y),
            0x81 => self.indx_w(bus, Cpu::sta),
            0x91 => self.indy_w(bus, Cpu::sta),
            0x86 => self.zp_w(bus, Cpu::stx),
            0x96 => self.zpi_w(bus, Cpu::stx, self.y),
            0x8E => self.abs_w(bus, Cpu::stx),
            0x84 => self.zp_w(bus, Cpu::sty),
            0x94 => self.zpi_w(bus, Cpu::sty, self.x),
            0x8C => self.abs_w(bus, Cpu::sty),
            0xAA => self.imp(bus, Cpu::tax),
            0xA8 => self.imp(bus, Cpu::tay),
            0xBA => self.imp(bus, Cpu::tsx),
            0x8A => self.imp(bus, Cpu::txa),
            0x9A => self.imp(bus, Cpu::txs),
            0x98 => self.imp(bus, Cpu::tya),
            _ => panic!("Invalid opcode: {:02X}", opcode),
        }
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

    fn imp(&mut self, bus: &mut impl CpuBus, ins: InsImp) -> u32 {
        ins(self);

        NUM_CYCLES_IMP
    }

    fn imm(&mut self, bus: &mut impl CpuBus, ins: InsR) -> u32 {
        let val = bus.cpu_read(self.pc());
        ins(self, val);

        NUM_CYCLES_IMM
    }

    fn zp_r(&mut self, bus: &mut impl CpuBus, ins: InsR) -> u32 {
        let addr = bus.cpu_read(self.pc());
        let val = bus.cpu_read(addr as u16);
        ins(self, val);

        NUM_CYCLES_ZPR
    }

    fn zpi_r(&mut self, bus: &mut impl CpuBus, ins: InsR, idx: u8) -> u32 {
        let addr = bus.cpu_read(self.pc());
        let val = bus.cpu_read((addr + idx) as u16);
        ins(self, val);

        NUM_CYCLES_ZPIR
    }

    fn abs_r(&mut self, bus: &mut impl CpuBus, ins: InsR) -> u32 {
        let addr_low = bus.cpu_read(self.pc());
        let addr_high = bus.cpu_read(self.pc());
        let val = bus.cpu_read(u8_to_u16(addr_low, addr_high));
        ins(self, val);

        NUM_CYCLES_ABSR
    }

    fn absi_r(&mut self, bus: &mut impl CpuBus, ins: InsR, idx: u8) -> u32 {
        let addr_low = bus.cpu_read(self.pc());
        let addr_high = bus.cpu_read(self.pc());
        let val = bus.cpu_read(u8_to_u16(addr_low, addr_high) + (idx as u16));
        ins(self, val);

        if check_overflow(addr_low, idx) {
            NUM_CYCLES_ABSIR + 1
        } else {
            NUM_CYCLES_ABSIR
        }
    }

    fn indx_r(&mut self, bus: &mut impl CpuBus, ins: InsR) -> u32 {
        let addr = bus.cpu_read(self.pc());
        let addr_low = bus.cpu_read((addr + self.x) as u16);
        let addr_high = bus.cpu_read((addr + self.x + 1) as u16);
        let val = bus.cpu_read(u8_to_u16(addr_low, addr_high));
        ins(self, val);

        NUM_CYCLES_INDXR
    }

    fn indy_r(&mut self, bus: &mut impl CpuBus, ins: InsR) -> u32 {
        let addr = bus.cpu_read(self.pc());
        let addr_low = bus.cpu_read(addr as u16);
        let addr_high = bus.cpu_read((addr + 1) as u16);
        let val = bus.cpu_read(u8_to_u16(addr_low, addr_high) + (self.y as u16));
        ins(self, val);

        if check_overflow(addr_low, self.y) {
            NUM_CYCLES_INDYR + 1
        } else {
            NUM_CYCLES_INDYR
        }
    }

    fn acc(&mut self, bus: &mut impl CpuBus, ins: InsRW) -> u32 {
        self.a = ins(self, self.a);

        NUM_CYCLES_ACC
    }

    fn zp_rw(&mut self, bus: &mut impl CpuBus, ins: InsRW) -> u32 {
        let addr = bus.cpu_read(self.pc());
        let val = bus.cpu_read(addr as u16);
        bus.cpu_write(addr as u16, ins(self, val));

        NUM_CYCLES_ZPRW
    }

    fn zpx_rw(&mut self, bus: &mut impl CpuBus, ins: InsRW) -> u32 {
        let addr = (bus.cpu_read(self.pc()) + self.x) as u16;
        let val = bus.cpu_read(addr);
        bus.cpu_write(addr, ins(self, val));

        NUM_CYCLES_ZPXRW
    }

    fn abs_rw(&mut self, bus: &mut impl CpuBus, ins: InsRW) -> u32 {
        let addr_low = bus.cpu_read(self.pc());
        let addr_high = bus.cpu_read(self.pc());
        let addr = u8_to_u16(addr_low, addr_high);
        let val = bus.cpu_read(addr);
        bus.cpu_write(addr, ins(self, val));

        NUM_CYCLES_ABSRW
    }

    fn absx_rw(&mut self, bus: &mut impl CpuBus, ins: InsRW) -> u32 {
        let addr_low = bus.cpu_read(self.pc());
        let addr_high = bus.cpu_read(self.pc());
        let addr = u8_to_u16(addr_low, addr_high) + (self.x as u16);
        let val = bus.cpu_read(addr);
        bus.cpu_write(addr, ins(self, val));

        NUM_CYCLES_ABSXRW
    }

    fn zp_w(&mut self, bus: &mut impl CpuBus, ins: InsW) -> u32 {
        let addr = bus.cpu_read(self.pc());
        bus.cpu_write(addr as u16, ins(self));

        NUM_CYCLES_ZPW
    }

    fn zpi_w(&mut self, bus: &mut impl CpuBus, ins: InsW, idx: u8) -> u32 {
        let addr = bus.cpu_read(self.pc());
        bus.cpu_write((addr + idx) as u16, ins(self));

        NUM_CYCLES_ZPIW
    }

    fn abs_w(&mut self, bus: &mut impl CpuBus, ins: InsW) -> u32 {
        let addr_low = bus.cpu_read(self.pc());
        let addr_high = bus.cpu_read(self.pc());
        bus.cpu_write(u8_to_u16(addr_low, addr_high), ins(self));

        NUM_CYCLES_ABSW
    }

    fn absi_w(&mut self, bus: &mut impl CpuBus, ins: InsW, idx: u8) -> u32 {
        let addr_low = bus.cpu_read(self.pc());
        let addr_high = bus.cpu_read(self.pc());
        bus.cpu_write(u8_to_u16(addr_low, addr_high) + (idx as u16), ins(self));

        NUM_CYCLES_ABSIW
    }

    fn indx_w(&mut self, bus: &mut impl CpuBus, ins: InsW) -> u32 {
        let addr = bus.cpu_read(self.pc());
        let addr_low = bus.cpu_read((addr + self.x) as u16);
        let addr_high = bus.cpu_read((addr + self.x + 1) as u16);
        bus.cpu_write(u8_to_u16(addr_low, addr_high), ins(self));

        NUM_CYCLES_INDXW
    }

    fn indy_w(&mut self, bus: &mut impl CpuBus, ins: InsW) -> u32 {
        let addr = bus.cpu_read(self.pc());
        let addr_low = bus.cpu_read(addr as u16);
        let addr_high = bus.cpu_read((addr + 1) as u16);
        bus.cpu_write(u8_to_u16(addr_low, addr_high) + (self.y as u16), ins(self));

        NUM_CYCLES_INDYW
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

    fn and(&mut self, val: u8) {
        self.a = self.a & val;
        self.set_zn(self.a);
    }

    fn asl(&mut self, val: u8) -> u8 {
        self.c = (val & 0x80) != 0;
        self.set_zn(val << 1);
        val << 1
    }

    fn bcc(&self) -> bool {
        !self.c
    }

    fn bcs(&self) -> bool {
        self.c
    }

    fn beq(&self) -> bool {
        self.z
    }

    fn bit(&mut self, val: u8) {
        self.n = (val & 0x80) != 0;
        self.v = (val & 0x40) != 0;
        self.z = (self.a & val) == 0;
    }

    fn bmi(&self) -> bool {
        self.n
    }

    fn bne(&self) -> bool {
        !self.z
    }

    fn bpl(&self) -> bool {
        !self.n
    }

    fn bvc(&self) -> bool {
        !self.v
    }

    fn bvs(&self) -> bool {
        self.v
    }

    fn clc(&mut self) {
        self.c = false;
    }

    fn cld(&mut self) {
        self.d = false;
    }

    fn cli(&mut self) {
        self.i = false;
    }

    fn clv(&mut self) {
        self.v = false;
    }

    fn cmp(&mut self, val: u8) {
        self.c = self.a >= val;
        self.set_zn(self.a - val);
    }

    fn cpx(&mut self, val: u8) {
        self.c = self.x >= val;
        self.set_zn(self.x - val);
    }

    fn cpy(&mut self, val: u8) {
        self.c = self.y >= val;
        self.set_zn(self.y - val);
    }

    fn dec(&mut self, val: u8) -> u8 {
        let val = val - 1;
        self.set_zn(val);
        val
    }

    fn dex(&mut self) {
        self.x -= 1;
        self.set_zn(self.x);
    }

    fn dey(&mut self) {
        self.y -= 1;
        self.set_zn(self.y);
    }

    fn eor(&mut self, val: u8) {
        self.a = self.a ^ val;
        self.set_zn(self.a);
    }

    fn inc(&mut self, val: u8) -> u8 {
        let val = val + 1;
        self.set_zn(val);
        val
    }

    fn inx(&mut self) {
        self.x += 1;
        self.set_zn(self.x);
    }

    fn iny(&mut self) {
        self.y += 1;
        self.set_zn(self.y);
    }

    fn lda(&mut self, val: u8) {
        self.a = val;
        self.set_zn(self.a);
    }

    fn ldx(&mut self, val: u8) {
        self.x = val;
        self.set_zn(self.x);
    }

    fn ldy(&mut self, val: u8) {
        self.y = val;
        self.set_zn(self.x);
    }

    fn lsr(&mut self, val: u8) -> u8 {
        self.c = (val & 0x01) != 0;
        let val = val >> 1;
        self.set_zn(val);
        val
    }

    fn nop(&mut self) {}

    fn ora(&mut self, val: u8) {
        self.a = self.a | val;
        self.set_zn(self.a);
    }

    fn pha(&self) -> u8 {
        self.a
    }

    fn php(&self) -> u8 {
        self.get_flags() | (1 << 4)
    }

    fn pla(&mut self, val: u8) {
        self.a = val;
    }

    fn plp(&mut self, val: u8) {
        self.set_flags(val);
    }

    fn rol(&mut self, val: u8) -> u8 {
        let old_c = self.c;
        self.c = (val & 0x80) != 0;
        let mut val = val << 1;
        if old_c {
            val |= 0x01;
        }
        self.set_zn(val);
        val
    }

    fn ror(&mut self, val: u8) -> u8 {
        let old_c = self.c;
        self.c = (val & 0x01) != 0;
        let mut val = val >> 1;
        if old_c {
            val |= 0x80;
        }
        self.set_zn(val);
        val
    }

    fn sbc(&mut self, val: u8) {
        self.adc(!val);
    }

    fn sec(&mut self) {
        self.c = true;
    }

    fn sed(&mut self) {
        self.d = true;
    }

    fn sei(&mut self) {
        self.i = true;
    }

    fn sta(&self) -> u8 {
        self.a
    }

    fn stx(&self) -> u8 {
        self.x
    }

    fn sty(&self) -> u8 {
        self.y
    }

    fn tax(&mut self) {
        self.x = self.a;
        self.set_zn(self.x);
    }

    fn tay(&mut self) {
        self.y = self.a;
        self.set_zn(self.y);
    }

    fn tsx(&mut self) {
        self.x = self.s;
        self.set_zn(self.x);
    }

    fn txa(&mut self) {
        self.a = self.x;
        self.set_zn(self.a);
    }

    fn txs(&mut self) {
        self.s = self.x;
    }

    fn tya(&mut self) {
        self.a = self.y;
        self.set_zn(self.a);
    }
}

fn u8_to_u16(low: u8, high: u8) -> u16 {
    ((high as u16) << 8) | (low as u16)
}

fn check_overflow(val_0: u8, val_1: u8) -> bool {
    let res: u16 = (val_0 as u16) + (val_1 as u16);

    (res >> 8) != 0
}

fn stack(s: u8) -> u16 {
    STACK_BASE + (s as u16)
}
