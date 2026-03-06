//! Shared test fixture types used across asset test modules.

use crate::assets::asset::{asset_type::AssetType, trait_def::Asset};

pub(super) struct TestTexture {
    #[allow(dead_code)]
    pub width: u32,
    #[allow(dead_code)]
    pub height: u32,
    #[allow(dead_code)]
    pub data: Vec<u8>,
}

impl Asset for TestTexture {
    fn asset_type_name() -> &'static str {
        "TestTexture"
    }

    fn asset_type() -> AssetType {
        AssetType::Texture
    }

    fn extensions() -> &'static [&'static str] {
        &["png", "jpg", "jpeg"]
    }
}

pub(super) struct TestAudio {
    #[allow(dead_code)]
    pub samples: Vec<f32>,
    #[allow(dead_code)]
    pub sample_rate: u32,
}

impl Asset for TestAudio {
    fn asset_type_name() -> &'static str {
        "TestAudio"
    }

    fn asset_type() -> AssetType {
        AssetType::Audio
    }

    fn extensions() -> &'static [&'static str] {
        &["wav", "ogg", "mp3"]
    }
}

/// Simple asset using default implementations.
pub(super) struct SimpleAsset {
    #[allow(dead_code)]
    pub value: i32,
}

impl Asset for SimpleAsset {}
