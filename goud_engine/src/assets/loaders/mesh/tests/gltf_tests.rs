use super::*;
use crate::assets::loaders::{ensure_3d_asset_loaders, TextureAsset};
use crate::assets::AssetServer;
use image::{ImageBuffer, ImageFormat, Rgba};
use std::fs;

// Split out of tests.rs to satisfy the repo's 500-line file limit.
include!("gltf_tests_impl.in");
