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
