use crate::types::{Texture, TextureManager};
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::{c_char, c_uint};
use std::rc::Rc;

impl TextureManager {
    pub fn new() -> Self {
        TextureManager {
            textures: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn create_texture(&mut self, texture_path: *const c_char) -> c_uint {
        let texture_path_str = unsafe { CStr::from_ptr(texture_path).to_str().unwrap() };
        let texture = Texture::new(texture_path_str.to_string()).unwrap();
        let id = self.next_id;
        self.textures.insert(id, Rc::new(texture));
        self.next_id += 1;
        id
    }

    pub fn get_texture(&self, texture_id: c_uint) -> Rc<Texture> {
        self.textures
            .get(&texture_id)
            .expect("Texture not found")
            .clone()
    }

    // pub fn get_first_texture(&self) -> Option<Rc<Texture>> {
    //     self.textures.values().next().cloned()
    // }
}
