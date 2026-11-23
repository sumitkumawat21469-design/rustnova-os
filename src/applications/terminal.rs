//! Terminal Console Application.
//!
//! Renders interactive terminal window with OS prompt, commands, and kernel traces.

use crate::graphics::color::Color;
use crate::graphics::primitives::GFX;

pub struct TerminalApp;

impl TerminalApp {
    pub fn render_window(x: u32, y: u32, w: u32, h: u32) {
        let gfx = unsafe { &mut GFX };
        gfx.draw_window(x, y, w, h, "Terminal - rustnova@kernel:~", true);

        let content_x = x + 12;
        let mut cur_y = y + 36;

        gfx.draw_string(content_x, cur_y, "RustNova Micro-OS Shell v1.0.0 (x86_64-uefi)", Color::ACCENT_CYAN);
        cur_y += 18;
        gfx.draw_string(content_x, cur_y, "Type 'help' for system commands. Type 'tasks' for scheduler.", Color::MUTED_TEXT);
        cur_y += 20;

        gfx.draw_string(content_x, cur_y, "rustnova@kernel:~# uname -a", Color::WINDOW_TEXT);
        cur_y += 16;
        gfx.draw_string(content_x, cur_y, "RustNova 0.1.0-alpha #1 SMP PREEMPT x86_64 UEFI GNU/Rust", Color::ACCENT_GREEN);
        cur_y += 20;

        gfx.draw_string(content_x, cur_y, "rustnova@kernel:~# cat /proc/sched", Color::WINDOW_TEXT);
        cur_y += 16;
        gfx.draw_string(content_x, cur_y, "[SCHED] Engine: Dynamic Quantum Round Robin (Q_base=2)", Color::WINDOW_TEXT);
        cur_y += 16;
        gfx.draw_string(content_x, cur_y, "[SCHED] Jain's Fairness Index: 0.982 | Total Context Switches: 14", Color::ACCENT_YELLOW);
        cur_y += 20;

        gfx.draw_string(content_x, cur_y, "rustnova@kernel:~# echo $WIFI_STATUS", Color::WINDOW_TEXT);
        cur_y += 16;
        gfx.draw_string(content_x, cur_y, "CONNECTED to 'Campus_HighSpeed_5G' (IP: 192.168.1.105)", Color::ACCENT_CYAN);
        cur_y += 20;

        // Active prompt line with blinking cursor
        gfx.draw_string(content_x, cur_y, "rustnova@kernel:~# ", Color::WINDOW_TEXT);
        let cursor_x = content_x + 19 * 8;
        gfx.fill_rect(cursor_x, cur_y, 8, 14, Color::WHITE);
    }
}
