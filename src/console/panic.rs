//! Rust Kernel Panic Handler.
//!
//! Captures fatal unrecoverable errors, displays panic diagnostics,
//! source location, and safely halts the processor without restarting.

use core::panic::PanicInfo;
use crate::println;
use crate::kernel::arch::hlt;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n\x1b[41;37m=======================================================\x1b[0m");
    println!("\x1b[1;31m [FATAL KERNEL PANIC] SYSTEM EXECUTION HALTED \x1b[0m");
    println!("\x1b[41;37m=======================================================\x1b[0m");
    
    if let Some(location) = info.location() {
        println!("[PANIC LOCATION] File: {}:{}", location.file(), location.line());
    } else {
        println!("[PANIC LOCATION] Unknown source coordinates");
    }

    println!("[PANIC REASON] {}", info.message());
    println!("[SYSTEM STATE] CPU interrupts disabled, system halting.");
    println!("=======================================================\n");

    loop {
        hlt();
    }
}
