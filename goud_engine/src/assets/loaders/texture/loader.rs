//! [`TextureLoader`] — decodes image bytes into [`TextureAsset`].

use image::{DynamicImage, ImageError};

use crate::assets::{Asset, AssetLoadError, AssetLoader, LoadContext};

use super::{asset::TextureAsset, format::TextureFormat, settings::TextureSettings};

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
    pub(super) fn load_from_bytes(
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

        // Optionally flip vertically
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
        let format = format_hint
            .unwrap_or_else(|| Self::detect_format(&img).unwrap_or(TextureFormat::Unknown));

        Ok(TextureAsset {
            data,
            width,
            height,
            format,
        })
    }

    /// Detects texture format from a [`DynamicImage`].
    ///
    /// The image crate does not store the original format after decoding,
    /// so this always returns `None`. The extension-based guess is used instead.
    fn detect_format(_img: &DynamicImage) -> Option<TextureFormat> {
        None
    }

    /// Converts an image crate error to [`AssetLoadError`].
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
