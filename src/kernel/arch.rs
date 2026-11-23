//! Architecture-specific x86_64 CPU instructions and assembly primitives.
//!
//! Provides low-level port I/O, CPU halt, interrupt flags manipulation,
//! and timestamp counter access for RUSTNOVA OS.

#![allow(dead_code)]

/// Writes an 8-bit byte to the specified I/O port.
///
/// # Safety
/// Port I/O can cause hardware side-effects. The caller must ensure the port address is valid.
#[inline(always)]
pub unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
    );
}

/// Reads an 8-bit byte from the specified I/O port.
///
/// # Safety
/// Reading from I/O ports may alter hardware controller state.
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") val,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    val
}

/// Writes a 16-bit word to the specified I/O port.
#[inline(always)]
pub unsafe fn outw(port: u16, val: u16) {
    core::arch::asm!(
        "out dx, ax",
        in("dx") port,
        in("ax") val,
        options(nomem, nostack, preserves_flags)
    );
}

/// Reads a 16-bit word from the specified I/O port.
#[inline(always)]
pub unsafe fn inw(port: u16) -> u16 {
    let val: u16;
    core::arch::asm!(
        "in ax, dx",
        out("ax") val,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    val
}

/// Writes a 32-bit dword to the specified I/O port.
#[inline(always)]
pub unsafe fn outl(port: u16, val: u32) {
    core::arch::asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") val,
        options(nomem, nostack, preserves_flags)
    );
}

/// Reads a 32-bit dword from the specified I/O port.
#[inline(always)]
pub unsafe fn inl(port: u16) -> u32 {
    let val: u32;
    core::arch::asm!(
        "in eax, dx",
        out("eax") val,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    val
}

/// Halts the CPU until the next external interrupt occurs.
#[inline(always)]
pub fn hlt() {
    unsafe {
        core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}

/// Disables CPU maskable interrupts (CLI).
#[inline(always)]
pub fn disable_interrupts() {
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
    }
}

/// Enables CPU maskable interrupts (STI).
#[inline(always)]
pub fn enable_interrupts() {
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
    }
}

/// Executes a CPU pause hint instruction for busy-wait loops.
#[inline(always)]
pub fn spin_hint() {
    core::hint::spin_loop();
}

/// Reads the CPU 64-bit Time Stamp Counter (RDTSC).
#[inline(always)]
pub fn rdtsc() -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        core::arch::asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
    ((high as u64) << 32) | (low as u64)
}
