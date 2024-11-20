// src/tiled_manager.rs

use std::path::Path;
use std::rc::Rc;
use std::{collections::HashMap, sync::Arc};
use tiled::{Loader, Map, Tileset};

use crate::types::{Texture, TextureManager};

pub struct TiledManager {
    loader: Loader,
    maps: HashMap<String, Rc<Map>>,
    texture_manager: Rc<TextureManager>,
}

impl TiledManager {
    pub fn new(texture_manager: Rc<TextureManager>) -> Self {
        Self {
            loader: Loader::new(),
            maps: HashMap::new(),
            texture_manager,
        }
    }

    pub fn load_map(&mut self, map_name: &str, file_path: &str) -> Result<Rc<Map>, String> {
        let map = self
            .loader
            .load_tmx_map(Path::new(file_path))
            .map_err(|e| format!("Failed to load map: {}", e))?;

        // Load associated tilesets
        for tileset in map.tilesets() {
            if let Some(image) = &tileset.image {
                let texture_path = image.source.to_string_lossy();
                let texture_id = texture_path
                    .parse::<u32>()
                    .map_err(|e| format!("Failed to parse texture ID: {}", e))?;
                self.texture_manager.get_texture(texture_id);
            }
        }

        let map_rc = Rc::new(map);
        self.maps.insert(map_name.to_string(), Rc::clone(&map_rc));
        Ok(map_rc)
    }

    pub fn get_map(&self, map_name: &str) -> Option<Rc<Map>> {
        self.maps.get(map_name).cloned()
    }

    pub fn get_tileset(&self, map_name: &str, tileset_name: &str) -> Option<Arc<Tileset>> {
        self.get_map(map_name).and_then(|map| {
            map.tilesets()
                .iter()
                .find(|ts| ts.name == tileset_name)
                .cloned()
        })
    }
}
