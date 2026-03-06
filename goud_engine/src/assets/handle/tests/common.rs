//! Shared test asset types used across handle test modules.

use crate::assets::{Asset, AssetType};

#[derive(Clone, Debug, PartialEq)]
pub struct TestTexture {
    #[allow(dead_code)]
    pub width: u32,
}

impl Asset for TestTexture {
    fn asset_type_name() -> &'static str {
        "TestTexture"
    }

    fn asset_type() -> AssetType {
        AssetType::Texture
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TestAudio {
    #[allow(dead_code)]
    pub duration: f32,
}

impl Asset for TestAudio {
    fn asset_type_name() -> &'static str {
        "TestAudio"
    }

    fn asset_type() -> AssetType {
        AssetType::Audio
    }
}

/// Simple asset using default trait implementations.
pub struct SimpleAsset;
impl Asset for SimpleAsset {}
