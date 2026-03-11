//! # SDK Texture API
//!
//! Provides methods on [`GoudGame`] for texture operations
//! including loading image files and destroying GPU texture resources.
//!
//! # Availability
//!
//! This module requires the `native` feature (desktop platform with OpenGL).

use super::GoudGame;
use crate::libs::graphics::backend::TextureOps;

/// Invalid texture handle sentinel value.
const INVALID_TEXTURE: u64 = u64::MAX;

// =============================================================================
// Texture Operations (annotated for FFI generation)
// =============================================================================

// NOTE: FFI wrappers are hand-written in ffi/texture.rs. The `#[goud_api]`
// attribute is omitted here to avoid duplicate `#[no_mangle]` symbol conflicts.
impl GoudGame {
    /// Loads a texture from an image file and returns a packed u64 handle.
    ///
    /// Returns `u64::MAX` on error.
    pub fn load(&mut self, path: &str) -> u64 {
        use crate::libs::graphics::backend::types::{TextureFilter, TextureFormat, TextureWrap};

        let backend = match self.render_backend.as_mut() {
            Some(b) => b,
            None => return INVALID_TEXTURE,
        };

        let img = match image::open(path) {
            Ok(i) => i.to_rgba8(),
            Err(_) => return INVALID_TEXTURE,
        };

        let width = img.width();
        let height = img.height();
        let data = img.into_raw();

        match backend.create_texture(
            width,
            height,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::ClampToEdge,
            &data,
        ) {
            Ok(handle) => ((handle.generation() as u64) << 32) | (handle.index() as u64),
            Err(_) => INVALID_TEXTURE,
        }
    }

    /// Destroys a texture and releases its GPU resources.
    pub fn destroy(&mut self, texture: u64) -> bool {
        use crate::libs::graphics::backend::types::TextureHandle;

        let backend = match self.render_backend.as_mut() {
            Some(b) => b,
            None => return false,
        };

        let index = (texture & 0xFFFF_FFFF) as u32;
        let generation = ((texture >> 32) & 0xFFFF_FFFF) as u32;
        let handle = TextureHandle::new(index, generation);
        backend.destroy_texture(handle)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::GameConfig;

    #[test]
    fn test_texture_load_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert_eq!(game.load("nonexistent.png"), u64::MAX);
    }

    #[test]
    fn test_texture_destroy_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.destroy(0));
    }
}
