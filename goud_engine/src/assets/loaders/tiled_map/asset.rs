//! [`TiledMapAsset`] -- parsed Tiled map data.

use crate::assets::{Asset, AssetType};

use super::layer::{ObjectLayer, TileLayer};

/// A loaded Tiled map asset containing parsed map data.
///
/// This is the asset-level representation of a Tiled `.tmx` or `.tmj`
/// file. It stores the map dimensions, layers, and tileset references
/// without holding any GPU resources or rendering state.
#[derive(Debug, Clone)]
pub struct TiledMapAsset {
    /// Width of the map in tiles.
    pub width: u32,
    /// Height of the map in tiles.
    pub height: u32,
    /// Width of each tile in pixels.
    pub tile_width: u32,
    /// Height of each tile in pixels.
    pub tile_height: u32,
    /// Tile layers extracted from the map.
    pub tile_layers: Vec<TileLayer>,
    /// Object layers extracted from the map.
    pub object_layers: Vec<ObjectLayer>,
    /// Paths to tileset image files referenced by this map.
    pub tileset_paths: Vec<String>,
}

impl TiledMapAsset {
    /// Returns the map dimensions in pixels.
    pub fn pixel_dimensions(&self) -> (u32, u32) {
        (self.width * self.tile_width, self.height * self.tile_height)
    }

    /// Returns the total number of layers (tile + object).
    pub fn layer_count(&self) -> usize {
        self.tile_layers.len() + self.object_layers.len()
    }
}

impl Asset for TiledMapAsset {
    fn asset_type_name() -> &'static str {
        "TiledMap"
    }

    fn asset_type() -> AssetType {
        AssetType::TiledMap
    }

    fn extensions() -> &'static [&'static str] {
        &["tmx", "tmj"]
    }
}

/// Computes the range of visible tiles given a camera position and viewport.
///
/// Returns `(col_start, col_end, row_start, row_end)` where the ranges
/// are clamped to valid map coordinates. The end values are exclusive.
///
/// This is pure math with no GPU dependency, suitable for camera culling
/// to avoid iterating over off-screen tiles.
pub fn visible_tile_range(
    camera_x: f32,
    camera_y: f32,
    viewport_w: f32,
    viewport_h: f32,
    tile_w: u32,
    tile_h: u32,
    map_w: u32,
    map_h: u32,
) -> (u32, u32, u32, u32) {
    if tile_w == 0 || tile_h == 0 || map_w == 0 || map_h == 0 {
        return (0, 0, 0, 0);
    }

    let tw = tile_w as f32;
    let th = tile_h as f32;

    // Compute the tile column/row at the top-left corner of the viewport.
    let col_start_f = camera_x / tw;
    let row_start_f = camera_y / th;

    // Compute the tile column/row at the bottom-right corner of the viewport.
    let col_end_f = (camera_x + viewport_w) / tw;
    let row_end_f = (camera_y + viewport_h) / th;

    // Clamp to valid map bounds. Start is floored, end is ceiled.
    let col_start = col_start_f.floor().max(0.0) as u32;
    let row_start = row_start_f.floor().max(0.0) as u32;
    let col_end = (col_end_f.ceil() as u32).min(map_w);
    let row_end = (row_end_f.ceil() as u32).min(map_h);

    // Ensure start does not exceed end (can happen if camera is beyond map).
    let col_start = col_start.min(col_end);
    let row_start = row_start.min(row_end);

    (col_start, col_end, row_start, row_end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::loaders::tiled_map::layer::{ObjectLayer, TileLayer};

    #[test]
    fn test_tiled_map_asset_pixel_dimensions() {
        let asset = TiledMapAsset {
            width: 20,
            height: 15,
            tile_width: 32,
            tile_height: 32,
            tile_layers: vec![],
            object_layers: vec![],
            tileset_paths: vec![],
        };
        assert_eq!(asset.pixel_dimensions(), (640, 480));
    }

    #[test]
    fn test_tiled_map_asset_layer_count() {
        let asset = TiledMapAsset {
            width: 10,
            height: 10,
            tile_width: 16,
            tile_height: 16,
            tile_layers: vec![
                TileLayer {
                    name: "bg".to_string(),
                    width: 10,
                    height: 10,
                    tiles: vec![0; 100],
                    visible: true,
                    opacity: 1.0,
                },
                TileLayer {
                    name: "fg".to_string(),
                    width: 10,
                    height: 10,
                    tiles: vec![0; 100],
                    visible: true,
                    opacity: 1.0,
                },
            ],
            object_layers: vec![ObjectLayer {
                name: "objs".to_string(),
                objects: vec![],
                visible: true,
            }],
            tileset_paths: vec![],
        };
        assert_eq!(asset.layer_count(), 3);
    }

    #[test]
    fn test_asset_type_metadata() {
        assert_eq!(TiledMapAsset::asset_type_name(), "TiledMap");
        assert_eq!(TiledMapAsset::asset_type(), AssetType::TiledMap);
        assert_eq!(TiledMapAsset::extensions(), &["tmx", "tmj"]);
    }

    // -- visible_tile_range tests --

    #[test]
    fn test_visible_tile_range_full_map() {
        // Camera at origin, viewport covers entire map.
        let (cs, ce, rs, re) = visible_tile_range(0.0, 0.0, 320.0, 320.0, 32, 32, 10, 10);
        assert_eq!((cs, ce, rs, re), (0, 10, 0, 10));
    }

    #[test]
    fn test_visible_tile_range_partial() {
        // Camera offset into the map.
        let (cs, ce, rs, re) = visible_tile_range(48.0, 64.0, 128.0, 96.0, 32, 32, 10, 10);
        // col_start = floor(48/32) = 1, col_end = ceil((48+128)/32) = ceil(5.5) = 6
        // row_start = floor(64/32) = 2, row_end = ceil((64+96)/32) = ceil(5.0) = 5
        assert_eq!((cs, ce, rs, re), (1, 6, 2, 5));
    }

    #[test]
    fn test_visible_tile_range_clamped_to_map() {
        // Camera near bottom-right, viewport extends past map.
        let (cs, ce, rs, re) = visible_tile_range(256.0, 256.0, 200.0, 200.0, 32, 32, 10, 10);
        assert_eq!(cs, 8);
        assert_eq!(ce, 10); // Clamped to map_w
        assert_eq!(rs, 8);
        assert_eq!(re, 10); // Clamped to map_h
    }

    #[test]
    fn test_visible_tile_range_negative_camera() {
        // Camera with negative position (scrolled before map origin).
        let (cs, ce, rs, re) = visible_tile_range(-16.0, -16.0, 128.0, 128.0, 32, 32, 10, 10);
        assert_eq!(cs, 0); // Clamped to 0
        assert_eq!(rs, 0); // Clamped to 0
        assert_eq!(ce, 4); // ceil((-16+128)/32) = ceil(3.5) = 4
        assert_eq!(re, 4);
    }

    #[test]
    fn test_visible_tile_range_zero_tile_size() {
        let (cs, ce, rs, re) = visible_tile_range(0.0, 0.0, 100.0, 100.0, 0, 32, 10, 10);
        assert_eq!((cs, ce, rs, re), (0, 0, 0, 0));
    }

    #[test]
    fn test_visible_tile_range_zero_map_size() {
        let (cs, ce, rs, re) = visible_tile_range(0.0, 0.0, 100.0, 100.0, 32, 32, 0, 0);
        assert_eq!((cs, ce, rs, re), (0, 0, 0, 0));
    }

    #[test]
    fn test_visible_tile_range_camera_beyond_map() {
        // Camera entirely past the map.
        let (cs, ce, rs, re) = visible_tile_range(500.0, 500.0, 100.0, 100.0, 32, 32, 10, 10);
        // col_start = floor(500/32) = 15, clamped to col_end = min(ceil(600/32),10) = 10
        // So col_start = min(15, 10) = 10, col_end = 10 => empty range.
        assert_eq!(cs, ce);
        assert_eq!(rs, re);
    }

    #[test]
    fn test_visible_tile_range_exact_tile_boundary() {
        // Camera aligned exactly to tile boundaries.
        let (cs, ce, rs, re) = visible_tile_range(64.0, 96.0, 128.0, 64.0, 32, 32, 20, 20);
        // col: floor(64/32)=2, ceil((64+128)/32)=ceil(6)=6
        // row: floor(96/32)=3, ceil((96+64)/32)=ceil(5)=5
        assert_eq!((cs, ce, rs, re), (2, 6, 3, 5));
    }
}
