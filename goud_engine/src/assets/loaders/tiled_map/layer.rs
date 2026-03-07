//! Tiled map layer types.
//!
//! These types represent the asset-level data for tile and object layers
//! extracted from a Tiled map file. They are decoupled from the `tiled`
//! crate's internal representations.

use std::collections::HashMap;

/// A tile layer containing a grid of tile GID values.
///
/// GID 0 represents an empty tile. Non-zero GIDs reference tiles
/// from the map's tilesets.
#[derive(Debug, Clone)]
pub struct TileLayer {
    /// The layer name as set in the Tiled editor.
    pub name: String,
    /// Width of the layer in tiles.
    pub width: u32,
    /// Height of the layer in tiles.
    pub height: u32,
    /// Flat array of GID values in row-major order. GID 0 = empty.
    pub tiles: Vec<u32>,
    /// Whether the layer is visible.
    pub visible: bool,
    /// Layer opacity from 0.0 (transparent) to 1.0 (opaque).
    pub opacity: f32,
}

impl TileLayer {
    /// Returns the GID at the given tile coordinate, or `None` if out of bounds.
    pub fn get_gid(&self, col: u32, row: u32) -> Option<u32> {
        if col < self.width && row < self.height {
            let idx = (row * self.width + col) as usize;
            self.tiles.get(idx).copied()
        } else {
            None
        }
    }

    /// Returns `true` if the tile at the given coordinate is empty (GID 0).
    pub fn is_empty_at(&self, col: u32, row: u32) -> bool {
        self.get_gid(col, row).is_none_or(|gid| gid == 0)
    }
}

/// An object layer containing positioned objects.
#[derive(Debug, Clone)]
pub struct ObjectLayer {
    /// The layer name as set in the Tiled editor.
    pub name: String,
    /// The objects in this layer.
    pub objects: Vec<MapObject>,
    /// Whether the layer is visible.
    pub visible: bool,
}

/// A single object in an object layer.
///
/// Objects represent positioned entities such as spawn points,
/// collision regions, or trigger zones.
#[derive(Debug, Clone)]
pub struct MapObject {
    /// Unique ID within the map.
    pub id: u32,
    /// Object name as set in the Tiled editor.
    pub name: String,
    /// Object type/class as set in the Tiled editor.
    pub object_type: String,
    /// X position in pixels.
    pub x: f32,
    /// Y position in pixels.
    pub y: f32,
    /// Width in pixels.
    pub width: f32,
    /// Height in pixels.
    pub height: f32,
    /// Custom properties as string key-value pairs.
    pub properties: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_layer_get_gid_valid() {
        let layer = TileLayer {
            name: "ground".to_string(),
            width: 3,
            height: 2,
            tiles: vec![1, 2, 3, 4, 5, 6],
            visible: true,
            opacity: 1.0,
        };
        assert_eq!(layer.get_gid(0, 0), Some(1));
        assert_eq!(layer.get_gid(2, 0), Some(3));
        assert_eq!(layer.get_gid(0, 1), Some(4));
        assert_eq!(layer.get_gid(2, 1), Some(6));
    }

    #[test]
    fn test_tile_layer_get_gid_out_of_bounds() {
        let layer = TileLayer {
            name: "ground".to_string(),
            width: 2,
            height: 2,
            tiles: vec![1, 2, 3, 4],
            visible: true,
            opacity: 1.0,
        };
        assert_eq!(layer.get_gid(2, 0), None);
        assert_eq!(layer.get_gid(0, 2), None);
        assert_eq!(layer.get_gid(5, 5), None);
    }

    #[test]
    fn test_tile_layer_is_empty_at() {
        let layer = TileLayer {
            name: "sparse".to_string(),
            width: 3,
            height: 1,
            tiles: vec![0, 5, 0],
            visible: true,
            opacity: 1.0,
        };
        assert!(layer.is_empty_at(0, 0));
        assert!(!layer.is_empty_at(1, 0));
        assert!(layer.is_empty_at(2, 0));
        // Out-of-bounds treated as empty.
        assert!(layer.is_empty_at(3, 0));
    }

    #[test]
    fn test_object_layer_construction() {
        let obj = MapObject {
            id: 1,
            name: "spawn".to_string(),
            object_type: "player_spawn".to_string(),
            x: 100.0,
            y: 200.0,
            width: 32.0,
            height: 32.0,
            properties: HashMap::from([("difficulty".to_string(), "hard".to_string())]),
        };
        let layer = ObjectLayer {
            name: "objects".to_string(),
            objects: vec![obj],
            visible: true,
        };
        assert_eq!(layer.objects.len(), 1);
        assert_eq!(layer.objects[0].name, "spawn");
        assert_eq!(
            layer.objects[0].properties.get("difficulty"),
            Some(&"hard".to_string())
        );
    }

    #[test]
    fn test_map_object_default_properties() {
        let obj = MapObject {
            id: 42,
            name: String::new(),
            object_type: String::new(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            properties: HashMap::new(),
        };
        assert_eq!(obj.id, 42);
        assert!(obj.name.is_empty());
        assert!(obj.properties.is_empty());
    }
}
