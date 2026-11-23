//! File Manager Graphical Application.
//!
//! Renders file hierarchy, directory browsing, file sizes, and permission flags.

use crate::graphics::color::Color;
use crate::graphics::primitives::GFX;
use crate::filesystem::ROOT_FILESYSTEM;

pub struct FileManagerApp;

impl FileManagerApp {
    pub fn render_window(x: u32, y: u32, w: u32, h: u32) {
        let gfx = unsafe { &mut GFX };
        gfx.draw_window(x, y, w, h, "MicroFS File Explorer - /", false);

        let content_x = x + 12;
        let mut cur_y = y + 36;

        gfx.draw_string(content_x, cur_y, "PATH: / (Root File System - MicroFS)", Color::ACCENT_CYAN);
        cur_y += 18;
        gfx.draw_string(content_x, cur_y, "PERMS       TYPE    SIZE (B)   NAME", Color::ACCENT_YELLOW);
        cur_y += 16;

        for entry in ROOT_FILESYSTEM.iter() {
            let type_str = if entry.is_dir { "DIR " } else { "FILE" };
            let color = if entry.is_dir { Color::ACCENT_CYAN } else { Color::WINDOW_TEXT };

            // Render line
            gfx.draw_string(content_x, cur_y, entry.permissions, Color::MUTED_TEXT);
            gfx.draw_string(content_x + 96, cur_y, type_str, Color::ACCENT_YELLOW);
            
            // Render path name
            gfx.draw_string(content_x + 192, cur_y, entry.path, color);

            cur_y += 16;
            if cur_y > y + h - 20 {
                break;
            }
        }
    }
}
