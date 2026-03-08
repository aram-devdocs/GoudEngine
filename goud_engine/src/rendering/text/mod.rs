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
pub mod glyph_atlas;
pub mod layout;
pub mod rasterizer;

pub use atlas_cache::GlyphAtlasCache;
pub use glyph_atlas::{GlyphAtlas, GlyphInfo, UvRect};
pub use layout::{
    layout_text, LayoutGlyph, TextAlignment, TextBoundingBox, TextLayoutConfig, TextLayoutResult,
};
pub use rasterizer::{GlyphMetrics, RasterizedGlyph};
