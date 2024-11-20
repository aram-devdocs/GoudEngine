use std::collections::HashMap;
use std::ffi::c_uint;
use std::path::Path;
use std::rc::Rc;
use tiled::{Loader, Map};

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
        texture_ids: HashMap<String, u32>,
    ) -> Result<c_uint, String> {
        let map = self
            .loader
            .load_tmx_map(Path::new(file_path))
            .map_err(|e| format!("Failed to load map: {}", e))?;

        // Load associated tilesets
        for tileset in map.tilesets() {
            println!("Loading tileset: {}", tileset.name);
            if let Some(texture_id) = texture_ids.get(&tileset.name) {
                println!("Using texture ID: {}", texture_id);
            } else {
                println!("No texture ID found for tileset: {}", tileset.name);
            }
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
        // Ok(map_rc)
        Ok(tiled_id)
    }

    pub fn get_map_by_id(&self, map_id: c_uint) -> Option<Rc<Map>> {
        // self.maps.get(map_name).map(|tiled| Rc::clone(&tiled.map))
        self.maps
            .values()
            .find(|tiled| tiled.id == map_id)
            .map(|tiled| Rc::clone(&tiled.map))
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

    fn setup_tiled_manager() -> TiledManager {
        let mut tiled_manager = TiledManager::new();
        let mut texture_ids = HashMap::new();
        texture_ids.insert("Tileset".to_string(), 1);
        tiled_manager
            .load_map(
                "map",
                "/Users/aramhammoudeh/dev/game/GoudEngine/goud_engine/src/libs/platform/graphics/rendering/tiled/_Tiled/Maps/Map.tmx",
                texture_ids,
            )
            .unwrap();
        tiled_manager
    }

    #[test]
    fn test_load_map() {
        let tiled_manager = setup_tiled_manager();
        let _map_id = tiled_manager.maps.get("map").unwrap().id;
    }

    #[test]
    fn test_get_map_by_id() {
        let tiled_manager = setup_tiled_manager();
        let map_id = tiled_manager.maps.get("map").unwrap().id;
        let map = tiled_manager.get_map_by_id(map_id);
        assert!(map.is_some());
    }

    #[test]
    fn test_set_selected_map_by_id() {
        let mut tiled_manager = setup_tiled_manager();
        let map_id = tiled_manager.maps.get("map").unwrap().id;
        let result = tiled_manager.set_selected_map_by_id(map_id);
        assert!(result.is_ok());
        assert_eq!(tiled_manager.selected_map_id, Some(map_id));
    }

    #[test]
    fn test_clear_selected_map() {
        let mut tiled_manager = setup_tiled_manager();
        tiled_manager.set_selected_map_by_id(0).unwrap();
        tiled_manager.clear_selected_map();
        assert!(tiled_manager.selected_map_id.is_none());
    }
}
