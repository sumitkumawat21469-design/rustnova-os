//! Console and Logging Module.

pub mod logger;
pub mod panic;

pub use logger::{log, set_min_log_level, LogLevel};
