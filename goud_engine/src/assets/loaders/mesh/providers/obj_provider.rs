//! OBJ [`ModelProvider`] implementation.
//!
//! Wraps the existing [`parse_obj`](super::super::obj_parser::parse_obj)
//! parser behind the provider trait.  OBJ files do not carry skeleton or
//! animation data, so those fields are always empty.

use crate::assets::{AssetLoadError, LoadContext};

use super::super::provider::{ModelData, ModelProvider};

/// Wavefront OBJ model provider.
#[derive(Debug, Clone, Copy, Default)]
pub struct ObjProvider;

impl ModelProvider for ObjProvider {
    fn name(&self) -> &str {
        "OBJ"
    }

    fn extensions(&self) -> &[&str] {
        &["obj"]
    }

    fn load(&self, bytes: &[u8], _context: &mut LoadContext) -> Result<ModelData, AssetLoadError> {
        let mesh = super::super::obj_parser::parse_obj(bytes)?;
        Ok(ModelData {
            mesh,
            skeleton: None,
            animations: vec![],
        })
    }
}
