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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiled_manager() {
        let mut tiled_manager = TiledManager::new();
        let map_id = tiled_manager.load_map(
            "test_map",
            "src/libs/graphics/components/tiled/_Tiled/Maps/Map.tmx",
            vec![1, 2, 3],
        );
        if let Err(e) = &map_id {
            println!("Error loading map: {}", e);
        }
        assert!(map_id.is_ok());
    }

    #[test]
    fn test_get_map_by_id() {
        let mut tiled_manager = TiledManager::new();
        let map_id = tiled_manager
            .load_map(
                "test_map",
                "src/libs/graphics/components/tiled/_Tiled/Maps/Map.tmx",
                vec![1, 2, 3],
            )
            .unwrap();

        // Test getting existing map
        let map = tiled_manager.get_map_by_id(map_id);
        assert!(map.is_some());
        assert_eq!(map.unwrap().id, map_id);

        // Test getting non-existent map
        let non_existent_map = tiled_manager.get_map_by_id(999);
        assert!(non_existent_map.is_none());
    }

    #[test]
    fn test_set_selected_map() {
        let mut tiled_manager = TiledManager::new();
        let map_id = tiled_manager
            .load_map(
                "test_map",
                "src/libs/graphics/components/tiled/_Tiled/Maps/Map.tmx",
                vec![1, 2, 3],
            )
            .unwrap();

        // Test setting valid map
        let result = tiled_manager.set_selected_map_by_id(map_id);
        assert!(result.is_ok());
        assert_eq!(tiled_manager.selected_map_id, Some(map_id));

        // Test setting invalid map
        let result = tiled_manager.set_selected_map_by_id(999);
        assert!(result.is_err());
        assert_eq!(tiled_manager.selected_map_id, Some(map_id)); // Should remain unchanged
    }

    #[test]
    fn test_clear_selected_map() {
        let mut tiled_manager = TiledManager::new();
        let map_id = tiled_manager
            .load_map(
                "test_map",
                "src/libs/graphics/components/tiled/_Tiled/Maps/Map.tmx",
                vec![1, 2, 3],
            )
            .unwrap();

        tiled_manager.set_selected_map_by_id(map_id).unwrap();
        assert_eq!(tiled_manager.selected_map_id, Some(map_id));

        tiled_manager.clear_selected_map();
        assert_eq!(tiled_manager.selected_map_id, None);
    }
}
