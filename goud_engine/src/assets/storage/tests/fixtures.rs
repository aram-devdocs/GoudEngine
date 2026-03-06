//! Shared test asset types used across storage test modules.

use crate::assets::{Asset, AssetType};

#[derive(Clone, Debug, PartialEq)]
pub struct TestTexture {
    pub width: u32,
    pub height: u32,
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

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SimpleAsset {
    pub value: i32,
}

impl Asset for SimpleAsset {}
