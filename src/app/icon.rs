/// A tiny hand-drawn 8x8 bitmap glyph for the letter "R", used to build
/// both the tray icon and the main window icon at runtime -- no external
/// .ico/.png asset needed. 1 = foreground pixel, 0 = background.
const GLYPH_R: [[u8; 8]; 8] = [
    [1, 1, 1, 1, 0, 0, 0, 0],
    [1, 0, 0, 0, 1, 0, 0, 0],
    [1, 0, 0, 0, 1, 0, 0, 0],
    [1, 1, 1, 1, 0, 0, 0, 0],
    [1, 0, 1, 0, 0, 0, 0, 0],
    [1, 0, 0, 1, 0, 0, 0, 0],
    [1, 0, 0, 0, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0],
];

/// Renders `GLYPH_R` as RGBA8 bytes, upscaled by `scale` (e.g. `scale = 4`
/// -> a 32x32 image), filled with `bg` and drawn in `fg`. Returns
/// `(rgba_bytes, width, height)`.
pub fn render_icon_rgba(scale: u32, bg: [u8; 4], fg: [u8; 4]) -> (Vec<u8>, u32, u32) {
    let size = 8 * scale;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    for pixel in rgba.chunks_exact_mut(4) {
        pixel.copy_from_slice(&bg);
    }

    for (gy, row) in GLYPH_R.iter().enumerate() {
        for (gx, &on) in row.iter().enumerate() {
            if on != 1 {
                continue;
            }
            for dy in 0..scale {
                for dx in 0..scale {
                    let x = gx as u32 * scale + dx;
                    let y = gy as u32 * scale + dy;
                    let idx = ((y * size + x) * 4) as usize;
                    rgba[idx..idx + 4].copy_from_slice(&fg);
                }
            }
        }
    }

    (rgba, size, size)
}

/// Shared color scheme for the "R" icon: dark red background, gold glyph.
pub const ICON_BG: [u8; 4] = [139, 0, 0, 255];
pub const ICON_FG: [u8; 4] = [255, 215, 0, 255];
