use super::*;
use crate::assemble::assemble;

const PRG_ADDR: u16 = 0x8000;

struct TestCpuBus {
    mem: [u8; 0x10000],
}

impl Default for TestCpuBus {
    fn default() -> Self {
        Self { mem: [0; 0x10000] }
    }
}

impl CpuBus for TestCpuBus {
    fn cpu_read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn cpu_write(&mut self, addr: u16, data: u8) -> () {
        self.mem[addr as usize] = data
    }
}

impl TestCpuBus {
    fn cpu_write_16(&mut self, addr: u16, data: u16) -> () {
        let low = data as u8;
        let high = (data >> 8) as u8;
        self.cpu_write(addr, low);
        self.cpu_write(addr + 1, high);
    }

    fn cpu_write_vec(&mut self, addr: u16, vec: Vec<u8>) {
        let mut addr = addr;

        for byte in vec {
            self.cpu_write(addr, byte);
            addr += 1;
        }
    }
}

fn init(prg_src: &str, prg_addr: u16) -> (Cpu, TestCpuBus) {
    let mut cpu = Cpu::default();
    let mut bus = TestCpuBus::default();

    bus.cpu_write_16(VEC_RESET, prg_addr);

    let prg_bin =
        assemble(prg_src).unwrap_or_else(|msg| panic!("Error assembling program: {}", msg));

    bus.cpu_write_vec(prg_addr, prg_bin);

    cpu.reset = true;
    cpu.step(&mut bus);
    cpu.reset = false;

    (cpu, bus)
}

fn cycles_since_reset(cpu: &Cpu) -> u32 {
    cpu.cycles - (NUM_CYCLES_INT as u32)
}

#[test]
fn reset() {
    let (cpu, _bus) = init("", 0xBEEF);

    assert_eq!(cpu.cycles, NUM_CYCLES_INT);
    assert_eq!(cpu.pc, 0xBEEF);
    assert_eq!(cpu.s, 0xFD);
    assert_eq!(cpu.i, true);
}

#[test]
fn adc_imm() {
    let (mut cpu, mut bus) = init(
        "ADC #$FF
        ADC #$01
        ADC #$01",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFF);
    assert_eq!(cpu.n, true);
    assert_eq!(cpu.v, false);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0);
    assert_eq!(cpu.z, true);
    assert_eq!(cpu.c, true);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 2);
}

#[test]
fn adc_zp() {
    let (mut cpu, mut bus) = init("ADC $00", PRG_ADDR);

    bus.cpu_write(0x0000, 0xFF);

    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFF);
}

#[test]
fn adc_zpx() {
    let (mut cpu, mut bus) = init(
        "LDX #$01
         ADC $00,X",
        PRG_ADDR,
    );

    bus.cpu_write(0x0001, 0xFF);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFF);
}

#[test]
fn adc_zpx_wrap() {
    let (mut cpu, mut bus) = init(
        "LDX #$FF
         ADC $01,X",
        PRG_ADDR,
    );

    bus.cpu_write(0x0000, 0xFF);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFF);
}

#[test]
fn adc_abs() {
    let (mut cpu, mut bus) = init("ADC $06FF", PRG_ADDR);

    bus.cpu_write(0x06FF, 0xFF);

    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFF);
}

#[test]
fn adc_absx_no_cross() {
    let (mut cpu, mut bus) = init(
        "LDX #$01
        ADC $00FE,X",
        PRG_ADDR,
    );

    bus.cpu_write(0x00FF, 0xFF);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0xFF);
    assert_eq!(cycles_since_reset(&cpu), 6);
}

#[test]
fn adc_absx_cross() {
    let (mut cpu, mut bus) = init(
        "LDX #$01
        ADC $00FF,X",
        PRG_ADDR,
    );

    bus.cpu_write(0x0100, 0xFF);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0xFF);
    assert_eq!(cycles_since_reset(&cpu), 7);
}

#[test]
fn adc_absy() {
    let (mut cpu, mut bus) = init(
        "LDY #$01
        ADC $00FE,Y",
        PRG_ADDR,
    );

    bus.cpu_write(0x00FF, 0xFF);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0xFF);
    assert_eq!(cycles_since_reset(&cpu), 6);
}

#[test]
fn adc_indx() {
    let (mut cpu, mut bus) = init(
        "LDX #$01
        ADC ($04,X)",
        PRG_ADDR,
    );

    bus.cpu_write(0x0005, 0xFF);
    bus.cpu_write(0x0006, 0x06);
    bus.cpu_write(0x06FF, 0xFF);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0xFF);
    assert_eq!(cycles_since_reset(&cpu), 8);
}

#[test]
fn adc_indy_nocross() {
    let (mut cpu, mut bus) = init(
        "LDY #$01
        ADC ($05),Y",
        PRG_ADDR,
    );

    bus.cpu_write(0x0005, 0xFE);
    bus.cpu_write(0x0006, 0x06);
    bus.cpu_write(0x06FF, 0xFF);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0xFF);
    assert_eq!(cycles_since_reset(&cpu), 7);
}

#[test]
fn adc_indy_cross() {
    let (mut cpu, mut bus) = init(
        "LDY #$01
        ADC ($05),Y",
        PRG_ADDR,
    );

    bus.cpu_write(0x0005, 0xFF);
    bus.cpu_write(0x0006, 0x06);
    bus.cpu_write(0x0700, 0xFF);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0xFF);
    assert_eq!(cycles_since_reset(&cpu), 8);
}

#[test]
fn and() {
    let (mut cpu, mut bus) = init(
        "ADC #$FF
        AND #$0F",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x0F);
}

#[test]
fn asl_acc() {
    let (mut cpu, mut bus) = init(
        "ADC #$FF
        ASL A",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFE);
    assert_eq!(cpu.c, true);
    assert_eq!(cpu.n, true);
}

#[test]
fn asl_zp() {
    let (mut cpu, mut bus) = init(
        "ASL $AA
        ASL $AA
        ASL $AA",
        PRG_ADDR,
    );

    bus.cpu_write(0x00AA, 0xC0);

    cpu.step(&mut bus);
    assert_eq!(bus.cpu_read(0x00AA), 0x80);
    assert_eq!(cpu.c, true);
    assert_eq!(cpu.n, true);
    cpu.step(&mut bus);
    assert_eq!(bus.cpu_read(0x00AA), 0x00);
    assert_eq!(cpu.c, true);
    assert_eq!(cpu.z, true);
    cpu.step(&mut bus);
    assert_eq!(bus.cpu_read(0x00AA), 0x00);
    assert_eq!(cpu.c, false);
    assert_eq!(cpu.z, true);
}

#[test]
fn asl_zpx() {
    let (mut cpu, mut bus) = init(
        "LDX #$0A
        ASL $A0,X",
        PRG_ADDR,
    );

    bus.cpu_write(0x00AA, 0xC0);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(bus.cpu_read(0x00AA), 0x80);
}

#[test]
fn asl_abs() {
    let (mut cpu, mut bus) = init("ASL $1111", PRG_ADDR);

    bus.cpu_write(0x1111, 0xC0);

    cpu.step(&mut bus);
    assert_eq!(bus.cpu_read(0x1111), 0x80);
}

#[test]
fn asl_absx() {
    let (mut cpu, mut bus) = init(
        "LDX #$01
        ASL $06FE,X",
        PRG_ADDR,
    );

    bus.cpu_write(0x06FF, 0xC0);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(bus.cpu_read(0x06FF), 0x80);
}

#[test]
fn bit() {
    let (mut cpu, mut bus) = init(
        "ADC #$01
        BIT $11",
        PRG_ADDR,
    );

    bus.cpu_write(0x0011, 0xC0);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.z, true);
    assert_eq!(cpu.n, true);
    assert_eq!(cpu.v, true);
}

#[test]
fn clc() {
    let (mut cpu, mut bus) = init("CLC", PRG_ADDR);

    cpu.c = true;
    cpu.step(&mut bus);
    assert_eq!(cpu.c, false);
}

#[test]
fn cld() {
    let (mut cpu, mut bus) = init("CLD", PRG_ADDR);

    cpu.d = true;
    cpu.step(&mut bus);
    assert_eq!(cpu.d, false);
}

#[test]
fn cli() {
    let (mut cpu, mut bus) = init("CLI", PRG_ADDR);

    cpu.step(&mut bus);
    assert_eq!(cpu.i, false);
}

#[test]
fn clv() {
    let (mut cpu, mut bus) = init("CLV", PRG_ADDR);

    cpu.v = true;
    cpu.step(&mut bus);
    assert_eq!(cpu.v, false);
}

#[test]
fn cmp() {
    let (mut cpu, mut bus) = init(
        "ADC #$FF
        CMP #$FF",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.z, true);
    assert_eq!(cpu.c, true);
    assert_eq!(cpu.n, false);
}

#[test]
fn cpx() {
    let (mut cpu, mut bus) = init(
        "LDX #$FF
        CPX #$FF",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.z, true);
    assert_eq!(cpu.c, true);
    assert_eq!(cpu.n, false);
}

#[test]
fn cpy() {
    let (mut cpu, mut bus) = init(
        "LDY #$FF
        CPY #$FF",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.z, true);
    assert_eq!(cpu.c, true);
    assert_eq!(cpu.n, false);
}

#[test]
fn dec() {
    let (mut cpu, mut bus) = init("DEC $1111", PRG_ADDR);

    bus.cpu_write(0x1111, 0x08);

    cpu.step(&mut bus);
    assert_eq!(bus.cpu_read(0x1111), 0x07);
}

#[test]
fn dex() {
    let (mut cpu, mut bus) = init(
        "LDX #$FF
        DEX",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.x, 0xFE);
    assert_eq!(cpu.n, true);
    assert_eq!(cpu.z, false);
}

#[test]
fn dey() {
    let (mut cpu, mut bus) = init(
        "LDY #$0F
        DEY",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.y, 0x0E);
    assert_eq!(cpu.n, false);
    assert_eq!(cpu.z, false);
}

#[test]
fn eor() {
    let (mut cpu, mut bus) = init(
        "ADC #$FF
        EOR #$0F",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xF0);
}

#[test]
fn inc() {
    let (mut cpu, mut bus) = init("INC $11", PRG_ADDR);

    bus.cpu_write(0x0011, 0x08);

    cpu.step(&mut bus);
    assert_eq!(bus.cpu_read(0x0011), 0x09);
}

#[test]
fn inx() {
    let (mut cpu, mut bus) = init(
        "LDX #$FF
        INX",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.x, 0x00);
    assert_eq!(cpu.z, true);
    assert_eq!(cpu.n, false);
}

#[test]
fn iny() {
    let (mut cpu, mut bus) = init(
        "LDY #$00
        INY",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.y, 0x01);
    assert_eq!(cpu.z, false);
    assert_eq!(cpu.n, false);
}

#[test]
fn lda() {
    let (mut cpu, mut bus) = init("LDA #$FF", PRG_ADDR);

    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFF);
}

#[test]
fn ldx_imm() {
    let (mut cpu, mut bus) = init("LDX #$BE", PRG_ADDR);

    cpu.step(&mut bus);
    assert_eq!(cpu.x, 0xBE);
}

#[test]
fn ldy_imm() {
    let (mut cpu, mut bus) = init("LDY #$EF", PRG_ADDR);

    cpu.step(&mut bus);
    assert_eq!(cpu.y, 0xEF);
}

#[test]
fn lsr() {
    let (mut cpu, mut bus) = init(
        "LDA #$FF
        LSR A",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x7F);
    assert_eq!(cpu.c, true);
    assert_eq!(cpu.n, false);
    assert_eq!(cpu.z, false);
}

#[test]
fn ora() {
    let (mut cpu, mut bus) = init(
        "LDA #$0F
        ORA $EE",
        PRG_ADDR,
    );

    bus.cpu_write(0x00EE, 0xF0);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFF);
    assert_eq!(cpu.n, true);
    assert_eq!(cpu.z, false);
}

#[test]
fn rol() {
    let (mut cpu, mut bus) = init(
        "LDA #$FF
        ROL A
        ROL A",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFE);
    assert_eq!(cpu.c, true);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFD);
    assert_eq!(cpu.c, true);
}

#[test]
fn ror() {
    let (mut cpu, mut bus) = init(
        "LDA #$FF
        ROR A
        ROR A",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x7F);
    assert_eq!(cpu.c, true);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xBF);
    assert_eq!(cpu.c, true);
}

#[test]
fn sbc() {
    let (mut cpu, mut bus) = init(
        "SBC #$01
        SBC #$01",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFE);
    assert_eq!(cpu.n, true);
    assert_eq!(cpu.v, false);
    assert_eq!(cpu.c, false);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFC);
    assert_eq!(cpu.n, true);
    assert_eq!(cpu.v, false);
    assert_eq!(cpu.c, true);
}

#[test]
fn sta_zp() {
    let (mut cpu, mut bus) = init(
        "LDA #$FF
        STA $AA",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(bus.cpu_read(0x00AA), 0xFF);
}

#[test]
fn sta_zpx() {
    let (mut cpu, mut bus) = init(
        "LDA #$FF
        LDX #$0A
        STA $A0,X",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(bus.cpu_read(0x00AA), 0xFF);
}

#[test]
fn sta_abs() {
    let (mut cpu, mut bus) = init(
        "LDA #$FF
        STA $1122",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(bus.cpu_read(0x1122), 0xFF);
}

#[test]
fn sta_absx() {
    let (mut cpu, mut bus) = init(
        "LDA #$FF
        LDX #$01
        STA $00FF,X",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(bus.cpu_read(0x0100), 0xFF);
}

#[test]
fn sta_absy() {
    let (mut cpu, mut bus) = init(
        "LDA #$FF
        LDY #$01
        STA $1000,Y",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(bus.cpu_read(0x1001), 0xFF);
}

#[test]
fn sta_indx() {
    let (mut cpu, mut bus) = init(
        "LDA #$FF
        LDX #$01
        STA ($10,X)",
        PRG_ADDR,
    );

    bus.cpu_write_16(0x0011, 0x1122);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(bus.cpu_read(0x1122), 0xFF);
}

#[test]
fn sta_indy() {
    let (mut cpu, mut bus) = init(
        "LDA #$FF
        LDY #$01
        STA ($10),Y",
        PRG_ADDR,
    );

    bus.cpu_write_16(0x0010, 0x2221);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(bus.cpu_read(0x2222), 0xFF);
}

#[test]
fn bcc_not_taken() {
    let (mut cpu, mut bus) = init(
        "SEC
        BCC $0F",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.pc, PRG_ADDR + 3);
    assert_eq!(cycles_since_reset(&cpu), 4);
}

#[test]
fn bcc_taken_nocross_forward() {
    let (mut cpu, mut bus) = init("BCC $0F", PRG_ADDR);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, PRG_ADDR + 2 + 0x0F);
    assert_eq!(cycles_since_reset(&cpu), 3);
}

#[test]
fn bcc_taken_nocross_backward() {
    let (mut cpu, mut bus) = init("BCC $FF", PRG_ADDR);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, PRG_ADDR + 1);
    assert_eq!(cycles_since_reset(&cpu), 3);
}

#[test]
fn bcc_taken_cross_forward() {
    let (mut cpu, mut bus) = init("BCC $03", 0x80FC);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x8101);
    assert_eq!(cycles_since_reset(&cpu), 4);
}

#[test]
fn bcc_taken_cross_backward() {
    let (mut cpu, mut bus) = init("BCC $FD", 0x8000);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x7FFF);
    assert_eq!(cycles_since_reset(&cpu), 4);
}

#[test]
fn jmp_abs() {
    let (mut cpu, mut bus) = init("JMP $BEEF", PRG_ADDR);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0xBEEF);
    assert_eq!(cycles_since_reset(&cpu), 3);
}

#[test]
fn jmp_ind_nowrap() {
    let (mut cpu, mut bus) = init("JMP ($1111)", PRG_ADDR);

    bus.cpu_write_16(0x1111, 0xBEEF);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0xBEEF);
    assert_eq!(cycles_since_reset(&cpu), 5);
}

#[test]
fn jmp_ind_wrap() {
    let (mut cpu, mut bus) = init("JMP ($10FF)", PRG_ADDR);

    bus.cpu_write(0x10FF, 0xEF);
    bus.cpu_write(0x1000, 0xBE);

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0xBEEF);
    assert_eq!(cycles_since_reset(&cpu), 5);
}

#[test]
fn jsr_rts() {
    let (mut cpu, mut bus) = init("JSR $8888", 0x8000);

    let sub_bin = assemble(
        "LDA #$FF
        SEC
        RTS",
    )
    .unwrap_or_else(|msg| panic!("Error assembling program: {}", msg));

    bus.cpu_write_vec(0x8888, sub_bin);

    cpu.step(&mut bus);
    assert_eq!(bus.cpu_read(0x01FD), 0x80);
    assert_eq!(bus.cpu_read(0x01FC), 0x02);
    assert_eq!(cpu.pc, 0x8888);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8003);
}

#[test]
fn push_pull_a() {
    let (mut cpu, mut bus) = init(
        "LDA #$FF
        PHA
        LDA #$00
        PLA",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(bus.cpu_read(0x01FD), 0xFF);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFF);
}

#[test]
fn push_pull_flags() {
    let (mut cpu, mut bus) = init(
        "SED
        SEC
        PHP
        CLD
        CLC
        PLP",
        PRG_ADDR,
    );

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.c, true);
    assert_eq!(cpu.d, true);
    cpu.step(&mut bus);
    assert_eq!(bus.cpu_read(0x01FD), 0x3D);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.c, false);
    assert_eq!(cpu.d, false);
    cpu.step(&mut bus);
    assert_eq!(cpu.c, true);
    assert_eq!(cpu.d, true);
    assert_eq!(cpu.get_flags(), 0x2D);
}

#[test]
fn brk() {
    let (mut cpu, mut bus) = init(
        "SEC
        BRK",
        PRG_ADDR,
    );

    let isr_bin = assemble(
        "SED
        RTI",
    )
    .unwrap_or_else(|msg| panic!("Error assembling program: {}", msg));

    bus.cpu_write_16(VEC_IRQ, 0x1111);
    bus.cpu_write_vec(0x1111, isr_bin);

    cpu.step(&mut bus);
    assert_eq!(cpu.c, true);
    cpu.step(&mut bus);
    assert_eq!(bus.cpu_read(0x01FD), 0x80);
    assert_eq!(bus.cpu_read(0x01FC), 0x03);
    assert_eq!(bus.cpu_read(0x01FB), 0x35);
    assert_eq!(cpu.pc, 0x1111);
    cpu.step(&mut bus);
    assert_eq!(cpu.d, true);
    cpu.step(&mut bus);
    assert_eq!(cpu.d, false);
    assert_eq!(cpu.pc, 0x8003);
}

#[test]
fn irq() {
    let (mut cpu, mut bus) = init(
        "CLI
        LDA #$AA",
        PRG_ADDR,
    );

    let isr_bin = assemble(
        "SED
        RTI",
    )
    .unwrap_or_else(|msg| panic!("Error assembling program: {}", msg));

    bus.cpu_write_16(VEC_IRQ, 0x1111);
    bus.cpu_write_vec(0x1111, isr_bin);

    cpu.step(&mut bus);
    assert_eq!(cpu.i, false);
    cpu.irq = true;
    cpu.step(&mut bus);
    assert_eq!(cpu.i, true);
    assert_eq!(cpu.pc, 0x1111);
    cpu.step(&mut bus);
    assert_eq!(cpu.d, true);
    cpu.step(&mut bus);
    assert_eq!(cpu.d, false);
    assert_eq!(cpu.i, false);
    assert_eq!(cpu.pc, 0x8001);
    cpu.irq = false;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xAA);
}

#[test]
fn nmi() {
    let (mut cpu, mut bus) = init(
        "SEI
        LDA #$AA
        LDA #$FF",
        PRG_ADDR,
    );

    let isr_bin = assemble(
        "SED
        RTI",
    )
    .unwrap_or_else(|msg| panic!("Error assembling program: {}", msg));

    bus.cpu_write_16(VEC_NMI, 0x1111);
    bus.cpu_write_vec(0x1111, isr_bin);

    cpu.step(&mut bus);
    assert_eq!(cpu.i, true);
    cpu.nmi = true;
    cpu.step(&mut bus);
    assert_eq!(cpu.i, true);
    assert_eq!(cpu.pc, 0x1111);
    cpu.step(&mut bus);
    assert_eq!(cpu.d, true);
    cpu.step(&mut bus);
    assert_eq!(cpu.d, false);
    assert_eq!(cpu.i, true);
    assert_eq!(cpu.pc, 0x8001);
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xAA);
    assert_eq!(cpu.nmi, true);
    cpu.nmi = false;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFF);
}