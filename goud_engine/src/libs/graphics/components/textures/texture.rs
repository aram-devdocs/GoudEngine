// src/texture.rs

use crate::types::Texture;
use gl::types::*;
use std::ffi::c_void;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

static MOCK_MODE: AtomicBool = AtomicBool::new(false);

impl Texture {
    /// Loads a texture from a file.
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Texture, String> {
        let img = image::open(file_path.as_ref())
            .map_err(|_| format!("Failed to load texture from {:?}", file_path.as_ref()))?;
        let data = img.flipv().to_rgba8();
        let width = img.width();
        let height = img.height();

        if MOCK_MODE.load(Ordering::Relaxed) {
            return Ok(Texture {
                id: 1,
                width,
                height,
            });
        }

        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            if id == 0 {
                return Err("Failed to generate texture ID".into());
            }
            gl::BindTexture(gl::TEXTURE_2D, id);

            // Set texture parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        Ok(Texture { id, width, height })
    }

    /// Binds the texture to a texture unit.
    pub fn bind(&self, unit: GLenum) {
        if !MOCK_MODE.load(Ordering::Relaxed) {
            unsafe {
                gl::ActiveTexture(unit);
                gl::BindTexture(gl::TEXTURE_2D, self.id);
            }
        }
    }

    /// Returns the width of the texture.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the texture.
    pub fn height(&self) -> u32 {
        self.height
    }

    // pub fn terminate(&self) {
    //     unsafe {
    //         gl::DeleteTextures(1, &self.id);
    //     }
    // }
}

impl Drop for Texture {
    fn drop(&mut self) {
        if !MOCK_MODE.load(Ordering::Relaxed) {
            unsafe {
                gl::DeleteTextures(1, &self.id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::graphics::components::utils::test_helper::init_test_context;

    #[test]
    fn test_texture_creation() {
        // Initialize test context
        let has_context = init_test_context();
        MOCK_MODE.store(!has_context, Ordering::Relaxed);

        // Test creating a texture with a valid image
        let texture =
            Texture::new("src/libs/graphics/components/tiled/_Tiled/Tilesets/Tileset.png");
        assert!(texture.is_ok());
        let texture = texture.unwrap();
        assert_eq!(texture.width(), 384);
        assert_eq!(texture.height(), 256);
    }

    #[test]
    fn test_texture_creation_invalid_path() {
        // Initialize test context
        let has_context = init_test_context();
        MOCK_MODE.store(!has_context, Ordering::Relaxed);

        // Test creating a texture with an invalid path
        let texture = Texture::new("nonexistent.png");
        assert!(texture.is_err());
    }

    #[test]
    fn test_texture_bind() {
        // Initialize test context
        let has_context = init_test_context();
        MOCK_MODE.store(!has_context, Ordering::Relaxed);

        // Test texture binding
        let texture =
            Texture::new("src/libs/graphics/components/tiled/_Tiled/Tilesets/Tileset.png").unwrap();
        texture.bind(gl::TEXTURE0);
        // Note: We can't easily verify the binding state without OpenGL context
        // This test mainly ensures the function doesn't panic
    }

    #[test]
    fn test_texture_dimensions() {
        // Initialize test context
        let has_context = init_test_context();
        MOCK_MODE.store(!has_context, Ordering::Relaxed);

        // Test texture dimension getters
        let texture =
            Texture::new("src/libs/graphics/components/tiled/_Tiled/Tilesets/Tileset.png").unwrap();
        assert_eq!(texture.width(), 384);
        assert_eq!(texture.height(), 256);
    }

    #[test]
    fn test_texture_drop() {
        // Initialize test context
        let has_context = init_test_context();
        MOCK_MODE.store(!has_context, Ordering::Relaxed);

        // Test texture cleanup on drop
        let texture =
            Texture::new("src/libs/graphics/components/tiled/_Tiled/Tilesets/Tileset.png").unwrap();
        let _texture_id = texture.id;
        drop(texture);
        // Note: We can't easily verify the texture was deleted without OpenGL context
        // This test mainly ensures the drop implementation doesn't panic
    }
}
