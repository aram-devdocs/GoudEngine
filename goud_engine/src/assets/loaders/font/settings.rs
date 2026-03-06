//! [`FontSettings`] for controlling font load behavior.

/// Font loading settings.
#[derive(Clone, Debug)]
pub struct FontSettings {
    /// Default font size in pixels for rasterization.
    pub default_size_px: f32,
    /// Index of the font within a font collection (TTC/OTC).
    pub collection_index: u32,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            default_size_px: 16.0,
            collection_index: 0,
        }
    }
}
