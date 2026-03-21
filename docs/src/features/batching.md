# Batching

SpriteBatch and TextBatch reduce GPU overhead by combining many draw calls into a single batched pass. Instead of issuing one draw call per sprite or text label, the batch renderer groups commands by texture and draws all sprites sharing a texture in one GPU call.

## Why Batching Matters

Each individual draw call has CPU overhead: binding textures, uploading vertex data, and issuing GPU commands. In a typical 2D game with hundreds of sprites, unbatched rendering can become CPU-bound from draw call overhead alone.

Batching solves this by:

1. Sorting sprites by z-layer then by texture
2. Building a single vertex buffer for all sprites
3. Issuing one draw call per texture group

A scene with 500 sprites using 10 textures drops from 500 draw calls to 10.

## SpriteBatch

### FfiSpriteCmd

Each sprite in the batch is described by an `FfiSpriteCmd` struct (`#[repr(C)]`, 72 bytes):

| Field | Type | Description |
|---|---|---|
| `texture` | `u64` | Texture handle from `goud_texture_load` |
| `x`, `y` | `f32` | Position in screen-space pixels |
| `width`, `height` | `f32` | Sprite dimensions on screen |
| `rotation` | `f32` | Rotation in radians |
| `src_x`, `src_y` | `f32` | Source rectangle offset in pixel coordinates |
| `src_w`, `src_h` | `f32` | Source rectangle size (0,0 = full texture) |
| `r`, `g`, `b`, `a` | `f32` | Color tint and opacity |
| `z_layer` | `i32` | Depth sorting (lower values drawn first) |

### Drawing

```
goud_renderer_draw_sprite_batch(context_id, cmds, count) -> u32
```

| Parameter | Description |
|---|---|
| `context_id` | Engine context handle |
| `cmds` | Pointer to an array of `FfiSpriteCmd` |
| `count` | Number of commands in the array |

Returns the number of sprites drawn (0 on error).

The renderer:

1. Sorts commands by `z_layer`, then by `texture`
2. Builds rotated quad vertices with UV mapping
3. Groups consecutive sprites with the same texture into GPU batches
4. Draws each batch with a single indexed draw call

Source-rect fields (`src_x`, `src_y`, `src_w`, `src_h`) are in pixel coordinates. The renderer converts them to UV coordinates automatically. When `src_w` and `src_h` are both 0, the full texture is used.

## TextBatch

### FfiTextCmd

Each text label is described by an `FfiTextCmd` struct (`#[repr(C)]`, 56 bytes):

| Field | Type | Description |
|---|---|---|
| `font_handle` | `u64` | Font handle from `goud_font_load` |
| `text` | `*const c_char` | Null-terminated UTF-8 string |
| `x`, `y` | `f32` | Position in screen-space pixels |
| `font_size` | `f32` | Font size in pixels |
| `alignment` | `u8` | 0=Left, 1=Center, 2=Right |
| `direction` | `u8` | 0=Auto, 1=LTR, 2=RTL |
| `max_width` | `f32` | Maximum line width (0 = no wrap) |
| `line_spacing` | `f32` | Line spacing multiplier (default 1.0) |
| `r`, `g`, `b`, `a` | `f32` | Text color and opacity |

### Drawing

```
goud_renderer_draw_text_batch(context_id, cmds, count) -> u32
```

Returns the number of text labels drawn (0 on error).

Each command reuses the glyph atlas cached by `(font_handle, font_size)`, so repeated labels with the same font and size avoid redundant atlas rebuilds. Commands with null text pointers, empty strings, or invalid font sizes are silently skipped.

## Performance Expectations

| Scenario | Without Batching | With Batching |
|---|---|---|
| 100 sprites, 5 textures | 100 draw calls | 5 draw calls |
| 500 sprites, 10 textures | 500 draw calls | 10 draw calls |
| 1000 sprites, 1 texture | 1000 draw calls | 1 draw call |

GPU buffer management is handled automatically:

- Vertex and index buffers are created lazily on first use
- Buffers grow dynamically when the sprite count exceeds current capacity
- Old buffers are destroyed only after the replacement is allocated

## Integration with Debugger

Both batch renderers report statistics to the runtime debugger:

- Sprites/text labels drawn
- Triangle count
- Number of GPU batches
- Draw calls issued

These metrics appear in the debugger overlay when the runtime debugger is active.

## FFI

Sprite batch FFI is in `goud_engine/src/ffi/renderer/draw/batch.rs`.
Text batch FFI is in `goud_engine/src/ffi/renderer/text/batch.rs`.
