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
            texture_path_to_id_dict: HashMap::new(),
        }
    }

    //    1. We need a func to convert a c_char to c_uint that will return the same values each time. a decode and encode function.
    pub fn encode_texture_path_to_id(&mut self, texture_path: *const c_char) -> c_uint {
        let texture_path_str = unsafe { std::ffi::CStr::from_ptr(texture_path).to_str().unwrap() };
        let texture_path = texture_path_str.to_string();
        let texture_id = self.textures.len() as c_uint;
        self.texture_path_to_id_dict
            .insert(texture_path, texture_id);
        texture_id
    }

    pub fn decode_texture_path_to_id(&self, texture_path: *const c_char) -> c_uint {
        let texture_path_str = unsafe { std::ffi::CStr::from_ptr(texture_path).to_str().unwrap() };
        let texture_path = texture_path_str.to_string();
        *self.texture_path_to_id_dict.get(&texture_path).unwrap()
    }

    // 2. We need to be able to create a texture from a file path, and respond with a texture id so it can be used in the future.
    pub fn create_texture(&mut self, texture_path: *const c_char) -> c_uint {
        let texture_path_str = unsafe { std::ffi::CStr::from_ptr(texture_path).to_str().unwrap() };
        let texture_path = texture_path_str.to_string();
        let texture = Texture::new(texture_path).unwrap();
        let texture_id = self.textures.len() as c_uint;
        self.textures.insert(texture_id, texture);
        texture_id
    }

    // 3. We need to be able to get a texture from a texture id, and provide it for the rendering system.
    pub fn get_texture(&self, texture_id: c_uint) -> Rc<Texture> {
        self.textures.get(&texture_id).unwrap().clone()
    }

    // 4. We need to be able to bind a texture to a texture unit.
}
