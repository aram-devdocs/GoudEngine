use super::*;
use crate::libs::graphics::backend::null::NullBackend;
use crate::libs::graphics::backend::{ShaderLanguage, TextureOps};
use crate::rendering::sprite_batch::config::SpriteBatchConfig;
use image::{ImageBuffer, ImageFormat, Rgba};

fn create_test_png(width: u32, height: u32) -> Vec<u8> {
    let image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |x, y| {
        if (x + y) % 2 == 0 {
            Rgba([255, 0, 0, 255])
        } else {
            Rgba([0, 255, 0, 255])
        }
    });
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut std::io::Cursor::new(&mut bytes), ImageFormat::Png)
        .expect("test png encoding should succeed");
    bytes
}

#[test]
fn test_default_sprite_shader_loads_through_asset_server() {
    let mut asset_server = AssetServer::new();
    let handle = ensure_default_sprite_shader_loaded(&mut asset_server, ShaderLanguage::Glsl);
    let shader = asset_server
        .get(&handle)
        .expect("default sprite shader should be loaded");

    assert!(shader.get_stage(ShaderStage::Vertex).is_some());
    assert!(shader.get_stage(ShaderStage::Fragment).is_some());
}

#[test]
fn test_default_sprite_shader_loads_wgsl_through_asset_server() {
    let mut asset_server = AssetServer::new();
    let handle = ensure_default_sprite_shader_loaded(&mut asset_server, ShaderLanguage::Wgsl);
    let shader = asset_server
        .get(&handle)
        .expect("default WGSL sprite shader should be loaded");

    assert!(shader.get_stage(ShaderStage::Vertex).is_some());
    assert!(shader.get_stage(ShaderStage::Fragment).is_some());
}

#[test]
fn test_resolve_texture_reuses_cached_handle_and_reuploads_when_invalidated() {
    let mut asset_server = AssetServer::new();
    ensure_sprite_asset_loaders(&mut asset_server);
    let texture =
        asset_server.load_from_bytes::<TextureAsset>("tests/sprite.png", &create_test_png(2, 2));

    let mut batch = SpriteBatch::new(NullBackend::new(), SpriteBatchConfig::default()).unwrap();

    let first = batch
        .resolve_texture(texture, &asset_server)
        .expect("first texture upload should succeed");
    let second = batch
        .resolve_texture(texture, &asset_server)
        .expect("cached texture lookup should succeed");

    assert_eq!(
        first, second,
        "second lookup should reuse the cached GPU handle"
    );

    assert!(
        batch.backend.destroy_texture(first),
        "test should be able to invalidate the cached texture handle"
    );

    let third = batch
        .resolve_texture(texture, &asset_server)
        .expect("invalidated texture should be uploaded again");

    assert_ne!(
        first, third,
        "stale cached handles should be replaced after backend invalidation"
    );
}
