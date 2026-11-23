//! Hardware Drivers Subsystem.

pub mod serial;
pub mod timer;
pub mod vga;
pub mod keyboard;

pub use serial::{SERIAL_COM1, serial_print, serial_write_fmt};
pub use timer::HARDWARE_TIMER;
pub use keyboard::Keyboard;
