pub(super) fn apply_fxaa_like_filter(width: u32, height: u32, rgba8: &[u8]) -> Vec<u8> {
    if width == 0 || height == 0 || rgba8.len() != (width * height * 4) as usize {
        return rgba8.to_vec();
    }

    let mut output = rgba8.to_vec();
    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let north = luminance(rgba8, width, x, y - 1);
            let south = luminance(rgba8, width, x, y + 1);
            let west = luminance(rgba8, width, x - 1, y);
            let east = luminance(rgba8, width, x + 1, y);
            let contrast =
                (north.max(south).max(west).max(east) - north.min(south).min(west).min(east)).abs();

            if contrast < 0.125 {
                continue;
            }

            let idx = ((y * width + x) * 4) as usize;
            for channel in 0..3 {
                let blended = (rgba8[idx + channel] as f32 * 0.5)
                    + (rgba8[idx + channel - 4] as f32 * 0.125)
                    + (rgba8[idx + channel + 4] as f32 * 0.125)
                    + (rgba8[idx + channel - (width as usize * 4)] as f32 * 0.125)
                    + (rgba8[idx + channel + (width as usize * 4)] as f32 * 0.125);
                output[idx + channel] = blended.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    output
}

fn luminance(rgba8: &[u8], width: u32, x: u32, y: u32) -> f32 {
    let idx = ((y * width + x) * 4) as usize;
    let r = rgba8[idx] as f32 / 255.0;
    let g = rgba8[idx + 1] as f32 / 255.0;
    let b = rgba8[idx + 2] as f32 / 255.0;
    0.2126 * r + 0.7152 * g + 0.0722 * b
}
