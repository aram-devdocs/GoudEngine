//! Atlas packing statistics.

/// Statistics about a packed texture atlas.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtlasStats {
    /// Number of textures packed into the atlas.
    pub texture_count: u32,
    /// Atlas width in pixels.
    pub width: u32,
    /// Atlas height in pixels.
    pub height: u32,
    /// Total pixel area consumed by packed textures.
    pub used_pixels: u64,
    /// Total pixel area of the atlas (width * height).
    pub total_pixels: u64,
    /// Pack efficiency as a percentage (0.0 - 100.0).
    pub efficiency: f32,
    /// Wasted pixel area (total - used).
    pub wasted_pixels: u64,
}

impl AtlasStats {
    /// Computes statistics from raw values.
    pub fn compute(texture_count: u32, width: u32, height: u32, used_pixels: u64) -> Self {
        let total_pixels = u64::from(width) * u64::from(height);
        let efficiency = if total_pixels > 0 {
            (used_pixels as f64 / total_pixels as f64 * 100.0) as f32
        } else {
            0.0
        };
        Self {
            texture_count,
            width,
            height,
            used_pixels,
            total_pixels,
            efficiency,
            wasted_pixels: total_pixels.saturating_sub(used_pixels),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_full_atlas() {
        let s = AtlasStats::compute(1, 64, 64, 64 * 64);
        assert_eq!(s.total_pixels, 4096);
        assert_eq!(s.wasted_pixels, 0);
        assert!((s.efficiency - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_stats_half_used() {
        let s = AtlasStats::compute(2, 100, 100, 5000);
        assert_eq!(s.total_pixels, 10000);
        assert_eq!(s.wasted_pixels, 5000);
        assert!((s.efficiency - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_stats_empty() {
        let s = AtlasStats::compute(0, 256, 256, 0);
        assert_eq!(s.wasted_pixels, 256 * 256);
        assert!((s.efficiency - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_stats_zero_size() {
        let s = AtlasStats::compute(0, 0, 0, 0);
        assert_eq!(s.total_pixels, 0);
        assert_eq!(s.efficiency, 0.0);
    }
}
