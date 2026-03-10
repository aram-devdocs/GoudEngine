//! Async texture loading for the wasm backend.
//!
//! Fetches image data via the browser Fetch API, decodes it with the
//! `image` crate, and uploads RGBA pixels to a wgpu texture.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use super::sprite_renderer::{create_texture_entry, TextureEntry};

/// Fetches raw bytes from a URL using the browser Fetch API.
#[allow(dead_code)]
pub async fn fetch_bytes(url: &str) -> Result<Vec<u8>, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no global window"))?;
    let resp_val = JsFuture::from(window.fetch_with_str(url)).await?;
    let resp: web_sys::Response = resp_val.dyn_into()?;

    if !resp.ok() {
        return Err(JsValue::from_str(&format!(
            "HTTP {} fetching {}",
            resp.status(),
            url
        )));
    }

    let buffer = JsFuture::from(
        resp.array_buffer()
            .map_err(|_| JsValue::from_str("array_buffer() failed"))?,
    )
    .await?;
    let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
    Ok(bytes)
}

/// Loads an image from a URL and creates a GPU texture.
///
/// Returns a [`TextureEntry`] containing the wgpu texture view and a
/// pre-built bind group ready for use with the sprite renderer.
#[allow(dead_code)]
pub async fn load_texture_from_url(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    url: &str,
) -> Result<TextureEntry, JsValue> {
    let bytes = fetch_bytes(url).await?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| JsValue::from_str(&format!("Image decode error: {}", e)))?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    Ok(create_texture_entry(
        device, queue, layout, sampler, width, height, &rgba,
    ))
}
