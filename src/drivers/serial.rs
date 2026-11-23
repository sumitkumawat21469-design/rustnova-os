//! 16550 UART Serial Driver for COM1 (0x3F8).
//!
//! Provides hardware-level serial output for debugging, kernel traces,
//! and automated testing in QEMU.

use crate::kernel::arch::{inb, outb};
use core::fmt::{self, Write};

/// COM1 base I/O port.
const COM1: u16 = 0x3F8;

/// Global serial port driver instance.
pub struct SerialPort {
    port: u16,
}

impl SerialPort {
    /// Creates a new SerialPort abstraction at the specified I/O port base.
    pub const fn new(port: u16) -> Self {
        Self { port }
    }

    /// Initializes the 16550 UART serial controller:
    /// - Sets baud rate divisor to 1 (115200 baud)
    /// - Configures 8 bits, no parity, 1 stop bit (8N1)
    /// - Enables FIFO with 14-byte threshold
    /// - Sets RTS/DTR pins
    pub fn init(&mut self) {
        unsafe {
            // Disable all interrupts
            outb(self.port + 1, 0x00);
            // Enable DLAB (set baud rate divisor)
            outb(self.port + 3, 0x80);
            // Set divisor to 1 (lo byte) -> 115200 baud
            outb(self.port + 0, 0x01);
            // Set divisor hi byte
            outb(self.port + 1, 0x00);
            // 8 bits, no parity, 1 stop bit
            outb(self.port + 3, 0x03);
            // Enable FIFO, clear them, with 14-byte threshold
            outb(self.port + 2, 0xC7);
            // IRQs enabled, RTS/DSR set
            outb(self.port + 4, 0x0B);
        }
    }

    /// Checks if the transmitter holding register is empty.
    #[inline(always)]
    fn is_transmit_empty(&self) -> bool {
        unsafe { (inb(self.port + 5) & 0x20) != 0 }
    }

    /// Sends a single byte over the serial port.
    pub fn send_byte(&mut self, byte: u8) {
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }
        unsafe {
            outb(self.port, byte);
        }
    }

    /// Sends a string slice over the serial port.
    pub fn send_str(&mut self, s: &str) {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.send_byte(b'\r');
            }
            self.send_byte(byte);
        }
    }
}

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.send_str(s);
        Ok(())
    }
}

/// Global static COM1 serial driver.
pub static mut SERIAL_COM1: SerialPort = SerialPort::new(COM1);

/// Helper function to write formatted text to COM1 without heap allocation.
pub fn serial_write_fmt(args: fmt::Arguments) {
    unsafe {
        let _ = SERIAL_COM1.write_fmt(args);
    }
}

/// Helper function to print a string directly to COM1.
pub fn serial_print(s: &str) {
    unsafe {
        SERIAL_COM1.send_str(s);
    }
}
