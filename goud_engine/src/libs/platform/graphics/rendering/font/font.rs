// src/font.rs

use freetype::Library;
use std::path::Path;
use std::collections::HashMap;
use gl::types::*;
use std::ffi::c_void;

use crate::types::{Character, Font};

impl Font {
    pub fn new<P: AsRef<Path>>(library: &Library, file_path: P, size: u32) -> Result<Font, String> {
        let face = library.new_face(file_path.as_ref(), 0)
            .map_err(|e| format!("Failed to load font: {:?}", e))?;
        face.set_pixel_sizes(0, size)
            .map_err(|e| format!("Failed to set font size: {:?}", e))?;
        
        let mut characters = HashMap::new();

        // Load ASCII characters 32 - 126
        for c in 32u8..127u8 {
            let c = c as char;
            face.load_char(c as usize, freetype::face::LoadFlag::RENDER)
                .map_err(|e| format!("Failed to load character '{}': {:?}", c, e))?;
            let glyph = face.glyph();
            let bitmap = glyph.bitmap();
            
            // Generate texture
            let mut texture_id: GLuint = 0;
            unsafe {
                gl::GenTextures(1, &mut texture_id);
                gl::BindTexture(gl::TEXTURE_2D, texture_id);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RED as i32,
                    bitmap.width(),
                    bitmap.rows(),
                    0,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    bitmap.buffer().as_ptr() as *const c_void,
                );
                
                // Set texture options
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            }
            
            // Store character
            let character = Character {
                texture_id,
                size: (bitmap.width() as u32, bitmap.rows() as u32),
                bearing: (glyph.bitmap_left(), glyph.bitmap_top()),
                advance: (glyph.advance().x >> 6) as u32,
            };
            characters.insert(c, character);
        }
        
        Ok(Font {
            characters,
            size,
        })
    }
}