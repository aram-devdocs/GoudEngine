use std::collections::HashMap;

use crate::assets::{AssetLoader, LoadContext};

use super::{SpriteRegion, SpriteSheetAsset, SpriteSheetLoader};

#[test]
fn test_sprite_sheet_asset_accessors() {
    let mut regions = HashMap::new();
    regions.insert("idle".to_string(), SpriteRegion::new(0.0, 0.0, 16.0, 16.0));

    let asset = SpriteSheetAsset::new("textures/player.png", regions);
    assert_eq!(asset.texture_path(), "textures/player.png");
    assert_eq!(asset.region("idle").unwrap().rect.width, 16.0);
    assert!(asset.region("missing").is_none());
}

#[test]
fn test_sprite_sheet_loader_parses_descriptor() {
    let bytes = br#"{
        "name": "player",
        "texture_path": "textures/player.png",
        "regions": {
            "idle": { "x": 0.0, "y": 0.0, "width": 32.0, "height": 48.0 },
            "run_1": { "x": 32.0, "y": 0.0, "width": 32.0, "height": 48.0 }
        }
    }"#;

    let loader = SpriteSheetLoader;
    let mut context = LoadContext::new("sprites/player.sheet.json".into());
    let asset = loader
        .load(bytes, &(), &mut context)
        .expect("load sprite sheet");

    assert_eq!(asset.name.as_deref(), Some("player"));
    assert_eq!(asset.texture_path(), "textures/player.png");
    assert_eq!(asset.region("idle").unwrap().rect.height, 48.0);
    assert!(context
        .into_dependencies()
        .contains(&"textures/player.png".to_string()));
}

#[test]
fn test_sprite_sheet_loader_rejects_invalid_region_size() {
    let bytes = br#"{
        "texture_path": "textures/player.png",
        "regions": {
            "broken": { "x": 0.0, "y": 0.0, "width": 0.0, "height": 48.0 }
        }
    }"#;

    let loader = SpriteSheetLoader;
    let mut context = LoadContext::new("sprites/player.sheet.json".into());
    assert!(loader.load(bytes, &(), &mut context).is_err());
}
