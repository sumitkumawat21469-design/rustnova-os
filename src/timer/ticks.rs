//! Monotonic System Tick and Timer Accounting Engine.
//!
//! Maintains high-resolution 64-bit tick counter, calculates uptime in milliseconds,
//! and notifies the scheduler when time-slice boundaries are crossed.

use core::sync::atomic::{AtomicU64, Ordering};

/// Monotonic system tick count since boot.
static SYSTEM_TICKS: AtomicU64 = AtomicU64::new(0);

/// Monotonic timer engine.
pub struct TimerEngine;

impl TimerEngine {
    /// Returns current system tick count.
    #[inline(always)]
    pub fn current_tick() -> u64 {
        SYSTEM_TICKS.load(Ordering::Relaxed)
    }

    /// Increments monotonic system tick by 1.
    #[inline(always)]
    pub fn advance_tick() -> u64 {
        SYSTEM_TICKS.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Resets tick counter (for benchmark runs).
    pub fn reset() {
        SYSTEM_TICKS.store(0, Ordering::SeqCst);
    }

    /// Returns elapsed uptime in milliseconds assuming 100 Hz timer tick (10 ms per tick).
    pub fn uptime_ms() -> u64 {
        Self::current_tick() * 10
    }
}
