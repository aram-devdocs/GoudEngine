//! FBX [`ModelProvider`] implementation.
//!
//! Uses the `fbxcel` crate to parse binary FBX files.  Extracts mesh
//! geometry, skeleton (bones + skin weights), and keyframe animations.

mod animation;
mod geometry;
mod helpers;
mod skeleton;
#[cfg(test)]
mod tests;

use crate::assets::loaders::mesh::provider::{ModelData, ModelProvider};
use crate::assets::{AssetLoadError, LoadContext};

use helpers::parse_connections;

/// Default roughness for FBX materials that don't specify one.
const DEFAULT_FBX_ROUGHNESS: f32 = 0.5;

/// FBX model provider backed by the `fbxcel` crate.
#[derive(Debug, Clone, Copy, Default)]
pub struct FbxProvider;

impl ModelProvider for FbxProvider {
    fn name(&self) -> &str {
        "FBX"
    }

    fn extensions(&self) -> &[&str] {
        &["fbx"]
    }

    fn load(&self, bytes: &[u8], _context: &mut LoadContext) -> Result<ModelData, AssetLoadError> {
        use fbxcel::pull_parser::any::AnyParser;
        use fbxcel::tree::v7400::Loader as TreeLoader;
        use std::io::Cursor;

        let cursor = Cursor::new(bytes);
        let any_parser = AnyParser::from_seekable_reader(cursor)
            .map_err(|e| AssetLoadError::decode_failed(format!("FBX parse error: {e}")))?;
        let mut parser = match any_parser {
            AnyParser::V7400(p) => p,
            _ => return Err(AssetLoadError::decode_failed("Unsupported FBX version")),
        };
        let (tree, _footer) = TreeLoader::new()
            .load(&mut parser)
            .map_err(|e| AssetLoadError::decode_failed(format!("FBX tree load error: {e}")))?;

        let root = tree.root();
        let conns = parse_connections(&root);
        let materials = geometry::extract_materials(&root);
        let (mesh, vertex_to_cp, geom_id) = geometry::extract_geometry(&root, &conns, &materials)?;
        let skeleton =
            skeleton::extract_skeleton(&root, &conns, geom_id, &vertex_to_cp, mesh.vertices.len());
        let animations = animation::extract_animations(&root, &conns, &skeleton);

        Ok(ModelData {
            mesh,
            skeleton,
            animations,
        })
    }
}
