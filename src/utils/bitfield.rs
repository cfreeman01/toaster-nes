macro_rules! field {
    ($val:expr, $pos:expr, $width:expr) => {{
        let mut mask = 0;
        for _ in 0..$width {
            mask <<= 1;
            mask |= 0x1;
        }
        ($val >> $pos) & mask
    }};
}

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

pub(crate) use field;
pub(crate) use get_set_field;
pub(crate) use mask;
