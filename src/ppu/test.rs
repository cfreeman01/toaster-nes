use super::ppu_regs::*;
use super::*;

struct TestPpuBus {
    mem: [u8; 0x4000],
}

impl Default for TestPpuBus {
    fn default() -> Self {
        Self { mem: [0; 0x4000] }
    }
}

impl PpuBus for TestPpuBus {
    fn ppu_read(&mut self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn ppu_write(&mut self, addr: u16, data: u8) -> () {
        self.mem[addr as usize] = data
    }
}

fn init() -> (Ppu, TestPpuBus) {
    let mut ppu = Ppu::default();
    let mut bus = TestPpuBus::default();

    (ppu, bus)
}

#[test]
fn get_set_v() {
    let (mut ppu, _bus) = init();

    ppu.v.data = 0x571F;
    assert_eq!(ppu.v.coarse_x(), 0x001F);
    assert_eq!(ppu.v.coarse_y(), 0x0018);
    assert_eq!(ppu.v.n(), 1);
    assert_eq!(ppu.v.nx(), 1);
    assert_eq!(ppu.v.ny(), 0);
    assert_eq!(ppu.v.fine_y(), 0x5);

    ppu.v.data = 0;
    ppu.v.set_coarse_x(0x1F);
    ppu.v.set_coarse_y(0x18);
    ppu.v.set_nx(1);
    ppu.v.set_ny(0);
    ppu.v.set_fine_y(0x5);
    assert_eq!(ppu.v.data, 0x571F);
}

#[test]
fn reg_write() {
    let (mut ppu, mut bus) = init();

    ppu.cpu_write(0x2000, 0x01, &mut bus);
    assert_eq!(ppu.t.n(), 0x01);
    assert_eq!(ppu.t.nx(), 1);
    assert_eq!(ppu.t.ny(), 0);
    ppu.cpu_write(0x2005, 0x7D, &mut bus);
    assert_eq!(ppu.x, 0x5);
    assert_eq!(ppu.t.coarse_x(), 0xF);
    ppu.cpu_write(0x2005, 0x5E, &mut bus);
    assert_eq!(ppu.t.fine_y(), 0x6);
    assert_eq!(ppu.t.coarse_y(), 0xB);
    ppu.cpu_write(0x2006, 0x3D, &mut bus);
    assert_eq!(ppu.t.addr_hi(), 0x3D);
    ppu.cpu_write(0x2006, 0xF0, &mut bus);
    assert_eq!(ppu.t.addr_low(), 0xF0);
    assert_eq!(ppu.v.data, 0x3DF0);
}
