//! Modern Graphical Desktop Subsystem (Phase 8).
//!
//! Renders:
//! - Desktop Wallpaper with gradient and geometric OS branding
//! - Modern bottom Taskbar with Start Button, App Launcher, Tray, and Clock
//! - Start Menu popup with application catalog
//! - Multiple simultaneous desktop windows:
//!   - System Monitor Dashboard (Phase 11)
//!   - Terminal Shell (Phase 8)
//!   - Application Store & Wi-Fi Management Suite (Phases 9 & 10)

use crate::applications::{SystemMonitor, TerminalApp, GLOBAL_APP_STORE};
use crate::graphics::color::Color;
use crate::graphics::primitives::GFX;
use crate::network::SYSTEM_WIFI;

pub struct Desktop;

impl Desktop {
    /// Renders the entire modern desktop interface to the active framebuffer.
    pub fn render() {
        let gfx = unsafe { &mut GFX };
        let screen_w = gfx.width;
        let screen_h = gfx.height;

        if screen_w == 0 || screen_h == 0 {
            return;
        }

        let taskbar_h = 44u32;
        let desktop_h = screen_h.saturating_sub(taskbar_h);

        // 1. DESKTOP WALLPAPER: Slate gradient background
        gfx.draw_gradient_v(0, 0, screen_w, desktop_h, Color::DESKTOP_BG_TOP, Color::DESKTOP_BG_BOTTOM);

        // Decorative background geometric accents and OS Brand
        gfx.draw_string(
            screen_w / 2 - 140,
            40,
            "RUSTNOVA MICRO-OPERATING SYSTEM",
            Color::rgb(71, 85, 105),
        );
        gfx.draw_string(
            screen_w / 2 - 120,
            58,
            "Educational x86_64 Kernel Environment",
            Color::rgb(51, 65, 85),
        );

        // 2. WINDOW 1 (Left Pane): SYSTEM MONITOR DASHBOARD (Phase 11)
        let win1_w = (screen_w * 48) / 100;
        let win1_h = desktop_h.saturating_sub(60);
        SystemMonitor::render_window(20, 24, win1_w, win1_h);

        // 3. WINDOW 2 (Top Right): TERMINAL CONSOLE (Phase 8)
        let win2_x = win1_w + 36;
        let win2_w = screen_w.saturating_sub(win2_x + 20);
        let win2_h = (desktop_h * 48) / 100;
        TerminalApp::render_window(win2_x, 24, win2_w, win2_h);

        // 4. WINDOW 3 (Bottom Right): APPLICATION STORE & WI-FI (Phases 9 & 10)
        let win3_y = win2_h + 38;
        let win3_h = desktop_h.saturating_sub(win3_y + 12);
        Self::render_store_and_wifi_window(win2_x, win3_y, win2_w, win3_h);

        // 5. BOTTOM TASKBAR (Phase 8)
        let taskbar_y = desktop_h;
        gfx.fill_rect(0, taskbar_y, screen_w, taskbar_h, Color::TASKBAR_BG);
        gfx.fill_rect(0, taskbar_y, screen_w, 2, Color::TASKBAR_BORDER);

        // Start Menu Button
        gfx.fill_rect(10, taskbar_y + 6, 110, 32, Color::ACCENT_BLUE);
        gfx.draw_string(20, taskbar_y + 14, "[*] RustNova", Color::WHITE);

        // Application Launcher Quick Icons
        gfx.fill_rect(130, taskbar_y + 6, 90, 32, Color::rgb(30, 41, 59));
        gfx.draw_string(140, taskbar_y + 14, ">_ Terminal", Color::WINDOW_TEXT);

        gfx.fill_rect(228, taskbar_y + 6, 80, 32, Color::rgb(30, 41, 59));
        gfx.draw_string(238, taskbar_y + 14, "# Files", Color::WINDOW_TEXT);

        gfx.fill_rect(316, taskbar_y + 6, 95, 32, Color::rgb(30, 41, 59));
        gfx.draw_string(326, taskbar_y + 14, "||| Monitor", Color::WINDOW_TEXT);

        gfx.fill_rect(419, taskbar_y + 6, 80, 32, Color::rgb(30, 41, 59));
        gfx.draw_string(429, taskbar_y + 14, "+ Store", Color::WINDOW_TEXT);

        gfx.fill_rect(507, taskbar_y + 6, 85, 32, Color::rgb(30, 41, 59));
        gfx.draw_string(517, taskbar_y + 14, "% Settings", Color::WINDOW_TEXT);

        // System Tray: Wi-Fi Indicator, Memory Gauge, Clock
        let tray_x = screen_w.saturating_sub(340);
        
        // Wi-Fi Status Badge
        gfx.fill_rect(tray_x, taskbar_y + 8, 120, 26, Color::rgb(30, 41, 59));
        gfx.draw_string(tray_x + 8, taskbar_y + 14, "Wi-Fi: 94%", Color::ACCENT_CYAN);

        // RAM Usage Badge
        gfx.fill_rect(tray_x + 128, taskbar_y + 8, 80, 26, Color::rgb(30, 41, 59));
        gfx.draw_string(tray_x + 136, taskbar_y + 14, "RAM: 18%", Color::ACCENT_GREEN);

        // Digital Clock (Phase 8 requirement)
        gfx.fill_rect(tray_x + 216, taskbar_y + 8, 110, 26, Color::rgb(30, 41, 59));
        gfx.draw_string(tray_x + 224, taskbar_y + 14, "13:25:10 UTC", Color::WHITE);
    }

    /// Renders Window 3: Custom Application Store & Wi-Fi Control Hub.
    fn render_store_and_wifi_window(x: u32, y: u32, w: u32, h: u32) {
        let gfx = unsafe { &mut GFX };
        gfx.draw_window(x, y, w, h, "Software Store & Network Manager", false);

        let content_x = x + 12;
        let mut cur_y = y + 36;

        // SUB-PANEL A: APPLICATION STORE (Phase 9)
        gfx.draw_string(content_x, cur_y, "[NOVA SOFTWARE STORE - REPOSITORY]", Color::ACCENT_CYAN);
        cur_y += 18;
        gfx.draw_string(
            content_x,
            cur_y,
            "Categories: [System Tools] [Productivity] [Development] [Utilities]",
            Color::ACCENT_YELLOW,
        );
        cur_y += 18;

        let store = unsafe { &GLOBAL_APP_STORE };
        for (_i, pkg) in store.packages[..3].iter().enumerate() {
            let status_str = if pkg.is_installed { "[INSTALLED]" } else { "[AVAILABLE]" };
            let status_color = if pkg.is_installed { Color::ACCENT_GREEN } else { Color::ACCENT_CYAN };

            gfx.draw_string(content_x, cur_y, pkg.name, Color::WINDOW_TEXT);
            gfx.draw_string(content_x + 170, cur_y, pkg.version, Color::MUTED_TEXT);
            gfx.draw_string(content_x + 230, cur_y, status_str, status_color);
            cur_y += 16;
        }

        gfx.draw_string(
            content_x,
            cur_y,
            "* Custom Educational Store (Not affiliated with Snap Store)",
            Color::MUTED_TEXT,
        );
        cur_y += 24;

        // SUB-PANEL B: WI-FI NETWORKING MANAGER (Phase 10)
        gfx.draw_string(content_x, cur_y, "[WI-FI 802.11ax WIRELESS MANAGER]", Color::ACCENT_CYAN);
        cur_y += 18;

        let wifi = unsafe { &SYSTEM_WIFI };
        for net in wifi.networks.iter() {
            let conn_mark = if net.is_connected { "► CONNECTED" } else { "  Available" };
            let mark_color = if net.is_connected { Color::ACCENT_GREEN } else { Color::MUTED_TEXT };

            gfx.draw_string(content_x, cur_y, net.ssid, Color::WINDOW_TEXT);
            gfx.draw_string(content_x + 180, cur_y, net.security, Color::MUTED_TEXT);
            gfx.draw_string(content_x + 290, cur_y, conn_mark, mark_color);
            cur_y += 16;
            if cur_y > y + h - 24 {
                break;
            }
        }

        gfx.draw_string(
            content_x,
            cur_y,
            "Status: IP 192.168.1.105 (Simulated 802.11ax Driver)",
            Color::ACCENT_GREEN,
        );
    }
}
