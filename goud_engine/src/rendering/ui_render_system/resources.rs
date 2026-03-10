use std::collections::HashMap;

use crate::assets::loaders::{FontAsset, FontLoader, TextureAsset, TextureLoader};
use crate::assets::{AssetHandle, AssetServer};
use crate::libs::graphics::backend::types::{
    TextureFilter, TextureFormat as BackendTextureFormat, TextureHandle, TextureWrap,
};
use crate::libs::graphics::backend::RenderBackend;
use crate::ui::{UI_DEFAULT_FONT_ASSET_PATH, UI_DEFAULT_FONT_FAMILY};

/// Ensures UI text and image asset loaders are registered on the asset server.
pub(crate) fn ensure_ui_asset_loaders(asset_server: &mut AssetServer) {
    if !asset_server.has_loader_for_type::<FontAsset>() {
        asset_server.register_loader(FontLoader);
    }
    if !asset_server.has_loader_for_type::<TextureAsset>() {
        asset_server.register_loader(TextureLoader);
    }
}

/// Resolves a font family to a loaded font handle.
///
/// Resolution chain:
/// 1. `fonts/{requested_family}.ttf`
/// 2. [`UI_DEFAULT_FONT_ASSET_PATH`]
/// 3. `None` (text draw skipped)
pub(crate) fn resolve_font(
    asset_server: &mut AssetServer,
    font_cache: &mut HashMap<String, AssetHandle<FontAsset>>,
    font_family: &str,
) -> Option<AssetHandle<FontAsset>> {
    let requested_family = normalized_font_family(font_family);
    let is_default_family = requested_family.eq_ignore_ascii_case(UI_DEFAULT_FONT_FAMILY);
    if let Some(handle) = font_cache.get(requested_family) {
        if asset_server.is_loaded(handle) {
            return Some(*handle);
        }
        font_cache.remove(requested_family);
    }

    let (resolved, cacheable) = if is_default_family {
        let handle = asset_server.load::<FontAsset>(UI_DEFAULT_FONT_ASSET_PATH);
        if asset_server.is_loaded(&handle) {
            (Some(handle), true)
        } else {
            (None, false)
        }
    } else {
        let candidate_path = format!("fonts/{requested_family}.ttf");
        let candidate_handle = asset_server.load::<FontAsset>(&candidate_path);
        if asset_server.is_loaded(&candidate_handle) {
            (Some(candidate_handle), true)
        } else {
            let fallback = asset_server.load::<FontAsset>(UI_DEFAULT_FONT_ASSET_PATH);
            if asset_server.is_loaded(&fallback) {
                (Some(fallback), false)
            } else {
                (None, false)
            }
        }
    };

    if let Some(handle) = resolved {
        if cacheable {
            font_cache.insert(requested_family.to_string(), handle);
        } else if asset_server.is_loaded(&handle) {
            font_cache.insert(UI_DEFAULT_FONT_FAMILY.to_string(), handle);
        }
    }
    resolved
}

/// Resolves a texture asset path to a backend texture, creating GPU texture on demand.
pub(crate) fn resolve_texture_asset(
    asset_server: &mut AssetServer,
    backend: &mut dyn RenderBackend,
    texture_cache: &mut HashMap<String, TextureHandle>,
    texture_path: &str,
) -> Option<TextureHandle> {
    if texture_path.is_empty() {
        return None;
    }

    if let Some(&cached) = texture_cache.get(texture_path) {
        if backend.is_texture_valid(cached) {
            return Some(cached);
        }
        texture_cache.remove(texture_path);
    }

    let texture_asset_handle = asset_server.load::<TextureAsset>(texture_path);
    if !asset_server.is_loaded(&texture_asset_handle) {
        return None;
    }

    let texture_asset = asset_server.get(&texture_asset_handle)?;
    let gpu_texture = backend
        .create_texture(
            texture_asset.width,
            texture_asset.height,
            BackendTextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::ClampToEdge,
            &texture_asset.data,
        )
        .ok()?;
    texture_cache.insert(texture_path.to_string(), gpu_texture);
    Some(gpu_texture)
}

fn normalized_font_family(font_family: &str) -> &str {
    let trimmed = font_family.trim();
    if trimmed.is_empty() {
        UI_DEFAULT_FONT_FAMILY
    } else {
        trimmed
    }
}
