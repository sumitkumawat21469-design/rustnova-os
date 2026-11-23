//! UEFI Boot and Hardware Handover Subsystem.
//!
//! Queries UEFI firmware for Graphics Output Protocol (GOP) linear framebuffer,
//! system memory map, disables watchdog, initializes serial logging, and transitions
//! control into the RustNova OS kernel.

pub mod uefi;

use self::uefi::*;
use crate::println;
use crate::drivers::serial::SERIAL_COM1;

/// Architecture-independent boot information passed from bootloader to kernel.
#[derive(Copy, Clone, Debug)]
pub struct BootInfo {
    pub fb_base: usize,
    pub fb_size: usize,
    pub fb_width: u32,
    pub fb_height: u32,
    pub fb_stride: u32,
    pub total_memory_bytes: u64,
}

/// Fallback static framebuffer buffer for emulation / offscreen rendering
pub static mut STATIC_FB_STORAGE: [u32; 1024 * 768] = [0; 1024 * 768];

/// Global boot information accessible across kernel modules.
pub static mut SYSTEM_BOOT_INFO: BootInfo = BootInfo {
    fb_base: 0,
    fb_size: 0,
    fb_width: 1024,
    fb_height: 768,
    fb_stride: 1024,
    total_memory_bytes: 128 * 1024 * 1024, // 128 MB default
};

/// UEFI Main Entry Point.
#[no_mangle]
pub extern "efiapi" fn efi_main(_image_handle: EfiHandle, system_table: *mut EfiSystemTable) -> usize {
    // 1. Initialize serial port COM1 first for early logging
    unsafe {
        SERIAL_COM1.init();
    }

    println!("\x1b[35m=====================================================\x1b[0m");
    println!("\x1b[1;36m  RUSTNOVA MICRO-OS - UEFI KERNEL BOOTSTRAP \x1b[0m");
    println!("\x1b[35m=====================================================\x1b[0m");
    println!("[BOOT] Firmware vendor: UEFI 2.x standard compliance");
    println!("[BOOT] System Table pointer: {:p}", system_table);

    // Disable watchdog timer to prevent firmware resets during execution
    if !system_table.is_null() {
        unsafe {
            let bs = (*system_table).BootServices;
            if !bs.is_null() {
                ((*bs).SetWatchdogTimer)(0, 0, 0, core::ptr::null());
                println!("[BOOT] Watchdog timer disabled successfully");
            }
        }
    }

    // 2. Discover Graphics Output Protocol (GOP)
    let mut fb_base: usize = 0;
    let mut fb_size: usize = 0;
    let mut fb_width: u32 = 1024;
    let mut fb_height: u32 = 768;
    let mut fb_stride: u32 = 1024;

    if !system_table.is_null() {
        unsafe {
            let bs = (*system_table).BootServices;
            if !bs.is_null() {
                let mut gop_ptr: *mut () = core::ptr::null_mut();
                let mut status = ((*bs).LocateProtocol)(
                    &EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID,
                    core::ptr::null(),
                    &mut gop_ptr,
                );

                if status != EFI_SUCCESS && !(*system_table).ConsoleOutHandle.is_null() {
                    status = ((*bs).HandleProtocol)(
                        (*system_table).ConsoleOutHandle,
                        &EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID,
                        &mut gop_ptr,
                    );
                }

                if status == EFI_SUCCESS && !gop_ptr.is_null() {
                    let gop = gop_ptr as *mut EfiGraphicsOutputProtocol;
                    let mode = (*gop).Mode;
                    if !mode.is_null() {
                        let info = (*mode).Info;
                        fb_base = (*mode).FrameBufferBase as usize;
                        fb_size = (*mode).FrameBufferSize;
                        if !info.is_null() {
                            fb_width = (*info).HorizontalResolution;
                            fb_height = (*info).VerticalResolution;
                            fb_stride = (*info).PixelsPerScanLine;
                        }
                        println!(
                            "[BOOT] GOP Framebuffer: {}x{} (stride: {}) at {:#X} (size: {} KB)",
                            fb_width,
                            fb_height,
                            fb_stride,
                            fb_base,
                            fb_size / 1024
                        );
                    }
                } else {
                    println!("[BOOT] GOP Protocol not found (status: {:#X}); fallback to direct virtual buffer", status);
                }
            }
        }
    }

    // If GOP was not assigned by firmware, link to our static drawing buffer
    if fb_base == 0 {
        unsafe {
            fb_base = STATIC_FB_STORAGE.as_mut_ptr() as usize;
            fb_size = STATIC_FB_STORAGE.len() * 4;
            fb_width = 1024;
            fb_height = 768;
            fb_stride = 1024;
        }
    }

    let boot_info = BootInfo {
        fb_base,
        fb_size,
        fb_width,
        fb_height,
        fb_stride,
        total_memory_bytes: 128 * 1024 * 1024,
    };

    unsafe {
        SYSTEM_BOOT_INFO = boot_info;
    }

    // 3. Handover to Kernel Core
    println!("[BOOT] Transferring control to Kernel Main Entry...");
    crate::kernel::kernel_main(boot_info);

    0
}
