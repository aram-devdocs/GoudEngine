//! [`TextureAsset`] — decoded image data ready for GPU upload.

use crate::assets::{Asset, AssetType};

use super::format::TextureFormat;

/// A loaded texture asset containing image data.
///
/// `TextureAsset` stores decoded image data in memory. It does not contain
/// GPU resources - those should be created separately from this data.
///
/// # Fields
///
/// - `data`: Raw pixel data in RGBA8 format (4 bytes per pixel)
/// - `width`: Image width in pixels
/// - `height`: Image height in pixels
/// - `format`: The original image format (PNG, JPG, etc.)
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, loaders::TextureAsset};
///
/// let texture = TextureAsset {
///     data: vec![255; 64 * 64 * 4], // 64x64 white texture
///     width: 64,
///     height: 64,
///     format: goud_engine::assets::loaders::TextureFormat::Png,
/// };
///
/// assert_eq!(texture.pixel_count(), 64 * 64);
/// assert_eq!(texture.bytes_per_pixel(), 4);
/// ```
#[derive(Debug, Clone)]
pub struct TextureAsset {
    /// Raw pixel data in RGBA8 format (4 bytes per pixel).
    pub data: Vec<u8>,

    /// Width of the texture in pixels.
    pub width: u32,

    /// Height of the texture in pixels.
    pub height: u32,

    /// The original image format this texture was loaded from.
    pub format: TextureFormat,
}

impl TextureAsset {
    /// Creates a new texture asset from raw RGBA8 data.
    ///
    /// # Arguments
    ///
    /// - `data`: Raw pixel data in RGBA8 format (must be width × height × 4 bytes)
    /// - `width`: Image width in pixels
    /// - `height`: Image height in pixels
    /// - `format`: The image format this data represents
    ///
    /// # Panics
    ///
    /// Panics if data length doesn't match width × height × 4.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::loaders::{TextureAsset, TextureFormat};
    ///
    /// let data = vec![255; 4 * 4 * 4]; // 4x4 white texture
    /// let texture = TextureAsset::new(data, 4, 4, TextureFormat::Png);
    /// assert_eq!(texture.pixel_count(), 16);
    /// ```
    pub fn new(data: Vec<u8>, width: u32, height: u32, format: TextureFormat) -> Self {
        let expected_len = (width * height * 4) as usize;
        if data.len() != expected_len {
            panic!(
                "Texture data length mismatch: expected {} bytes for {}x{} RGBA8, got {}",
                expected_len,
                width,
                height,
                data.len()
            );
        }
        Self {
            data,
            width,
            height,
            format,
        }
    }

    /// Returns the total number of pixels in the texture.
    #[inline]
    pub fn pixel_count(&self) -> u32 {
        self.width * self.height
    }

    /// Returns the number of bytes per pixel (always 4 for RGBA8).
    #[inline]
    pub const fn bytes_per_pixel(&self) -> u32 {
        4
    }

    /// Returns the total size of the texture data in bytes.
    #[inline]
    pub fn size_bytes(&self) -> usize {
        self.data.len()
    }

    /// Returns the aspect ratio (width / height) of the texture.
    #[inline]
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Returns true if the texture dimensions are powers of two.
    ///
    /// Power-of-two textures are required by some older GPUs and can
    /// be more efficient for certain operations.
    pub fn is_power_of_two(&self) -> bool {
        self.width.is_power_of_two() && self.height.is_power_of_two()
    }

    /// Returns a slice of the pixel data for a specific pixel.
    ///
    /// Returns `None` if the coordinates are out of bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::loaders::{TextureAsset, TextureFormat};
    ///
    /// let data = vec![255, 0, 0, 255, 0, 255, 0, 255]; // 2 pixels: red, green
    /// let texture = TextureAsset::new(data, 2, 1, TextureFormat::Png);
    ///
    /// let pixel = texture.get_pixel(0, 0).unwrap();
    /// assert_eq!(pixel, &[255, 0, 0, 255]); // Red
    ///
    /// let pixel = texture.get_pixel(1, 0).unwrap();
    /// assert_eq!(pixel, &[0, 255, 0, 255]); // Green
    /// ```
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<&[u8]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = ((y * self.width + x) * 4) as usize;
        Some(&self.data[index..index + 4])
    }
}

impl Asset for TextureAsset {
    fn asset_type_name() -> &'static str {
        "Texture"
    }

    fn asset_type() -> AssetType {
        AssetType::Texture
    }

    fn extensions() -> &'static [&'static str] {
        &[
            "png", "jpg", "jpeg", "bmp", "tga", "gif", "webp", "ico", "tiff",
        ]
    }
}
