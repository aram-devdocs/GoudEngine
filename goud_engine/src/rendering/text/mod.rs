//! Text rendering infrastructure.
//!
//! This module provides glyph rasterization, atlas generation, and caching
//! for rendering text with TrueType/OpenType fonts.
//!
//! # Architecture
//!
//! 1. **Rasterizer** (`rasterizer`) - wraps `fontdue` to produce per-glyph
//!    grayscale bitmaps with metrics.
//! 2. **Glyph Atlas** (`glyph_atlas`) - packs rasterized glyphs into a single
//!    RGBA8 texture atlas with UV lookup.
//! 3. **Atlas Cache** (`atlas_cache`) - caches generated atlases by
//!    (font handle, pixel size) to avoid repeated rasterization.

pub mod atlas_cache;
pub mod bitmap_atlas;
pub mod glyph_atlas;
pub mod glyph_provider;
pub mod layout;
pub mod rasterizer;
pub mod text_batch;
pub mod text_render_system;

pub use atlas_cache::GlyphAtlasCache;
pub use bitmap_atlas::{BitmapGlyphAtlas, FontAtlas};
pub use glyph_atlas::{GlyphAtlas, GlyphInfo, UvRect};
pub use glyph_provider::GlyphInfoProvider;
pub use layout::{
    layout_text, LayoutGlyph, TextAlignment, TextBoundingBox, TextLayoutConfig, TextLayoutResult,
};
pub use rasterizer::{GlyphMetrics, RasterizedGlyph};
pub use text_batch::{TextBatch, TextDrawBatch, TextRenderStats};
pub use text_render_system::TextRenderSystem;
