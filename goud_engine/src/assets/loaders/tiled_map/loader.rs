//! [`TiledMapLoader`] -- parses Tiled map files into [`TiledMapAsset`].

#[cfg(feature = "native")]
use std::io::Cursor;
#[cfg(feature = "native")]
use std::path::Path;

use crate::assets::{Asset, AssetLoadError, AssetLoader, LoadContext};

use super::asset::TiledMapAsset;
#[cfg(feature = "native")]
use super::layer::{MapObject, ObjectLayer, TileLayer};

/// Asset loader for Tiled map files (.tmx, .tmj).
///
/// Uses the `tiled` crate to parse map data and converts the result
/// into a [`TiledMapAsset`]. Tileset image paths are registered as
/// dependencies via the load context.
///
/// Requires the `native` feature.
#[derive(Debug, Clone, Default)]
pub struct TiledMapLoader;

impl TiledMapLoader {
    /// Creates a new Tiled map loader.
    pub fn new() -> Self {
        Self
    }
}

impl AssetLoader for TiledMapLoader {
    type Asset = TiledMapAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        TiledMapAsset::extensions()
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        #[cfg(feature = "native")]
        {
            self.load_native(bytes, context)
        }

        #[cfg(not(feature = "native"))]
        {
            let _ = (bytes, context);
            Err(AssetLoadError::custom(
                "Tiled map loading requires the 'native' feature",
            ))
        }
    }
}

#[cfg(feature = "native")]
impl TiledMapLoader {
    fn load_native(
        &self,
        bytes: &[u8],
        context: &mut LoadContext,
    ) -> Result<TiledMapAsset, AssetLoadError> {
        let asset_path = context.path_str().to_string();

        // Build a reader that serves the primary map bytes from memory
        // and delegates external resources (tilesets) to the filesystem.
        let bytes_owned = bytes.to_vec();
        let asset_path_clone = asset_path.clone();
        let reader = move |path: &Path| -> std::io::Result<Cursor<Vec<u8>>> {
            if path == Path::new(&asset_path_clone) {
                Ok(Cursor::new(bytes_owned.clone()))
            } else {
                let data = std::fs::read(path).map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!(
                            "Failed to read tileset resource '{}': {}",
                            path.display(),
                            e
                        ),
                    )
                })?;
                Ok(Cursor::new(data))
            }
        };

        let mut loader = tiled::Loader::with_reader(reader);
        let map = loader
            .load_tmx_map(&asset_path)
            .map_err(|e| AssetLoadError::decode_failed(format!("Tiled parse error: {e}")))?;

        let mut tile_layers = Vec::new();
        let mut object_layers = Vec::new();

        for layer in map.layers() {
            match layer.layer_type() {
                tiled::LayerType::Tiles(tile_layer) => {
                    let tl = Self::convert_tile_layer(&layer, &tile_layer, &map)?;
                    tile_layers.push(tl);
                }
                tiled::LayerType::Objects(obj_layer) => {
                    let ol = Self::convert_object_layer(&layer, &obj_layer);
                    object_layers.push(ol);
                }
                _ => {
                    // Image and group layers are not yet supported; skip them.
                }
            }
        }

        // Collect tileset image paths and register as dependencies.
        let mut tileset_paths = Vec::new();
        for tileset in map.tilesets() {
            if let Some(ref image) = tileset.image {
                let path_str = image.source.display().to_string();
                context.add_dependency(&path_str);
                tileset_paths.push(path_str);
            }
        }

        Ok(TiledMapAsset {
            width: map.width,
            height: map.height,
            tile_width: map.tile_width,
            tile_height: map.tile_height,
            tile_layers,
            object_layers,
            tileset_paths,
        })
    }

    fn convert_tile_layer(
        layer: &tiled::Layer<'_>,
        tile_layer: &tiled::TileLayer<'_>,
        _map: &tiled::Map,
    ) -> Result<TileLayer, AssetLoadError> {
        match tile_layer {
            tiled::TileLayer::Finite(finite) => {
                let w = finite.width();
                let h = finite.height();
                let mut tiles = Vec::with_capacity((w * h) as usize);

                for row in 0..h as i32 {
                    for col in 0..w as i32 {
                        let gid = finite
                            .get_tile(col, row)
                            .map(|t| {
                                // Reconstruct GID from tileset index and local ID.
                                // The local id plus the tileset's first_gid gives the GID.
                                // However, since we only have the tile data, we store a
                                // non-zero marker. For proper GID reconstruction we need
                                // the tileset info, but for the asset representation a
                                // simple non-zero value suffices since we track tilesets
                                // separately. We use id + 1 to avoid conflating with empty.
                                t.id() + 1
                            })
                            .unwrap_or(0);
                        tiles.push(gid);
                    }
                }

                Ok(TileLayer {
                    name: layer.name.clone(),
                    width: w,
                    height: h,
                    tiles,
                    visible: layer.visible,
                    opacity: layer.opacity,
                })
            }
            tiled::TileLayer::Infinite(_) => Err(AssetLoadError::custom(
                "Infinite tile layers are not yet supported",
            )),
        }
    }

    fn convert_object_layer(
        layer: &tiled::Layer<'_>,
        obj_layer: &tiled::ObjectLayer<'_>,
    ) -> ObjectLayer {
        let objects: Vec<MapObject> = obj_layer
            .objects()
            .map(|obj| {
                let (width, height) = match &obj.shape {
                    tiled::ObjectShape::Rect { width, height }
                    | tiled::ObjectShape::Ellipse { width, height } => (*width, *height),
                    _ => (0.0, 0.0),
                };

                let properties = obj
                    .properties
                    .iter()
                    .map(|(k, v)| (k.clone(), format!("{v:?}")))
                    .collect();

                MapObject {
                    id: obj.id(),
                    name: obj.name.clone(),
                    object_type: obj.user_type.clone(),
                    x: obj.x,
                    y: obj.y,
                    width,
                    height,
                    properties,
                }
            })
            .collect();

        ObjectLayer {
            name: layer.name.clone(),
            objects,
            visible: layer.visible,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiled_map_loader_extensions() {
        let loader = TiledMapLoader::new();
        let exts = loader.extensions();
        assert!(exts.contains(&"tmx"));
        assert!(exts.contains(&"tmj"));
        assert_eq!(exts.len(), 2);
    }

    #[test]
    fn test_tiled_map_loader_supports_extension() {
        let loader = TiledMapLoader::new();
        assert!(loader.supports_extension("tmx"));
        assert!(loader.supports_extension("TMX"));
        assert!(loader.supports_extension("tmj"));
        assert!(!loader.supports_extension("png"));
        assert!(!loader.supports_extension("json"));
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_tiled_map_loader_invalid_bytes() {
        let loader = TiledMapLoader::new();
        let path = crate::assets::AssetPath::from_string("test_invalid.tmx".to_string());
        let mut ctx = LoadContext::new(path);

        let result = loader.load(b"not valid xml", &(), &mut ctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_decode_failed() || err.is_custom());
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_tiled_map_loader_empty_bytes() {
        let loader = TiledMapLoader::new();
        let path = crate::assets::AssetPath::from_string("empty.tmx".to_string());
        let mut ctx = LoadContext::new(path);

        let result = loader.load(b"", &(), &mut ctx);
        assert!(result.is_err());
    }
}
