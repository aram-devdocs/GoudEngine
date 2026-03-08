# Text Rendering

GoudEngine renders text using TrueType fonts with glyph atlas caching for performance.

## Text Component

Attach `Text` to an entity to render text.

| Field | Type | Default | Description |
|---|---|---|---|
| `content` | `String` | `""` | Text to display |
| `font_handle` | `AssetHandle<FontAsset>` | — | TrueType font asset |
| `bitmap_font_handle` | `Option<AssetHandle<BitmapFontAsset>>` | None | Bitmap font (.fnt) |
| `font_size` | `f32` | 16.0 | Font size in pixels |
| `color` | `Color` | white | Text color (RGBA) |
| `alignment` | `TextAlignment` | Left | Horizontal alignment |
| `max_width` | `Option<f32>` | None | Word-wrap width |
| `line_spacing` | `f32` | 1.0 | Line height multiplier |

## Font Types

Two font formats are supported:

- **TrueType** (TTF/OTF) — vector fonts loaded via `FontAsset`, rasterized into a glyph atlas at the requested size
- **Bitmap** (.fnt) — pre-rendered sprite fonts loaded via `BitmapFontAsset`

## Text Alignment

`TextAlignment` controls horizontal positioning:

- `Left` — left edge aligned to entity position
- `Center` — centered on entity position
- `Right` — right edge aligned to entity position

## Word Wrapping

Set `max_width` to enable word-wrapping. Text breaks at word boundaries when a line exceeds the specified width. The `line_spacing` multiplier controls vertical distance between lines.

## Glyph Atlas

TrueType fonts are rasterized into a glyph atlas texture on first use. The atlas caches rendered glyphs to avoid per-frame rasterization. Different font sizes produce separate atlas entries.
