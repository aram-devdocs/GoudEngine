//! Browser Fetch API bridge for async asset loading on web targets.
//!
//! Uses `web-sys` and `wasm-bindgen-futures` to perform HTTP fetch requests
//! from WebAssembly, returning raw bytes for the asset loader pipeline.

use crate::assets::AssetLoadError;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// Fetches raw bytes from a URL using the browser Fetch API.
///
/// # Arguments
///
/// * `url` - The URL to fetch (absolute or relative to page origin)
///
/// # Errors
///
/// - `AssetLoadError::NotFound` for HTTP 404 responses
/// - `AssetLoadError::IoError` for network failures or non-OK status codes
pub async fn web_fetch(url: &str) -> Result<Vec<u8>, AssetLoadError> {
    let window = web_sys::window()
        .ok_or_else(|| AssetLoadError::custom("No global window object available"))?;

    let resp_value = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| AssetLoadError::IoError {
            path: url.to_string(),
            message: format!("Fetch request failed: {:?}", e),
        })?;

    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|_| AssetLoadError::custom("Failed to cast fetch result to Response"))?;

    if !resp.ok() {
        let status = resp.status();
        if status == 404 {
            return Err(AssetLoadError::not_found(url));
        }
        return Err(AssetLoadError::IoError {
            path: url.to_string(),
            message: format!("HTTP {}", status),
        });
    }

    let array_buffer_promise = resp
        .array_buffer()
        .map_err(|_| AssetLoadError::custom("Failed to initiate ArrayBuffer read on Response"))?;

    let array_buffer =
        JsFuture::from(array_buffer_promise)
            .await
            .map_err(|e| AssetLoadError::IoError {
                path: url.to_string(),
                message: format!("Failed to read response body: {:?}", e),
            })?;

    let bytes = js_sys::Uint8Array::new(&array_buffer).to_vec();
    Ok(bytes)
}
