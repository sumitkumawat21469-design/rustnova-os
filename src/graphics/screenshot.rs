//! BMP Screenshot Generator.
//!
//! Encodes the GOP framebuffer into a standard 24-bit Windows Bitmap (BMP) file
//! for inspection, archival, and documentation deliverables.

use crate::graphics::primitives::GFX;

#[repr(C, packed)]
struct BmpFileHeader {
    file_type: u16,   // "BM" = 0x4D42
    file_size: u32,
    reserved1: u16,
    reserved2: u16,
    offset_data: u32,
}

#[repr(C, packed)]
struct BmpInfoHeader {
    size: u32,        // 40
    width: i32,
    height: i32,
    planes: u16,      // 1
    bit_count: u16,   // 24
    compression: u32, // 0
    size_image: u32,
    x_pels_per_meter: i32,
    y_pels_per_meter: i32,
    clr_used: u32,
    clr_important: u32,
}

/// Renders a scaled 800x600 BMP image from the framebuffer into a destination byte buffer.
/// Returns number of bytes written.
pub fn capture_screenshot_bmp(dest: &mut [u8]) -> usize {
    let width = unsafe { GFX.width }.min(800) as usize;
    let height = unsafe { GFX.height }.min(600) as usize;
    let stride = unsafe { GFX.stride } as usize;
    let fb = unsafe { GFX.fb_ptr };

    if fb.is_null() || width == 0 || height == 0 {
        return 0;
    }

    let row_size = (width * 3 + 3) & !3; // 4-byte aligned row
    let image_size = row_size * height;
    let file_size = 54 + image_size;

    if dest.len() < file_size {
        return 0;
    }

    // Write File Header (14 bytes)
    dest[0] = b'B';
    dest[1] = b'M';
    dest[2..6].copy_from_slice(&(file_size as u32).to_le_bytes());
    dest[6..10].copy_from_slice(&[0, 0, 0, 0]);
    dest[10..14].copy_from_slice(&54u32.to_le_bytes());

    // Write Info Header (40 bytes)
    dest[14..18].copy_from_slice(&40u32.to_le_bytes());
    dest[18..22].copy_from_slice(&(width as i32).to_le_bytes());
    dest[22..26].copy_from_slice(&(height as i32).to_le_bytes()); // Bottom-up
    dest[26..28].copy_from_slice(&1u16.to_le_bytes());
    dest[28..30].copy_from_slice(&24u16.to_le_bytes());
    dest[30..34].copy_from_slice(&0u32.to_le_bytes());
    dest[34..38].copy_from_slice(&(image_size as u32).to_le_bytes());
    dest[38..42].copy_from_slice(&2835i32.to_le_bytes()); // 72 DPI
    dest[42..46].copy_from_slice(&2835i32.to_le_bytes());
    dest[46..50].copy_from_slice(&0u32.to_le_bytes());
    dest[50..54].copy_from_slice(&0u32.to_le_bytes());

    // Write Pixel Data (bottom-to-top)
    let mut offset = 54;
    for y in (0..height).rev() {
        for x in 0..width {
            let pixel_val = unsafe { fb.add(y * stride + x).read_volatile() };
            let b = (pixel_val & 0xFF) as u8;
            let g = ((pixel_val >> 8) & 0xFF) as u8;
            let r = ((pixel_val >> 16) & 0xFF) as u8;

            dest[offset] = b;
            dest[offset + 1] = g;
            dest[offset + 2] = r;
            offset += 3;
        }
        // Pad to 4-byte boundary
        let pad = row_size - (width * 3);
        for _ in 0..pad {
            dest[offset] = 0;
            offset += 1;
        }
    }

    file_size
}
