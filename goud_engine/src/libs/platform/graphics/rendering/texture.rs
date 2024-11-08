// src/texture.rs

use gl::types::*;
use image::GenericImageView;
use std::ffi::c_void;
use std::path::Path;
use std::rc::Rc;

use crate::types::Texture;

impl Texture {
    /// Loads a texture from a file.
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Rc<Texture>, String> {
        let img = image::open(file_path.as_ref())
            .map_err(|_| format!("Failed to load texture from {:?}", file_path.as_ref()))?;
        let data = img.flipv().to_rgba8();
        let width = img.width();
        let height = img.height();

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

        Ok(Rc::new(Texture { id, width, height }))
    }

    /// Binds the texture to a texture unit.
    pub fn bind(&self, unit: GLenum) {
        unsafe {
            gl::ActiveTexture(unit);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
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
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
