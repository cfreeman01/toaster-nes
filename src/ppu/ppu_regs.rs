macro_rules! mask {
    ($pos:expr, $width:expr) => {{
        let mut mask = 0;
        for _ in 0..$width {
            mask <<= 1;
            mask |= 0x1;
        }
        mask << $pos
    }};
}

macro_rules! get_set_field {
    ($get:ident, $set:ident, $pos:expr, $width:expr, $type:ident) => {
        pub fn $get(&self) -> $type {
            (self.data & mask!($pos, $width)) >> $pos
        }

        pub fn $set(&mut self, val: $type) {
            let mask = mask!($pos, $width);
            let val = (val << $pos) & mask;
            self.data &= !mask;
            self.data |= val;
        }
    };
}

#[derive(Default)]
pub struct VramAddr {
    pub data: u16,
}

impl VramAddr {
    get_set_field!(coarse_x, set_coarse_x, 0, 5, u16);
    get_set_field!(coarse_y, set_coarse_y, 5, 5, u16);
    get_set_field!(n, set_n, 10, 2, u16);
    get_set_field!(nx, set_nx, 10, 1, u16);
    get_set_field!(ny, set_ny, 11, 1, u16);
    get_set_field!(fine_y, set_fine_y, 12, 3, u16);
    get_set_field!(addr, set_addr, 0, 14, u16);
    get_set_field!(addr_low, set_addr_low, 0, 8, u16);
    get_set_field!(addr_hi, set_addr_hi, 8, 6, u16);
}

#[derive(Default)]
pub struct PpuCtrl {
    pub data: u8,
}

impl PpuCtrl {
    get_set_field!(n, set_n, 0, 2, u8);
    get_set_field!(nx, set_nx, 0, 1, u8);
    get_set_field!(ny, set_ny, 1, 1, u8);
    get_set_field!(i, set_i, 2, 1, u8);
    get_set_field!(s, set_s, 3, 1, u8);
    get_set_field!(b, set_b, 4, 1, u8);
    get_set_field!(h, set_h, 5, 1, u8);
    get_set_field!(p, set_p, 6, 1, u8);
    get_set_field!(v, set_v, 7, 1, u8);
}

#[derive(Default)]
pub struct PpuMask {
    pub data: u8,
}

impl PpuMask {
    get_set_field!(b, set_b, 3, 1, u8);
    get_set_field!(s, set_s, 4, 1, u8);
}

#[derive(Default)]
pub struct PpuStatus {
    pub data: u8,
}

impl PpuStatus {
    get_set_field!(o, set_o, 5, 1, u8);
    get_set_field!(s, set_s, 6, 1, u8);
    get_set_field!(v, set_v, 7, 1, u8);
}
