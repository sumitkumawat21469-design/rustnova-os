//! PS/2 8042 Keyboard Controller Driver.

use crate::kernel::arch::inb;

const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;

pub struct Keyboard;

impl Keyboard {
    pub fn has_data() -> bool {
        unsafe { (inb(PS2_STATUS_PORT) & 0x01) != 0 }
    }

    pub fn read_scancode() -> Option<u8> {
        if Self::has_data() {
            unsafe { Some(inb(PS2_DATA_PORT)) }
        } else {
            None
        }
    }
}
