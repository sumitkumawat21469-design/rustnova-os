//! VGA Hardware Text Buffer & Status Driver.

use crate::kernel::arch::outb;

const VGA_MISC_PORT: u16 = 0x3C2;
const VGA_CRTC_INDEX: u16 = 0x3D4;
const VGA_CRTC_DATA: u16 = 0x3D5;

pub struct VgaController;

impl VgaController {
    pub fn init() {
        // Safe VGA cursor configuration
        unsafe {
            outb(VGA_CRTC_INDEX, 0x0A);
            outb(VGA_CRTC_DATA, 0x20); // Disable hardware blinking cursor if needed
        }
    }
}
