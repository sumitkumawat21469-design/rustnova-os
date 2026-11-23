//! 32-bit ARGB Color Palette for Modern Graphical Desktop.

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Color(pub u32);

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }

    pub const fn argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }

    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    
    // Modern Dark Theme Palette
    pub const DESKTOP_BG_TOP: Color = Color::rgb(15, 23, 42);      // Slate 900
    pub const DESKTOP_BG_BOTTOM: Color = Color::rgb(30, 41, 59);   // Slate 800
    pub const TASKBAR_BG: Color = Color::rgb(15, 23, 42);          // Deep navy
    pub const TASKBAR_BORDER: Color = Color::rgb(51, 65, 85);      // Slate 700
    
    pub const WINDOW_BG: Color = Color::rgb(30, 41, 59);           // Slate 800
    pub const WINDOW_TITLEBAR: Color = Color::rgb(51, 65, 85);     // Slate 700
    pub const WINDOW_BORDER: Color = Color::rgb(71, 85, 105);      // Slate 600
    pub const WINDOW_TEXT: Color = Color::rgb(241, 245, 249);      // Slate 100
    pub const MUTED_TEXT: Color = Color::rgb(148, 163, 184);       // Slate 400
    
    // UI Accents
    pub const ACCENT_CYAN: Color = Color::rgb(6, 182, 212);        // Cyan 500
    pub const ACCENT_BLUE: Color = Color::rgb(59, 130, 246);       // Blue 500
    pub const ACCENT_PURPLE: Color = Color::rgb(168, 85, 247);     // Purple 500
    pub const ACCENT_GREEN: Color = Color::rgb(34, 197, 94);       // Green 500
    pub const ACCENT_YELLOW: Color = Color::rgb(234, 179, 8);      // Yellow 500
    pub const ACCENT_RED: Color = Color::rgb(239, 68, 68);         // Red 500
    
    pub const BTN_CLOSE: Color = Color::rgb(239, 68, 68);          // Window Close Red
    pub const BTN_MIN: Color = Color::rgb(234, 179, 8);            // Window Min Yellow
    pub const BTN_MAX: Color = Color::rgb(34, 197, 94);            // Window Max Green
}
