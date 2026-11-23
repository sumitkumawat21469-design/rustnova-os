//! Graphical Subsystem.

pub mod color;
pub mod desktop;
pub mod font;
pub mod primitives;
pub mod screenshot;

pub use color::Color;
pub use desktop::Desktop;
pub use font::{get_glyph, FONT_HEIGHT, FONT_WIDTH};
pub use primitives::{GraphicsEngine, GFX};
pub use screenshot::capture_screenshot_bmp;
