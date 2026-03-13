use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// Transform2D data transfer object
// ---------------------------------------------------------------------------

/// A plain-data snapshot of a `Transform2D` component, safe to pass across
/// the wasm-bindgen boundary.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct WasmTransform2D {
    /// X position.
    pub position_x: f32,
    /// Y position.
    pub position_y: f32,
    /// Rotation in radians.
    pub rotation: f32,
    /// X scale.
    pub scale_x: f32,
    /// Y scale.
    pub scale_y: f32,
}

// ---------------------------------------------------------------------------
// Sprite data transfer object
// ---------------------------------------------------------------------------

/// A plain-data snapshot of a `Sprite` component, safe to pass across
/// the wasm-bindgen boundary.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct WasmSprite {
    /// Texture handle backing this sprite.
    pub texture_handle: u32,
    /// Red channel tint.
    pub r: f32,
    /// Green channel tint.
    pub g: f32,
    /// Blue channel tint.
    pub b: f32,
    /// Alpha channel tint.
    pub a: f32,
    /// Horizontal texture flip flag.
    pub flip_x: bool,
    /// Vertical texture flip flag.
    pub flip_y: bool,
    /// X anchor in normalized sprite space.
    pub anchor_x: f32,
    /// Y anchor in normalized sprite space.
    pub anchor_y: f32,
}

// ---------------------------------------------------------------------------
// Render statistics data transfer object
// ---------------------------------------------------------------------------

/// Per-frame rendering statistics exposed to JavaScript.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct WasmRenderStats {
    /// Number of draw calls submitted this frame.
    pub draw_calls: u32,
    /// Number of triangles submitted this frame.
    pub triangles: u32,
    /// Number of texture bind switches this frame.
    pub texture_binds: u32,
}
