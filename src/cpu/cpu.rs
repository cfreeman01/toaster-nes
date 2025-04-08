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
    nmi_prev: bool,
    irq_level_detected: bool,
    nmi_edge_detected: bool,
    irq_rdy: bool,
    nmi_latch: bool,
    cycles: u32,
    ins_cycles: u32,
}

pub trait CpuBus {
    fn cpu_read(&mut self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, data: u8);
}

pub const VEC_NMI: u16 = 0xFFFA;
pub const VEC_RESET: u16 = 0xFFFC;
pub const VEC_IRQ: u16 = 0xFFFE;
pub const STACK_BASE: u16 = 0x0100;

const NUM_CYCLES_INT: u32 = 7;
const NUM_CYCLES_RTI: u32 = 6;
const NUM_CYCLES_IMM: u32 = 2;
const NUM_CYCLES_IMP: u32 = 2;
const NUM_CYCLES_ZP_R: u32 = 3;
const NUM_CYCLES_ZPI_R: u32 = 4;
const NUM_CYCLES_ABS_R: u32 = 4;
const NUM_CYCLES_ABSI_R: u32 = 4;
const NUM_CYCLES_INDX_R: u32 = 6;
const NUM_CYCLES_INDY_R: u32 = 5;
const NUM_CYCLES_ACC: u32 = 2;
const NUM_CYCLES_ZP_RW: u32 = 5;
const NUM_CYCLES_ZPX_RW: u32 = 6;
const NUM_CYCLES_ABS_RW: u32 = 6;
const NUM_CYCLES_ABSX_RW: u32 = 7;
const NUM_CYCLES_ZP_W: u32 = 4;
const NUM_CYCLES_ZPI_W: u32 = 4;
const NUM_CYCLES_ABS_W: u32 = 4;
const NUM_CYCLES_ABSI_W: u32 = 5;
const NUM_CYCLES_INDX_W: u32 = 6;
const NUM_CYCLES_INDY_W: u32 = 6;
const NUM_CYCLES_BR: u32 = 2;
const NUM_CYCLES_JMP_ABS: u32 = 3;
const NUM_CYCLES_JMP_IND: u32 = 5;
const NUM_CYCLES_JSR: u32 = 6;
const NUM_CYCLES_RTS: u32 = 6;
const NUM_CYCLES_PUSH: u32 = 3;
const NUM_CYCLES_PULL: u32 = 4;

type InsR = fn(&mut Cpu, val: u8);
type InsRW = fn(&mut Cpu, val: u8) -> u8;
type InsImp = fn(&mut Cpu);

impl Cpu {
    pub fn tick(&mut self, bus: &mut impl CpuBus) {
        if self.ins_cycles == 0 {
            self.ins_cycles = if self.reset {
                self.int(bus, VEC_RESET, false)
            } else if self.nmi_latch {
                self.nmi_latch = false;
                self.int(bus, VEC_NMI, false)
            } else if self.irq_rdy && !self.i {
                self.int(bus, VEC_IRQ, false)
            } else {
                let opcode = bus.cpu_read(self.pc());
                self.exec(bus, opcode)
            }
        }

        if self.nmi_edge_detected {
            self.nmi_latch = true;
        }
        self.irq_rdy = self.irq_level_detected;
        self.nmi_edge_detected = self.nmi && !self.nmi_prev;
        self.irq_level_detected = self.irq;
        self.nmi_prev = self.nmi;
        self.cycles += 1;
        self.ins_cycles -= 1;
    }

    pub fn step(&mut self, bus: &mut impl CpuBus) {
        while {
            self.tick(bus);
            self.ins_cycles != 0
        } {}
    }

    pub fn cycles(&self) -> u32 {
        self.cycles
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
            0x90 => self.br(bus, !self.c),
            0xB0 => self.br(bus, self.c),
            0xD0 => self.br(bus, !self.z),
            0xF0 => self.br(bus, self.z),
            0x10 => self.br(bus, !self.n),
            0x30 => self.br(bus, self.n),
            0x50 => self.br(bus, !self.v),
            0x70 => self.br(bus, self.v),
            0x24 => self.zp_r(bus, Cpu::bit),
            0x2C => self.abs_r(bus, Cpu::bit),
            0x00 => self.int(bus, VEC_IRQ, true),
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
            0x4C => self.jmp_abs(bus),
            0x6C => self.jmp_ind(bus),
            0x20 => self.jsr(bus),
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
            0x48 => self.ph(bus, self.a),
            0x08 => self.ph(bus, self.get_flags() | (1 << 4)),
            0x68 => self.pl(bus, Cpu::pla),
            0x28 => self.pl(bus, Cpu::plp),
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
            0x40 => self.rti(bus),
            0x60 => self.rts(bus),
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
            0x85 => self.zp_w(bus, self.a),
            0x95 => self.zpi_w(bus, self.a, self.x),
            0x8D => self.abs_w(bus, self.a),
            0x9D => self.absi_w(bus, self.a, self.x),
            0x99 => self.absi_w(bus, self.a, self.y),
            0x81 => self.indx_w(bus, self.a),
            0x91 => self.indy_w(bus, self.a),
            0x86 => self.zp_w(bus, self.x),
            0x96 => self.zpi_w(bus, self.x, self.y),
            0x8E => self.abs_w(bus, self.x),
            0x84 => self.zp_w(bus, self.y),
            0x94 => self.zpi_w(bus, self.y, self.x),
            0x8C => self.abs_w(bus, self.y),
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

        let (pcl, pch) = (self.pcl(), self.pch());
        self.push(bus, pch);
        self.push(bus, pcl);

        let flags = if brk {
            self.get_flags() | (1 << 4)
        } else {
            self.get_flags()
        };
        self.push(bus, flags);

        let vec_low = bus.cpu_read(vec);
        let vec_high = bus.cpu_read(vec + 1);

        self.pc = u8_to_u16(vec_low, vec_high);

        self.i = true;

        NUM_CYCLES_INT
    }

    fn rti(&mut self, bus: &mut impl CpuBus) -> u32 {
        let flags = self.pull(bus);
        let addr_low = self.pull(bus);
        let addr_high = self.pull(bus);
        self.set_flags(flags);
        self.pc = u8_to_u16(addr_low, addr_high);

        NUM_CYCLES_RTI
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

        NUM_CYCLES_ZP_R
    }

    fn zpi_r(&mut self, bus: &mut impl CpuBus, ins: InsR, idx: u8) -> u32 {
        let addr = bus.cpu_read(self.pc());
        let val = bus.cpu_read((addr + idx) as u16);
        ins(self, val);

        NUM_CYCLES_ZPI_R
    }

    fn abs_r(&mut self, bus: &mut impl CpuBus, ins: InsR) -> u32 {
        let addr_low = bus.cpu_read(self.pc());
        let addr_high = bus.cpu_read(self.pc());
        let val = bus.cpu_read(u8_to_u16(addr_low, addr_high));
        ins(self, val);

        NUM_CYCLES_ABS_R
    }

    fn absi_r(&mut self, bus: &mut impl CpuBus, ins: InsR, idx: u8) -> u32 {
        let addr_low = bus.cpu_read(self.pc());
        let addr_high = bus.cpu_read(self.pc());
        let val = bus.cpu_read(u8_to_u16(addr_low, addr_high) + (idx as u16));
        ins(self, val);

        if check_overflow(addr_low, idx) {
            NUM_CYCLES_ABSI_R + 1
        } else {
            NUM_CYCLES_ABSI_R
        }
    }

    fn indx_r(&mut self, bus: &mut impl CpuBus, ins: InsR) -> u32 {
        let addr = bus.cpu_read(self.pc());
        let addr_low = bus.cpu_read((addr + self.x) as u16);
        let addr_high = bus.cpu_read((addr + self.x + 1) as u16);
        let val = bus.cpu_read(u8_to_u16(addr_low, addr_high));
        ins(self, val);

        NUM_CYCLES_INDX_R
    }

    fn indy_r(&mut self, bus: &mut impl CpuBus, ins: InsR) -> u32 {
        let addr = bus.cpu_read(self.pc());
        let addr_low = bus.cpu_read(addr as u16);
        let addr_high = bus.cpu_read((addr + 1) as u16);
        let val = bus.cpu_read(u8_to_u16(addr_low, addr_high) + (self.y as u16));
        ins(self, val);

        if check_overflow(addr_low, self.y) {
            NUM_CYCLES_INDY_R + 1
        } else {
            NUM_CYCLES_INDY_R
        }
    }

    fn acc(&mut self, bus: &mut impl CpuBus, ins: InsRW) -> u32 {
        self.a = ins(self, self.a);

        NUM_CYCLES_ACC
    }

    fn zp_rw(&mut self, bus: &mut impl CpuBus, ins: InsRW) -> u32 {
        let addr = bus.cpu_read(self.pc());
        let val = bus.cpu_read(addr as u16);
        bus.cpu_write(addr as u16, val);
        bus.cpu_write(addr as u16, ins(self, val));

        NUM_CYCLES_ZP_RW
    }

    fn zpx_rw(&mut self, bus: &mut impl CpuBus, ins: InsRW) -> u32 {
        let addr = (bus.cpu_read(self.pc()) + self.x) as u16;
        let val = bus.cpu_read(addr);
        bus.cpu_write(addr, val);
        bus.cpu_write(addr, ins(self, val));

        NUM_CYCLES_ZPX_RW
    }

    fn abs_rw(&mut self, bus: &mut impl CpuBus, ins: InsRW) -> u32 {
        let addr_low = bus.cpu_read(self.pc());
        let addr_high = bus.cpu_read(self.pc());
        let addr = u8_to_u16(addr_low, addr_high);
        let val = bus.cpu_read(addr);
        bus.cpu_write(addr, val);
        bus.cpu_write(addr, ins(self, val));

        NUM_CYCLES_ABS_RW
    }

    fn absx_rw(&mut self, bus: &mut impl CpuBus, ins: InsRW) -> u32 {
        let addr_low = bus.cpu_read(self.pc());
        let addr_high = bus.cpu_read(self.pc());
        let addr = u8_to_u16(addr_low, addr_high) + (self.x as u16);
        let val = bus.cpu_read(addr);
        bus.cpu_write(addr, val);
        bus.cpu_write(addr, ins(self, val));

        NUM_CYCLES_ABSX_RW
    }

    fn zp_w(&mut self, bus: &mut impl CpuBus, val: u8) -> u32 {
        let addr = bus.cpu_read(self.pc());
        bus.cpu_write(addr as u16, val);

        NUM_CYCLES_ZP_W
    }

    fn zpi_w(&mut self, bus: &mut impl CpuBus, val: u8, idx: u8) -> u32 {
        let addr = bus.cpu_read(self.pc());
        bus.cpu_write((addr + idx) as u16, val);

        NUM_CYCLES_ZPI_W
    }

    fn abs_w(&mut self, bus: &mut impl CpuBus, val: u8) -> u32 {
        let addr_low = bus.cpu_read(self.pc());
        let addr_high = bus.cpu_read(self.pc());
        bus.cpu_write(u8_to_u16(addr_low, addr_high), val);

        NUM_CYCLES_ABS_W
    }

    fn absi_w(&mut self, bus: &mut impl CpuBus, val: u8, idx: u8) -> u32 {
        let addr_low = bus.cpu_read(self.pc());
        let addr_high = bus.cpu_read(self.pc());
        bus.cpu_write(u8_to_u16(addr_low, addr_high) + (idx as u16), val);

        NUM_CYCLES_ABSI_W
    }

    fn indx_w(&mut self, bus: &mut impl CpuBus, val: u8) -> u32 {
        let addr = bus.cpu_read(self.pc());
        let addr_low = bus.cpu_read((addr + self.x) as u16);
        let addr_high = bus.cpu_read((addr + self.x + 1) as u16);
        bus.cpu_write(u8_to_u16(addr_low, addr_high), val);

        NUM_CYCLES_INDX_W
    }

    fn indy_w(&mut self, bus: &mut impl CpuBus, val: u8) -> u32 {
        let addr = bus.cpu_read(self.pc());
        let addr_low = bus.cpu_read(addr as u16);
        let addr_high = bus.cpu_read((addr + 1) as u16);
        bus.cpu_write(u8_to_u16(addr_low, addr_high) + (self.y as u16), val);

        NUM_CYCLES_INDY_W
    }

    fn br(&mut self, bus: &mut impl CpuBus, cond: bool) -> u32 {
        let offset = bus.cpu_read(self.pc());

        if !cond {
            NUM_CYCLES_BR
        } else {
            let tmp_pc = u8_to_u16(self.pcl() + offset, self.pch());
            self.pc = add_u16_i8(self.pc, offset as i8);
            if self.pc == tmp_pc {
                NUM_CYCLES_BR + 1
            } else {
                NUM_CYCLES_BR + 2
            }
        }
    }

    fn jmp_abs(&mut self, bus: &mut impl CpuBus) -> u32 {
        let addr_low = bus.cpu_read(self.pc());
        let addr_high = bus.cpu_read(self.pc());
        self.pc = u8_to_u16(addr_low, addr_high);

        NUM_CYCLES_JMP_ABS
    }

    fn jmp_ind(&mut self, bus: &mut impl CpuBus) -> u32 {
        let ptr_low = bus.cpu_read(self.pc());
        let ptr_high = bus.cpu_read(self.pc());
        let addr_low = bus.cpu_read(u8_to_u16(ptr_low, ptr_high));
        let addr_high = bus.cpu_read(u8_to_u16(ptr_low + 1, ptr_high));
        self.pc = u8_to_u16(addr_low, addr_high);

        NUM_CYCLES_JMP_IND
    }

    fn jsr(&mut self, bus: &mut impl CpuBus) -> u32 {
        let addr_low = bus.cpu_read(self.pc());
        let (pcl, pch) = (self.pcl(), self.pch());
        self.push(bus, pch);
        self.push(bus, pcl);
        let addr_high = bus.cpu_read(self.pc());
        self.pc = u8_to_u16(addr_low, addr_high);

        NUM_CYCLES_JSR
    }

    fn rts(&mut self, bus: &mut impl CpuBus) -> u32 {
        let addr_low = self.pull(bus);
        let addr_high = self.pull(bus);
        self.pc = u8_to_u16(addr_low, addr_high) + 1;

        NUM_CYCLES_RTS
    }

    fn ph(&mut self, bus: &mut impl CpuBus, val: u8) -> u32 {
        self.push(bus, val);

        NUM_CYCLES_PUSH
    }

    fn pl(&mut self, bus: &mut impl CpuBus, ins: InsR) -> u32 {
        let val = self.pull(bus);
        ins(self, val);

        NUM_CYCLES_PULL
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

    fn push(&mut self, bus: &mut impl CpuBus, val: u8) {
        bus.cpu_write(stack(self.s), val);
        self.s -= 1;
    }

    fn pull(&mut self, bus: &mut impl CpuBus) -> u8 {
        self.s += 1;
        bus.cpu_read(stack(self.s))
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
        flags
    }

    fn adc(&mut self, val: u8) {
        let sum: u32 = (self.a as u32) + (val as u32) + (self.c as u32);
        self.c = sum > 0xff;
        self.v = (!(self.a ^ val) & (self.a ^ (sum as u8)) & 0x80) != 0;
        self.a = sum as u8;
        self.set_zn(self.a);
    }

    fn and(&mut self, val: u8) {
        self.a &= val;
        self.set_zn(self.a);
    }

    fn asl(&mut self, val: u8) -> u8 {
        self.c = (val & 0x80) != 0;
        self.set_zn(val << 1);
        val << 1
    }

    fn bit(&mut self, val: u8) {
        self.n = (val & 0x80) != 0;
        self.v = (val & 0x40) != 0;
        self.z = (self.a & val) == 0;
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
        self.a ^= val;
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
        self.set_zn(self.y);
    }

    fn lsr(&mut self, val: u8) -> u8 {
        self.c = (val & 0x01) != 0;
        let val = val >> 1;
        self.set_zn(val);
        val
    }

    fn nop(&mut self) {}

    fn ora(&mut self, val: u8) {
        self.a |= val;
        self.set_zn(self.a);
    }

    fn pla(&mut self, val: u8) {
        self.a = val;
        self.set_zn(self.a);
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

fn add_u16_i8(val: u16, offset: i8) -> u16 {
    let val = val as i16;
    let offset = offset as i16;
    (val + offset) as u16
}

fn check_overflow(val_0: u8, val_1: u8) -> bool {
    let res: u16 = (val_0 as u16) + (val_1 as u16);

    (res >> 8) != 0
}

fn stack(s: u8) -> u16 {
    STACK_BASE + (s as u16)
}
