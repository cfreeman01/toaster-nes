use super::ATTR_TABLE_OFFSET;

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
    get_set_field!(nt_addr, set_nt_addr, 0, 12, u16);
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

pub struct AttrAddr {
    data: u16,
}

impl Default for AttrAddr {
    fn default() -> Self {
        Self {
            data: ATTR_TABLE_OFFSET as u16,
        }
    }
}

impl AttrAddr {
    pub fn data(&self) -> u16 {
        self.data
    }

    get_set_field!(tile_group_x, set_tile_group_x, 0, 3, u16);
    get_set_field!(tile_group_y, set_tile_group_y, 3, 3, u16);
    get_set_field!(n, set_n, 10, 2, u16);
}

#[derive(Default)]
pub struct PatternAddr {
    pub data: u16,
}

impl PatternAddr {
    get_set_field!(fine_y, set_fine_y, 0, 3, u16);
    get_set_field!(p, set_p, 3, 1, u16);
    get_set_field!(tile, set_tile, 4, 8, u16);
    get_set_field!(h, set_h, 12, 1, u16);
}

#[derive(Default)]
pub struct PaletteAddr {
    pub data: u16,
}

impl PaletteAddr {
    get_set_field!(p0, set_p0, 0, 1, u16);
    get_set_field!(p1, set_p1, 1, 1, u16);
    get_set_field!(a0, set_a0, 2, 1, u16);
    get_set_field!(a1, set_a1, 3, 1, u16);
    get_set_field!(s, set_s, 4, 1, u16);
}

#[derive(Default)]
pub struct SpriteAttr {
    pub data: u8,
}

impl SpriteAttr {
    get_set_field!(palette, set_palette, 0, 2, u8);
    get_set_field!(palette0, set_palette0, 0, 1, u8);
    get_set_field!(palette1, set_palette1, 1, 1, u8);
    get_set_field!(priority, set_priority, 5, 1, u8);
    get_set_field!(flip_hor, set_flip_hor, 6, 1, u8);
    get_set_field!(flip_ver, set_flip_ver, 7, 1, u8);
}
