//! 2D Graphics Drawing Primitives.
//!
//! Provides pixel operations, filled rectangles, borders, vertical gradients,
//! string rendering, and window chrome widgets.

use crate::graphics::color::Color;
use crate::graphics::font::{get_glyph, FONT_HEIGHT, FONT_WIDTH};

pub struct GraphicsEngine {
    pub fb_ptr: *mut u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}

impl GraphicsEngine {
    pub const fn new() -> Self {
        Self {
            fb_ptr: core::ptr::null_mut(),
            width: 0,
            height: 0,
            stride: 0,
        }
    }

    pub fn init(&mut self, fb_ptr: *mut u32, width: u32, height: u32, stride: u32) {
        self.fb_ptr = fb_ptr;
        self.width = width;
        self.height = height;
        self.stride = stride;
    }

    #[inline(always)]
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height && !self.fb_ptr.is_null() {
            let offset = (y * self.stride + x) as isize;
            unsafe {
                self.fb_ptr.offset(offset).write_volatile(color.0);
            }
        }
    }

    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        let max_x = (x + w).min(self.width);
        let max_y = (y + h).min(self.height);

        for py in y..max_y {
            for px in x..max_x {
                self.draw_pixel(px, py, color);
            }
        }
    }

    pub fn draw_rect_outline(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        if w == 0 || h == 0 {
            return;
        }
        for px in x..(x + w) {
            self.draw_pixel(px, y, color);
            self.draw_pixel(px, y + h - 1, color);
        }
        for py in y..(y + h) {
            self.draw_pixel(x, py, color);
            self.draw_pixel(x + w - 1, py, color);
        }
    }

    pub fn draw_gradient_v(&mut self, x: u32, y: u32, w: u32, h: u32, top: Color, bottom: Color) {
        if h == 0 {
            return;
        }
        let r1 = ((top.0 >> 16) & 0xFF) as i32;
        let g1 = ((top.0 >> 8) & 0xFF) as i32;
        let b1 = (top.0 & 0xFF) as i32;

        let r2 = ((bottom.0 >> 16) & 0xFF) as i32;
        let g2 = ((bottom.0 >> 8) & 0xFF) as i32;
        let b2 = (bottom.0 & 0xFF) as i32;

        for py in 0..h {
            let t = py as i32;
            let r = (r1 + (r2 - r1) * t / (h as i32)) as u8;
            let g = (g1 + (g2 - g1) * t / (h as i32)) as u8;
            let b = (b1 + (b2 - b1) * t / (h as i32)) as u8;
            let row_color = Color::rgb(r, g, b);

            for px in x..(x + w) {
                self.draw_pixel(px, y + py, row_color);
            }
        }
    }

    pub fn draw_char(&mut self, x: u32, y: u32, c: char, color: Color) {
        let glyph = get_glyph(c);
        for (row_idx, row_byte) in glyph.iter().enumerate() {
            let py = y + row_idx as u32;
            for col_idx in 0..8 {
                if (*row_byte & (0x80 >> col_idx)) != 0 {
                    let px = x + col_idx as u32;
                    self.draw_pixel(px, py, color);
                }
            }
        }
    }

    pub fn draw_string(&mut self, x: u32, y: u32, text: &str, color: Color) {
        let mut cur_x = x;
        let mut cur_y = y;

        for c in text.chars() {
            if c == '\n' {
                cur_x = x;
                cur_y += FONT_HEIGHT as u32;
            } else {
                self.draw_char(cur_x, cur_y, c, color);
                cur_x += FONT_WIDTH as u32;
            }
        }
    }

    /// Draws a modern desktop window with title bar, close/min/max buttons, and body.
    pub fn draw_window(&mut self, x: u32, y: u32, w: u32, h: u32, title: &str, active: bool) {
        let titlebar_h = 28u32;
        let header_color = if active { Color::WINDOW_TITLEBAR } else { Color::TASKBAR_BG };

        // Window drop shadow
        self.fill_rect(x + 4, y + 4, w, h, Color::rgb(10, 15, 26));

        // Window body
        self.fill_rect(x, y + titlebar_h, w, h - titlebar_h, Color::WINDOW_BG);

        // Window titlebar
        self.fill_rect(x, y, w, titlebar_h, header_color);

        // Window border outline
        self.draw_rect_outline(x, y, w, h, Color::WINDOW_BORDER);

        // Window control buttons (Red Close, Yellow Min, Green Max)
        self.fill_rect(x + 10, y + 8, 12, 12, Color::BTN_CLOSE);
        self.fill_rect(x + 28, y + 8, 12, 12, Color::BTN_MIN);
        self.fill_rect(x + 46, y + 8, 12, 12, Color::BTN_MAX);

        // Title text
        self.draw_string(x + 70, y + 6, title, Color::WINDOW_TEXT);
    }
}

pub static mut GFX: GraphicsEngine = GraphicsEngine::new();
