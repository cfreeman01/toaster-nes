#[cfg(test)]
mod test;

#[derive(Default)]
pub struct Ppu {}

pub trait PpuBus {
    fn ppu_read(&self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, data: u8);
}

impl Ppu {
    pub fn step(&mut self, bus: &mut impl PpuBus) {
        println!("ppu step!");
    }
}
