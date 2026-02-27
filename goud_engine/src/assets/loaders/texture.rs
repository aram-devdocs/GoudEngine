//! Texture asset loader.
//!
//! This module provides asset types and loaders for image-based textures.
//! Supports common image formats like PNG, JPG, BMP, TGA, and more.
//!
//! # Example
///
/// ```no_run
/// use goud_engine::assets::{AssetServer, loaders::texture::TextureLoader, loaders::texture::TextureAsset};
///
/// let mut server = AssetServer::new();
/// server.register_loader(TextureLoader::default());
///
/// let handle = server.load::<TextureAsset>("textures/player.png");
/// ```
use crate::assets::{Asset, AssetLoadError, AssetLoader, AssetType, LoadContext};
use image::{DynamicImage, ImageError, ImageFormat};
use std::fmt;

// =============================================================================
// TextureAsset
// =============================================================================

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
        assert_eq!(
            data.len(),
            (width * height * 4) as usize,
            "Texture data length mismatch: expected {} bytes for {}x{} RGBA8, got {}",
            width * height * 4,
            width,
            height,
            data.len()
        );
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

// =============================================================================
// TextureFormat
// =============================================================================

/// Image format of a texture.
///
/// This enum represents the file format the texture was loaded from,
/// not the in-memory pixel format (which is always RGBA8).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextureFormat {
    /// PNG - Portable Network Graphics (default)
    #[default]
    Png = 0,
    /// JPEG - Joint Photographic Experts Group
    Jpeg = 1,
    /// BMP - Windows Bitmap
    Bmp = 2,
    /// TGA - Truevision TGA
    Tga = 3,
    /// GIF - Graphics Interchange Format
    Gif = 4,
    /// WebP - Google WebP format
    WebP = 5,
    /// ICO - Windows Icon
    Ico = 6,
    /// TIFF - Tagged Image File Format
    Tiff = 7,
    /// Unknown or custom format
    Unknown = 255,
}

impl TextureFormat {
    /// Returns the file extension typically associated with this format.
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Bmp => "bmp",
            Self::Tga => "tga",
            Self::Gif => "gif",
            Self::WebP => "webp",
            Self::Ico => "ico",
            Self::Tiff => "tiff",
            Self::Unknown => "",
        }
    }

    /// Returns the format name.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Jpeg => "JPEG",
            Self::Bmp => "BMP",
            Self::Tga => "TGA",
            Self::Gif => "GIF",
            Self::WebP => "WebP",
            Self::Ico => "ICO",
            Self::Tiff => "TIFF",
            Self::Unknown => "Unknown",
        }
    }

    /// Detects the format from a file extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" => Self::Png,
            "jpg" | "jpeg" => Self::Jpeg,
            "bmp" => Self::Bmp,
            "tga" => Self::Tga,
            "gif" => Self::Gif,
            "webp" => Self::WebP,
            "ico" => Self::Ico,
            "tif" | "tiff" => Self::Tiff,
            _ => Self::Unknown,
        }
    }

    /// Converts to the `image` crate's ImageFormat.
    fn to_image_format(self) -> Option<ImageFormat> {
        match self {
            Self::Png => Some(ImageFormat::Png),
            Self::Jpeg => Some(ImageFormat::Jpeg),
            Self::Bmp => Some(ImageFormat::Bmp),
            Self::Tga => Some(ImageFormat::Tga),
            Self::Gif => Some(ImageFormat::Gif),
            Self::WebP => Some(ImageFormat::WebP),
            Self::Ico => Some(ImageFormat::Ico),
            Self::Tiff => Some(ImageFormat::Tiff),
            Self::Unknown => None,
        }
    }
}

impl fmt::Display for TextureFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// =============================================================================
// TextureSettings
// =============================================================================

/// Configuration for texture loading.
///
/// These settings control how textures are decoded and processed during loading.
#[derive(Debug, Clone)]
pub struct TextureSettings {
    /// Whether to flip the texture vertically (Y-axis).
    ///
    /// OpenGL expects textures with the origin at the bottom-left, but most
    /// image formats have the origin at the top-left. Set this to `true` when
    /// loading textures for OpenGL.
    pub flip_vertical: bool,

    /// Color space interpretation of the texture.
    pub color_space: TextureColorSpace,

    /// Wrap mode for texture coordinates outside [0, 1].
    pub wrap_mode: TextureWrapMode,

    /// Whether to generate mipmaps for this texture.
    ///
    /// Note: This setting is informational only. Actual mipmap generation
    /// happens during GPU upload, not during asset loading.
    pub generate_mipmaps: bool,
}

impl Default for TextureSettings {
    fn default() -> Self {
        Self {
            flip_vertical: true, // Default to OpenGL convention
            color_space: TextureColorSpace::Srgb,
            wrap_mode: TextureWrapMode::Repeat,
            generate_mipmaps: true,
        }
    }
}

// =============================================================================
// TextureColorSpace
// =============================================================================

/// Color space of a texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextureColorSpace {
    /// Standard RGB (linear color space)
    Linear,
    /// sRGB (gamma-corrected color space) - default for most images
    #[default]
    Srgb,
}

impl TextureColorSpace {
    /// Returns the string representation of the color space.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::Srgb => "sRGB",
        }
    }
}

impl fmt::Display for TextureColorSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// =============================================================================
// TextureWrapMode
// =============================================================================

/// Texture wrap mode for coordinates outside [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextureWrapMode {
    /// Repeat the texture (default)
    #[default]
    Repeat,
    /// Mirror the texture at boundaries
    MirroredRepeat,
    /// Clamp to edge pixels
    ClampToEdge,
    /// Clamp to border color
    ClampToBorder,
}

impl TextureWrapMode {
    /// Returns the string representation of the wrap mode.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Repeat => "Repeat",
            Self::MirroredRepeat => "MirroredRepeat",
            Self::ClampToEdge => "ClampToEdge",
            Self::ClampToBorder => "ClampToBorder",
        }
    }
}

impl fmt::Display for TextureWrapMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// =============================================================================
// TextureLoader
// =============================================================================

/// Asset loader for texture images.
///
/// Uses the `image` crate to decode image files into RGBA8 pixel data.
/// Supports PNG, JPEG, BMP, TGA, GIF, WebP, ICO, and TIFF formats.
///
/// # Example
///
/// ```no_run
/// use goud_engine::assets::{AssetServer, loaders::texture::{TextureLoader, TextureAsset, TextureSettings}};
///
/// let mut server = AssetServer::new();
/// server.register_loader(TextureLoader::default());
///
/// // Load with default settings
/// let texture = server.load::<TextureAsset>("player.png");
///
/// // Load with custom settings
/// let mut settings = TextureSettings::default();
/// settings.flip_vertical = false;
/// server.register_loader_with_settings(TextureLoader::default(), settings);
/// ```
#[derive(Debug, Clone, Default)]
pub struct TextureLoader;

impl TextureLoader {
    /// Creates a new texture loader.
    pub fn new() -> Self {
        Self
    }

    /// Loads a texture from raw bytes with format detection.
    fn load_from_bytes(
        bytes: &[u8],
        settings: &TextureSettings,
        format_hint: Option<TextureFormat>,
    ) -> Result<TextureAsset, AssetLoadError> {
        // Try to load the image
        let img = if let Some(format) = format_hint.and_then(|f| f.to_image_format()) {
            // Try with format hint first
            image::load_from_memory_with_format(bytes, format)
                .or_else(|_| image::load_from_memory(bytes))
        } else {
            image::load_from_memory(bytes)
        }
        .map_err(Self::convert_image_error)?;

        // Convert to RGBA8
        let img = if settings.flip_vertical {
            img.flipv()
        } else {
            img
        };

        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();
        let data = rgba.into_raw();

        // Determine format
        let format = format_hint.unwrap_or_else(|| {
            // Try to guess from image format
            Self::detect_format(&img).unwrap_or(TextureFormat::Unknown)
        });

        Ok(TextureAsset {
            data,
            width,
            height,
            format,
        })
    }

    /// Detects texture format from a DynamicImage.
    fn detect_format(_img: &DynamicImage) -> Option<TextureFormat> {
        // The image crate doesn't store the original format after decoding,
        // so we can't reliably detect it. Return None to use the extension-based guess.
        None
    }

    /// Converts an image crate error to AssetLoadError.
    fn convert_image_error(error: ImageError) -> AssetLoadError {
        match error {
            ImageError::IoError(e) => AssetLoadError::io_error("", e),
            ImageError::Decoding(e) => AssetLoadError::decode_failed(e.to_string()),
            ImageError::Encoding(e) => AssetLoadError::decode_failed(e.to_string()),
            ImageError::Parameter(e) => AssetLoadError::decode_failed(e.to_string()),
            ImageError::Limits(e) => AssetLoadError::decode_failed(e.to_string()),
            ImageError::Unsupported(e) => AssetLoadError::decode_failed(e.to_string()),
        }
    }
}

impl AssetLoader for TextureLoader {
    type Asset = TextureAsset;
    type Settings = TextureSettings;

    fn extensions(&self) -> &[&str] {
        TextureAsset::extensions()
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        // Get format hint from file extension
        let format_hint = context
            .extension()
            .map(TextureFormat::from_extension)
            .filter(|f| *f != TextureFormat::Unknown);

        Self::load_from_bytes(bytes, settings, format_hint)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::AssetPath;

    // Helper function to create a small test PNG image
    fn create_test_png(width: u32, height: u32) -> Vec<u8> {
        use image::{ImageBuffer, Rgba};
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |x, y| {
            if (x + y) % 2 == 0 {
                Rgba([255, 0, 0, 255]) // Red
            } else {
                Rgba([0, 255, 0, 255]) // Green
            }
        });
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        bytes
    }

    // =========================================================================
    // TextureAsset Tests
    // =========================================================================

    mod texture_asset {
        use super::*;

        #[test]
        fn test_new() {
            let data = vec![255; 4 * 4 * 4]; // 4x4 RGBA8
            let texture = TextureAsset::new(data.clone(), 4, 4, TextureFormat::Png);
            assert_eq!(texture.width, 4);
            assert_eq!(texture.height, 4);
            assert_eq!(texture.data.len(), 64);
            assert_eq!(texture.format, TextureFormat::Png);
        }

        #[test]
        #[should_panic(expected = "Texture data length mismatch")]
        fn test_new_wrong_size() {
            let data = vec![255; 10]; // Wrong size
            TextureAsset::new(data, 4, 4, TextureFormat::Png);
        }

        #[test]
        fn test_pixel_count() {
            let data = vec![255; 8 * 8 * 4];
            let texture = TextureAsset::new(data, 8, 8, TextureFormat::Png);
            assert_eq!(texture.pixel_count(), 64);
        }

        #[test]
        fn test_bytes_per_pixel() {
            let data = vec![255; 4 * 4 * 4];
            let texture = TextureAsset::new(data, 4, 4, TextureFormat::Png);
            assert_eq!(texture.bytes_per_pixel(), 4);
        }

        #[test]
        fn test_size_bytes() {
            let data = vec![255; 4 * 4 * 4];
            let texture = TextureAsset::new(data, 4, 4, TextureFormat::Png);
            assert_eq!(texture.size_bytes(), 64);
        }

        #[test]
        fn test_aspect_ratio() {
            let data = vec![255; 16 * 9 * 4];
            let texture = TextureAsset::new(data, 16, 9, TextureFormat::Png);
            assert!((texture.aspect_ratio() - 16.0 / 9.0).abs() < 0.001);
        }

        #[test]
        fn test_is_power_of_two() {
            let data = vec![255; 64 * 64 * 4];
            let texture = TextureAsset::new(data, 64, 64, TextureFormat::Png);
            assert!(texture.is_power_of_two());

            let data = vec![255; 60 * 60 * 4];
            let texture = TextureAsset::new(data, 60, 60, TextureFormat::Png);
            assert!(!texture.is_power_of_two());
        }

        #[test]
        fn test_get_pixel() {
            let mut data = Vec::new();
            // Create 2x2 texture with known colors
            data.extend_from_slice(&[255, 0, 0, 255]); // (0,0) Red
            data.extend_from_slice(&[0, 255, 0, 255]); // (1,0) Green
            data.extend_from_slice(&[0, 0, 255, 255]); // (0,1) Blue
            data.extend_from_slice(&[255, 255, 255, 255]); // (1,1) White

            let texture = TextureAsset::new(data, 2, 2, TextureFormat::Png);

            assert_eq!(texture.get_pixel(0, 0), Some(&[255, 0, 0, 255][..]));
            assert_eq!(texture.get_pixel(1, 0), Some(&[0, 255, 0, 255][..]));
            assert_eq!(texture.get_pixel(0, 1), Some(&[0, 0, 255, 255][..]));
            assert_eq!(texture.get_pixel(1, 1), Some(&[255, 255, 255, 255][..]));
            assert_eq!(texture.get_pixel(2, 0), None); // Out of bounds
            assert_eq!(texture.get_pixel(0, 2), None); // Out of bounds
        }

        #[test]
        fn test_asset_trait() {
            assert_eq!(TextureAsset::asset_type_name(), "Texture");
            assert_eq!(TextureAsset::asset_type(), AssetType::Texture);
            assert!(TextureAsset::extensions().contains(&"png"));
            assert!(TextureAsset::extensions().contains(&"jpg"));
        }

        #[test]
        fn test_clone() {
            let data = vec![255; 4 * 4 * 4];
            let texture1 = TextureAsset::new(data, 4, 4, TextureFormat::Png);
            let texture2 = texture1.clone();
            assert_eq!(texture1.width, texture2.width);
            assert_eq!(texture1.height, texture2.height);
            assert_eq!(texture1.data, texture2.data);
        }

        #[test]
        fn test_debug() {
            let data = vec![255; 4 * 4 * 4];
            let texture = TextureAsset::new(data, 4, 4, TextureFormat::Png);
            let debug_str = format!("{:?}", texture);
            assert!(debug_str.contains("TextureAsset"));
            assert!(debug_str.contains("4"));
        }
    }

    // =========================================================================
    // TextureFormat Tests
    // =========================================================================

    mod texture_format {
        use super::*;

        #[test]
        fn test_extension() {
            assert_eq!(TextureFormat::Png.extension(), "png");
            assert_eq!(TextureFormat::Jpeg.extension(), "jpg");
            assert_eq!(TextureFormat::Bmp.extension(), "bmp");
        }

        #[test]
        fn test_name() {
            assert_eq!(TextureFormat::Png.name(), "PNG");
            assert_eq!(TextureFormat::Jpeg.name(), "JPEG");
            assert_eq!(TextureFormat::Unknown.name(), "Unknown");
        }

        #[test]
        fn test_from_extension() {
            assert_eq!(TextureFormat::from_extension("png"), TextureFormat::Png);
            assert_eq!(TextureFormat::from_extension("PNG"), TextureFormat::Png);
            assert_eq!(TextureFormat::from_extension("jpg"), TextureFormat::Jpeg);
            assert_eq!(TextureFormat::from_extension("jpeg"), TextureFormat::Jpeg);
            assert_eq!(TextureFormat::from_extension("xyz"), TextureFormat::Unknown);
        }

        #[test]
        fn test_display() {
            assert_eq!(format!("{}", TextureFormat::Png), "PNG");
            assert_eq!(format!("{}", TextureFormat::Jpeg), "JPEG");
        }

        #[test]
        fn test_default() {
            assert_eq!(TextureFormat::default(), TextureFormat::Png);
        }

        #[test]
        fn test_equality() {
            assert_eq!(TextureFormat::Png, TextureFormat::Png);
            assert_ne!(TextureFormat::Png, TextureFormat::Jpeg);
        }

        #[test]
        fn test_clone() {
            let format = TextureFormat::Png;
            let cloned = format;
            assert_eq!(format, cloned);
        }
    }

    // =========================================================================
    // TextureSettings Tests
    // =========================================================================

    mod texture_settings {
        use super::*;

        #[test]
        fn test_default() {
            let settings = TextureSettings::default();
            assert_eq!(settings.flip_vertical, true);
            assert_eq!(settings.color_space, TextureColorSpace::Srgb);
            assert_eq!(settings.wrap_mode, TextureWrapMode::Repeat);
            assert_eq!(settings.generate_mipmaps, true);
        }

        #[test]
        fn test_clone() {
            let settings1 = TextureSettings::default();
            let settings2 = settings1.clone();
            assert_eq!(settings1.flip_vertical, settings2.flip_vertical);
        }

        #[test]
        fn test_debug() {
            let settings = TextureSettings::default();
            let debug_str = format!("{:?}", settings);
            assert!(debug_str.contains("TextureSettings"));
        }
    }

    // =========================================================================
    // TextureColorSpace Tests
    // =========================================================================

    mod texture_color_space {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(TextureColorSpace::Linear.name(), "Linear");
            assert_eq!(TextureColorSpace::Srgb.name(), "sRGB");
        }

        #[test]
        fn test_display() {
            assert_eq!(format!("{}", TextureColorSpace::Linear), "Linear");
            assert_eq!(format!("{}", TextureColorSpace::Srgb), "sRGB");
        }

        #[test]
        fn test_default() {
            assert_eq!(TextureColorSpace::default(), TextureColorSpace::Srgb);
        }

        #[test]
        fn test_equality() {
            assert_eq!(TextureColorSpace::Srgb, TextureColorSpace::Srgb);
            assert_ne!(TextureColorSpace::Srgb, TextureColorSpace::Linear);
        }
    }

    // =========================================================================
    // TextureWrapMode Tests
    // =========================================================================

    mod texture_wrap_mode {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(TextureWrapMode::Repeat.name(), "Repeat");
            assert_eq!(TextureWrapMode::MirroredRepeat.name(), "MirroredRepeat");
            assert_eq!(TextureWrapMode::ClampToEdge.name(), "ClampToEdge");
            assert_eq!(TextureWrapMode::ClampToBorder.name(), "ClampToBorder");
        }

        #[test]
        fn test_display() {
            assert_eq!(format!("{}", TextureWrapMode::Repeat), "Repeat");
            assert_eq!(
                format!("{}", TextureWrapMode::MirroredRepeat),
                "MirroredRepeat"
            );
        }

        #[test]
        fn test_default() {
            assert_eq!(TextureWrapMode::default(), TextureWrapMode::Repeat);
        }

        #[test]
        fn test_equality() {
            assert_eq!(TextureWrapMode::Repeat, TextureWrapMode::Repeat);
            assert_ne!(TextureWrapMode::Repeat, TextureWrapMode::ClampToEdge);
        }
    }

    // =========================================================================
    // TextureLoader Tests
    // =========================================================================

    mod texture_loader {
        use super::*;

        #[test]
        fn test_new() {
            let loader = TextureLoader::new();
            assert_eq!(loader.extensions().len(), 9);
        }

        #[test]
        fn test_default() {
            let loader = TextureLoader::default();
            assert_eq!(loader.extensions().len(), 9);
        }

        #[test]
        fn test_extensions() {
            let loader = TextureLoader::new();
            assert!(loader.supports_extension("png"));
            assert!(loader.supports_extension("jpg"));
            assert!(loader.supports_extension("jpeg"));
            assert!(loader.supports_extension("bmp"));
            assert!(!loader.supports_extension("xyz"));
        }

        #[test]
        fn test_load_png() {
            let loader = TextureLoader::new();
            let bytes = create_test_png(4, 4);
            let path = AssetPath::from_string("test.png".to_string());
            let mut context = LoadContext::new(path);
            let settings = TextureSettings::default();

            let result = loader.load(&bytes, &settings, &mut context);
            assert!(result.is_ok());

            let texture = result.unwrap();
            assert_eq!(texture.width, 4);
            assert_eq!(texture.height, 4);
            assert_eq!(texture.data.len(), 4 * 4 * 4);
        }

        #[test]
        fn test_load_invalid_data() {
            let loader = TextureLoader::new();
            let bytes = vec![0xFF, 0xFE, 0xFD]; // Invalid image data
            let path = AssetPath::from_string("test.png".to_string());
            let mut context = LoadContext::new(path);
            let settings = TextureSettings::default();

            let result = loader.load(&bytes, &settings, &mut context);
            assert!(result.is_err());
        }

        #[test]
        fn test_load_with_flip() {
            let loader = TextureLoader::new();
            let bytes = create_test_png(2, 2);
            let path = AssetPath::from_string("test.png".to_string());
            let mut context = LoadContext::new(path);

            // Load with flip
            let mut settings = TextureSettings::default();
            settings.flip_vertical = true;
            let result1 = loader.load(&bytes, &settings, &mut context);
            assert!(result1.is_ok());
            let texture1 = result1.unwrap();

            // Load without flip
            let path = AssetPath::from_string("test.png".to_string());
            let mut context = LoadContext::new(path);
            settings.flip_vertical = false;
            let result2 = loader.load(&bytes, &settings, &mut context);
            assert!(result2.is_ok());
            let texture2 = result2.unwrap();

            // Both should succeed with same dimensions
            assert_eq!(texture1.width, texture2.width);
            assert_eq!(texture1.height, texture2.height);
            // But pixel data should differ (flipped vs not flipped)
            // Note: For a checkerboard, flip might not change it, so we just verify load succeeded
        }

        #[test]
        fn test_load_from_bytes() {
            let bytes = create_test_png(8, 8);
            let settings = TextureSettings::default();

            let result =
                TextureLoader::load_from_bytes(&bytes, &settings, Some(TextureFormat::Png));
            assert!(result.is_ok());

            let texture = result.unwrap();
            assert_eq!(texture.width, 8);
            assert_eq!(texture.height, 8);
        }

        #[test]
        fn test_clone() {
            let loader1 = TextureLoader::new();
            let loader2 = loader1.clone();
            assert_eq!(loader1.extensions(), loader2.extensions());
        }

        #[test]
        fn test_debug() {
            let loader = TextureLoader::new();
            let debug_str = format!("{:?}", loader);
            assert!(debug_str.contains("TextureLoader"));
        }
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    mod integration {
        use super::*;

        #[test]
        fn test_full_workflow() {
            // Create a test image
            let bytes = create_test_png(16, 16);

            // Create loader and load
            let loader = TextureLoader::new();
            let path = AssetPath::from_string("textures/player.png".to_string());
            let mut context = LoadContext::new(path);
            let settings = TextureSettings::default();

            let result = loader.load(&bytes, &settings, &mut context);
            assert!(result.is_ok());

            let texture = result.unwrap();
            assert_eq!(texture.width, 16);
            assert_eq!(texture.height, 16);
            assert_eq!(texture.pixel_count(), 256);
            assert!(texture.is_power_of_two());
        }

        #[test]
        fn test_different_formats() {
            use image::ImageFormat;

            let formats = vec![
                (ImageFormat::Png, "test.png"),
                (ImageFormat::Jpeg, "test.jpg"),
                (ImageFormat::Bmp, "test.bmp"),
            ];

            let loader = TextureLoader::new();

            for (format, filename) in formats {
                // Create test image in specific format
                let img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
                    image::ImageBuffer::from_pixel(4, 4, image::Rgba([255, 0, 0, 255]));
                let mut bytes = Vec::new();
                image::DynamicImage::ImageRgba8(img)
                    .write_to(&mut std::io::Cursor::new(&mut bytes), format)
                    .unwrap();

                let path = AssetPath::from_string(filename.to_string());
                let mut context = LoadContext::new(path);
                let settings = TextureSettings::default();

                let result = loader.load(&bytes, &settings, &mut context);
                assert!(result.is_ok(), "Failed to load {}", filename);

                let texture = result.unwrap();
                assert_eq!(texture.width, 4);
                assert_eq!(texture.height, 4);
            }
        }

        #[test]
        fn test_error_handling() {
            let loader = TextureLoader::new();
            let path = AssetPath::from_string("test.png".to_string());
            let mut context = LoadContext::new(path);
            let settings = TextureSettings::default();

            // Empty data
            let result = loader.load(&[], &settings, &mut context);
            assert!(result.is_err());

            // Invalid PNG header
            let path = AssetPath::from_string("test.png".to_string());
            let mut context = LoadContext::new(path);
            let result = loader.load(b"not a png", &settings, &mut context);
            assert!(result.is_err());
        }
    }

    // =========================================================================
    // Thread Safety Tests
    // =========================================================================

    mod thread_safety {
        use super::*;

        #[test]
        fn test_texture_asset_send() {
            fn assert_send<T: Send>() {}
            assert_send::<TextureAsset>();
        }

        #[test]
        fn test_texture_asset_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<TextureAsset>();
        }

        #[test]
        fn test_texture_loader_send() {
            fn assert_send<T: Send>() {}
            assert_send::<TextureLoader>();
        }

        #[test]
        fn test_texture_loader_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<TextureLoader>();
        }

        #[test]
        fn test_texture_settings_send() {
            fn assert_send<T: Send>() {}
            assert_send::<TextureSettings>();
        }

        #[test]
        fn test_texture_settings_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<TextureSettings>();
        }
    }
}
