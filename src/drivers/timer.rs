//! Intel 8254 Programmable Interval Timer (PIT) Driver.
//!
//! Configures hardware periodic timer interrupts at configurable frequency (e.g., 100 Hz),
//! driving the scheduler time quantum and monotonic system tick counter.

use crate::kernel::arch::outb;
use crate::log_info;

const PIT_CHANNEL_0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
const PIT_BASE_FREQUENCY: u32 = 1193182;

/// Periodic Timer Hardware Controller.
pub struct PitTimer {
    frequency_hz: u32,
    divisor: u16,
}

impl PitTimer {
    pub const fn new() -> Self {
        Self {
            frequency_hz: 100,
            divisor: 11931,
        }
    }

    /// Initializes Channel 0 in Mode 3 (Square Wave Generator).
    pub fn init(&mut self, target_hz: u32) {
        let divisor = (PIT_BASE_FREQUENCY / target_hz).clamp(1, 65535) as u16;
        self.frequency_hz = target_hz;
        self.divisor = divisor;

        unsafe {
            // Command byte:
            // 00 (Channel 0) | 11 (Access mode: low then high byte) | 011 (Mode 3 square wave) | 0 (16-bit binary)
            // = 0x36
            outb(PIT_COMMAND, 0x36);
            // Low byte
            outb(PIT_CHANNEL_0, (divisor & 0xFF) as u8);
            // High byte
            outb(PIT_CHANNEL_0, ((divisor >> 8) & 0xFF) as u8);
        }

        log_info!(
            "TIMER",
            "PIT 8254 initialized at {} Hz (divisor: {})",
            target_hz,
            divisor
        );
    }

    pub fn frequency(&self) -> u32 {
        self.frequency_hz
    }
}

pub static mut HARDWARE_TIMER: PitTimer = PitTimer::new();
