use crate::types::{Texture, TextureManager};
use std::{
    collections::HashMap,
    ffi::{c_char, c_uint},
    rc::Rc,
};

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

 

    pub fn create_texture(&mut self, texture_path: *const c_char) -> c_uint {
        let texture_path_str = unsafe { std::ffi::CStr::from_ptr(texture_path).to_str().unwrap() };
        let texture_path = texture_path_str.to_string();
        let texture = Texture::new(texture_path).unwrap();
        let texture_id = self.textures.len() as c_uint;
        self.textures.insert(texture_id, texture);
        texture_id
    }

    pub fn get_texture(&self, texture_id: c_uint) -> Rc<Texture> {
        self.textures.get(&texture_id).unwrap().clone()
    }

}
