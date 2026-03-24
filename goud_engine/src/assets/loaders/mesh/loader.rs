//! [`MeshLoader`] -- dispatches to format-specific parsers via [`ModelProviderRegistry`].

use crate::assets::{Asset, AssetLoadError, AssetLoader, LoadContext};

use super::asset::MeshAsset;

/// Mesh format detected from the file extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshFormat {
    /// GLTF 2.0 (JSON text format).
    Gltf,
    /// GLB (GLTF binary container).
    Glb,
    /// Wavefront OBJ.
    Obj,
    /// Autodesk FBX.
    Fbx,
}

impl MeshFormat {
    /// Returns the format for a given file extension, or `None`.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "gltf" => Some(Self::Gltf),
            "glb" => Some(Self::Glb),
            "obj" => Some(Self::Obj),
            "fbx" => Some(Self::Fbx),
            _ => None,
        }
    }
}

/// Asset loader for 3D mesh files.
///
/// Supports GLTF/GLB, OBJ, and FBX formats when the `native` feature is
/// enabled.  Internally dispatches through a [`ModelProviderRegistry`] so
/// the loader never touches format-specific types directly.
///
/// Without `native`, all formats return `UnsupportedFormat`.
///
/// # Example
///
/// ```no_run
/// use goud_engine::assets::{AssetServer, loaders::mesh::{MeshLoader, MeshAsset}};
///
/// let mut server = AssetServer::new();
/// server.register_loader(MeshLoader::default());
///
/// let handle = server.load::<MeshAsset>("models/character.glb");
/// ```
#[derive(Debug, Clone, Default)]
pub struct MeshLoader;

impl MeshLoader {
    /// Creates a new mesh loader.
    pub fn new() -> Self {
        Self
    }
}

impl AssetLoader for MeshLoader {
    type Asset = MeshAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        MeshAsset::extensions()
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        #[cfg(not(feature = "native"))]
        {
            let _ = bytes;
            let ext = context.extension().unwrap_or("unknown");
            return Err(AssetLoadError::unsupported_format(ext));
        }

        #[cfg(feature = "native")]
        {
            let ext = context.extension().unwrap_or("unknown").to_string();

            // Validate the extension is a known format before dispatching.
            if MeshFormat::from_extension(&ext).is_none() {
                return Err(AssetLoadError::unsupported_format(&ext));
            }

            let registry = super::providers::default_registry();
            let model_data = registry.load(&ext, bytes, context)?;
            Ok(model_data.mesh)
        }
    }
}
