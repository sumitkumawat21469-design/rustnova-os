//! System Settings Graphical Interface.
//!
//! Provides system configuration panels: Display, Memory, Network, and About OS.

use crate::graphics::color::Color;
use crate::graphics::primitives::GFX;

pub struct SettingsApp;

impl SettingsApp {
    pub fn render_window(x: u32, y: u32, w: u32, h: u32) {
        let gfx = unsafe { &mut GFX };
        gfx.draw_window(x, y, w, h, "System Settings", false);

        let content_x = x + 12;
        let mut cur_y = y + 36;

        gfx.draw_string(content_x, cur_y, "[SYSTEM CONFIGURATION]", Color::ACCENT_CYAN);
        cur_y += 20;

        gfx.draw_string(content_x, cur_y, "OS Version:     RustNova Micro-OS v0.1.0-alpha", Color::WINDOW_TEXT);
        cur_y += 18;
        gfx.draw_string(content_x, cur_y, "Kernel Build:   x86_64-unknown-uefi (nightly-gnu)", Color::WINDOW_TEXT);
        cur_y += 18;
        gfx.draw_string(content_x, cur_y, "Display Mode:   1024x768 Linear 32-bit Framebuffer", Color::WINDOW_TEXT);
        cur_y += 18;
        gfx.draw_string(content_x, cur_y, "Scheduler Mode: Dynamic Time Quantum Round Robin", Color::WINDOW_TEXT);
        cur_y += 18;
        gfx.draw_string(content_x, cur_y, "Memory Engine:  Coalescing Free-List Allocator (256 KB)", Color::WINDOW_TEXT);
        cur_y += 18;
        gfx.draw_string(content_x, cur_y, "Network Driver: Simulated 802.11ax Wi-Fi Stack", Color::ACCENT_GREEN);
    }
}
