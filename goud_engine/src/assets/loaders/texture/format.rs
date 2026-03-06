//! [`TextureFormat`] — image file format enumeration.

use image::ImageFormat;
use std::fmt;

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

    /// Converts to the `image` crate's [`ImageFormat`].
    pub(super) fn to_image_format(self) -> Option<ImageFormat> {
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
