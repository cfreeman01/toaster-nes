#[derive(Copy, Clone, PartialEq)]
pub enum Button {
    A = 0x01,
    B = 0x02,
    Select = 0x04,
    Start = 0x08,
    Up = 0x10,
    Down = 0x20,
    Left = 0x40,
    Right = 0x80,
}

#[derive(Default)]
pub struct Controller {
    pub strobe: bool,
    buttons_dyn: u8,
    buttons_latched: u8,
}

impl Controller {
    pub fn set_button_state(&mut self, button: Button, pressed: bool) {
        let button = button as u8;

        if pressed {
            self.buttons_dyn |= button;
        } else {
            self.buttons_dyn &= !button;
        }
    }

    pub fn latch_buttons(&mut self) {
        self.buttons_latched = self.buttons_dyn;
    }

    pub fn read(&mut self) -> u8 {
        let ret = self.buttons_latched & 0x1;
        self.buttons_latched >>= 1;
        ret
    }
}
