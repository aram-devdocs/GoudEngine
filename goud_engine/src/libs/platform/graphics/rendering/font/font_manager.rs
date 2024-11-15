// src/font_manager.rs

use freetype::Library;
use std::path::Path;
use std::collections::HashMap;
use crate::types::{Font, FontId};

pub struct FontManager {
    library: Library,
    fonts: HashMap<FontId, Font>,
    next_id: FontId,
}

impl FontManager {
    pub fn new() -> Result<Self, String> {
        let library = Library::init()
            .map_err(|e| format!("Could not initialize FreeType Library: {:?}", e))?;
        Ok(FontManager {
            library,
            fonts: HashMap::new(),
            next_id: 0,
        })
    }
    
    pub fn load_font<P: AsRef<Path>>(&mut self, file_path: P, size: u32) -> Result<FontId, String> {
        let font = Font::new(&self.library, file_path, size)?;
        let font_id = self.next_id;
        self.next_id += 1;
        self.fonts.insert(font_id, font);
        Ok(font_id)
    }
    
    pub fn get_font(&self, font_id: FontId) -> Option<&Font> {
        self.fonts.get(&font_id)
    }
}