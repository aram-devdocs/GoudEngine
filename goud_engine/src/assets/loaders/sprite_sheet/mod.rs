//! Sprite-sheet and atlas descriptor loading.

mod asset;
mod loader;

#[cfg(test)]
mod tests;

pub use asset::{SpriteRegion, SpriteSheetAsset};
pub use loader::SpriteSheetLoader;
