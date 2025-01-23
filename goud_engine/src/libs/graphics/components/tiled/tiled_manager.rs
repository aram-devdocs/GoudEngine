use std::collections::HashMap;
use std::ffi::c_uint;
use std::rc::Rc;
use tiled::Loader;

use crate::types::{Tiled, TiledManager};

impl TiledManager {
    pub fn new() -> Self {
        Self {
            loader: Loader::new(),
            maps: HashMap::new(),
            selected_map_id: None,
        }
    }

    pub fn load_map(
        &mut self,
        map_name: &str,
        file_path: &str,
        texture_ids: Vec<c_uint>,
    ) -> Result<c_uint, String> {
        // Resolve the file_path relative to the current working directory
        let full_path = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?
            .join(file_path);

        println!("Loading map from: {}", full_path.display());

        let map = self
            .loader
            .load_tmx_map(&full_path)
            .map_err(|e| format!("Failed to load map from '{}': {}", full_path.display(), e))?;

        // Load associated tilesets
        for tileset in map.tilesets() {
            println!("Loading tileset: {}", tileset.name);
            println!("Using texture IDs: {:?}", texture_ids);
        }

        let map_rc = Rc::new(map);
        let tiled_id = self.maps.len() as c_uint;
        self.maps.insert(
            map_name.to_string(),
            Tiled {
                id: tiled_id,
                map: Rc::clone(&map_rc),
                texture_ids,
            },
        );
        Ok(tiled_id)
    }

    pub fn get_map_by_id(&self, map_id: c_uint) -> Option<&Tiled> {
        self.maps.values().find(|tiled| tiled.id == map_id)
    }

    pub fn set_selected_map_by_id(&mut self, map_id: c_uint) -> Result<(), String> {
        if self.get_map_by_id(map_id).is_none() {
            return Err("Map not found".into());
        }
        self.selected_map_id = Some(map_id);
        Ok(())
    }

    pub fn clear_selected_map(&mut self) {
        self.selected_map_id = None;
    }
}
