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
