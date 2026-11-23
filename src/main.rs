//! RustNova OS - Educational Micro-Operating System written in Rust.
//!
//! An educational micro-kernel implementing:
//! - UEFI Bootloader handover & GOP linear framebuffer
//! - Task Management & Process Accounting (Ready, Running, Blocked, Terminated)
//! - Preemptive Round Robin & Advanced Dynamic Time Quantum Scheduler
//! - PIT 8254 Periodic Timer Driver & Monotonic Tick Counter
//! - Atomic Spinlock Concurrency & Synchronization
//! - Linked-List Free-Block Memory Allocator & Failure Handling
//! - Modern Graphical Desktop Interface, App Store, Wi-Fi Manager & System Monitor

#![no_std]
#![no_main]
#![allow(static_mut_refs, dead_code)]

pub mod applications;
pub mod boot;
pub mod console;
pub mod drivers;
pub mod filesystem;
pub mod graphics;
pub mod kernel;
pub mod memory;
pub mod network;
pub mod scheduler;
pub mod sync;
pub mod task;
pub mod timer;

// Re-export efi_main as the bootloader entry point
pub use boot::efi_main;
