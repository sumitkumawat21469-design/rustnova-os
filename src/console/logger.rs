//! Kernel Logging Subsystem.
//!
//! Implements multi-level logging (DEBUG, INFO, WARN, ERROR) with timestamps,
//! ANSI colored serial output, and formatted printing macros.

use core::fmt;
use crate::drivers::serial::serial_write_fmt;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u8)]
pub enum LogLevel {
    Debug = 0,
    Info  = 1,
    Warn  = 2,
    Error = 3,
}

impl LogLevel {
    pub const fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info  => "INFO ",
            LogLevel::Warn  => "WARN ",
            LogLevel::Error => "ERROR",
        }
    }

    pub const fn color_prefix(&self) -> &'static str {
        match self {
            LogLevel::Debug => "\x1b[36m", // Cyan
            LogLevel::Info  => "\x1b[32m", // Green
            LogLevel::Warn  => "\x1b[33m", // Yellow
            LogLevel::Error => "\x1b[31m", // Red
        }
    }
}

pub static mut MIN_LOG_LEVEL: LogLevel = LogLevel::Debug;

pub fn set_min_log_level(level: LogLevel) {
    unsafe {
        MIN_LOG_LEVEL = level;
    }
}

pub fn log(level: LogLevel, module: &str, args: fmt::Arguments) {
    unsafe {
        if level < MIN_LOG_LEVEL {
            return;
        }
    }
    
    // We print format: [TIMESTAMP] [LEVEL] [MODULE] message\n
    // ANSI color: \x1b[32m[INFO ]\x1b[0m [MODULE] message
    serial_write_fmt(format_args!(
        "{}[{}]\x1b[0m [{}] {}\n",
        level.color_prefix(),
        level.as_str(),
        module,
        args
    ));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::drivers::serial::serial_write_fmt(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($module:expr, $($arg:tt)*) => {
        $crate::console::logger::log($crate::console::logger::LogLevel::Debug, $module, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($module:expr, $($arg:tt)*) => {
        $crate::console::logger::log($crate::console::logger::LogLevel::Info, $module, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($module:expr, $($arg:tt)*) => {
        $crate::console::logger::log($crate::console::logger::LogLevel::Warn, $module, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($module:expr, $($arg:tt)*) => {
        $crate::console::logger::log($crate::console::logger::LogLevel::Error, $module, format_args!($($arg)*))
    };
}
