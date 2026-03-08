//! Texture operations sub-trait for `RenderBackend`.

use crate::libs::error::{GoudError, GoudResult};
use crate::libs::graphics::backend::types::{
    TextureFilter, TextureFormat, TextureHandle, TextureWrap,
};

/// GPU texture management operations.
///
/// Handles creation, updating, binding, and destruction of textures.
pub trait TextureOps {
    /// Creates a GPU texture with the specified dimensions, format, and initial data.
    ///
    /// # Arguments
    /// * `width` - Texture width in pixels (must be > 0)
    /// * `height` - Texture height in pixels (must be > 0)
    /// * `format` - Pixel format (RGBA8, RGB8, etc.)
    /// * `filter` - Minification/magnification filtering mode
    /// * `wrap` - Texture coordinate wrapping mode
    /// * `data` - Initial pixel data (may be empty for render targets)
    ///
    /// # Returns
    /// A handle to the created texture, or an error if creation failed.
    fn create_texture(
        &mut self,
        width: u32,
        height: u32,
        format: TextureFormat,
        filter: TextureFilter,
        wrap: TextureWrap,
        data: &[u8],
    ) -> GoudResult<TextureHandle>;

    /// Updates a region of an existing texture with new pixel data.
    ///
    /// # Arguments
    /// * `handle` - Handle to the texture to update
    /// * `x` - X offset in pixels (0 = left edge)
    /// * `y` - Y offset in pixels (0 = bottom edge)
    /// * `width` - Width of the update region in pixels
    /// * `height` - Height of the update region in pixels
    /// * `data` - New pixel data for the region
    fn update_texture(
        &mut self,
        handle: TextureHandle,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> GoudResult<()>;

    /// Destroys a texture and frees GPU memory.
    ///
    /// # Returns
    /// `true` if the texture was destroyed, `false` if the handle was invalid.
    fn destroy_texture(&mut self, handle: TextureHandle) -> bool;

    /// Checks if a texture handle is valid and refers to an existing texture.
    fn is_texture_valid(&self, handle: TextureHandle) -> bool;

    /// Returns the dimensions (width, height) of a texture, or `None` if invalid.
    fn texture_size(&self, handle: TextureHandle) -> Option<(u32, u32)>;

    /// Binds a texture to a texture unit for use in subsequent draw calls.
    ///
    /// # Arguments
    /// * `handle` - Handle to the texture to bind
    /// * `unit` - Texture unit index (0-based, typically 0-15 supported)
    fn bind_texture(&mut self, handle: TextureHandle, unit: u32) -> GoudResult<()>;

    /// Unbinds any texture from the specified texture unit.
    fn unbind_texture(&mut self, unit: u32);

    /// Creates a GPU texture from block-compressed data (BC1/BC3/BC5/BC7).
    ///
    /// # Arguments
    /// * `width` - Texture width in pixels (must be > 0)
    /// * `height` - Texture height in pixels (must be > 0)
    /// * `format` - Compressed pixel format (BC1, BC3, BC5, or BC7)
    /// * `data` - Block-compressed texture data
    /// * `mip_levels` - Number of mipmap levels in the data
    ///
    /// # Returns
    /// A handle to the created texture, or an error if the backend
    /// does not support compressed textures.
    ///
    /// # Default
    /// Returns `GoudError::BackendNotSupported` -- backends that support
    /// compressed textures should override this method.
    fn create_compressed_texture(
        &mut self,
        width: u32,
        height: u32,
        format: TextureFormat,
        data: &[u8],
        mip_levels: u32,
    ) -> GoudResult<TextureHandle> {
        let _ = (width, height, format, data, mip_levels);
        Err(GoudError::BackendNotSupported(
            "Compressed textures not supported by this backend".to_string(),
        ))
    }
}
