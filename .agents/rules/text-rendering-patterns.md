---
globs:
  - "**/text/**"
  - "**/ffi/component_text/**"
  - "**/rendering/text/**"
---

# Text Rendering Patterns

## Architecture

- `Text` component attached to entities for text display
- TrueType fonts loaded as `FontAsset`; bitmap fonts as `BitmapFontAsset`
- Glyph atlas caching: glyphs rasterized once per font-size combination
- Rendering handled by `text_render_system.rs` in `rendering/text/`, not by the Text component itself

## Text Component

- `content` — the text string to display
- `font_size` — size in pixels (default 16.0)
- `color` — RGBA color
- `alignment` — Left, Center, or Right
- `max_width` — enables word-wrapping when set
- `line_spacing` — multiplier for vertical line distance
- Source in `ecs/components/text/`

## Font Types

- TrueType (TTF/OTF) — vector fonts, rasterized into atlas at requested size via `rasterizer.rs`
- Bitmap (`.fnt`) — pre-rendered sprite fonts, loaded from descriptor + atlas image via `bitmap_atlas.rs`

## Glyph Pipeline

- `GlyphProvider` trait abstracts TrueType vs bitmap sources
- `atlas_cache.rs` manages per-font-size atlas texture allocation
- `text_batch.rs` batches glyph quads into a single draw call per frame

## FFI

- Text FFI in `ffi/component_text/` — factory, properties, and mod modules
- Text is managed through the ECS component system, not standalone FFI functions

## Testing

- Font loading tests may require asset files in `assets/`
- Layout math (alignment, wrapping) can be tested without GL context
- Glyph atlas tests may need GL context for texture creation
- Test files in `rendering/text/text_batch_tests.rs`
