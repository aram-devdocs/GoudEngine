# Texture Atlas

Runtime bin-packing of multiple textures into a single GPU texture. Combining many small textures into one atlas reduces texture bind calls during rendering, which is one of the most common GPU bottlenecks.

## When to Use

- Sprite sheets assembled at runtime from individual images
- UI elements packed into a single texture for efficient rendering
- Tile sets loaded from separate files but rendered as a batch
- Any scenario where many small textures cause excessive GPU state changes

## Lifecycle

### 1. Create an Atlas

```
goud_atlas_create(context_id, category, max_width, max_height) -> GoudAtlasHandle
```

| Parameter | Description |
|---|---|
| `context_id` | Engine context handle |
| `category` | Null-terminated C string naming the atlas (e.g., "sprites", "ui") |
| `max_width` | Maximum atlas width in pixels (0 = default 2048, max 8192) |
| `max_height` | Maximum atlas height in pixels (0 = default 2048, max 8192) |

Returns a valid atlas handle, or `GOUD_INVALID_ATLAS` on failure.

### 2. Add Textures

Two methods for adding image data:

| FFI Function | Description |
|---|---|
| `goud_atlas_add_from_file(ctx, atlas, key, path)` | Load an image file and pack it |
| `goud_atlas_add_pixels(ctx, atlas, key, pixels, width, height)` | Pack raw RGBA8 pixel data |

Each entry is identified by a string `key` used for later lookup. The atlas performs bin-packing to place each image in an available region.

`goud_atlas_add_texture` is reserved and currently returns an error (GPU pixel readback is not supported).

### 3. Finalize (Upload to GPU)

```
goud_atlas_finalize(context_id, atlas) -> GoudTextureHandle
```

Uploads the packed atlas to the GPU and returns a texture handle. After finalization, the atlas can be used for rendering. You must finalize before drawing.

### 4. Query Entries

```
goud_atlas_get_entry(context_id, atlas, key, out_entry) -> bool
```

Writes the UV coordinates and pixel placement for a packed texture into an `FfiAtlasEntry`:

| Field | Type | Description |
|---|---|---|
| `u_min`, `v_min` | `f32` | Top-left UV coordinates |
| `u_max`, `v_max` | `f32` | Bottom-right UV coordinates |
| `pixel_x`, `pixel_y` | `u32` | Pixel offset within the atlas |
| `pixel_w`, `pixel_h` | `u32` | Pixel dimensions of the entry |

Use the UV coordinates when rendering sprites from the atlas texture.

### 5. Query Stats

```
goud_atlas_get_stats(context_id, atlas, out_stats) -> bool
```

Returns packing statistics via `FfiAtlasStats`:

| Field | Type | Description |
|---|---|---|
| `texture_count` | `u32` | Number of packed textures |
| `width`, `height` | `u32` | Atlas dimensions |
| `used_pixels` | `u32` | Pixels occupied by packed textures |
| `total_pixels` | `u32` | Total atlas area |
| `efficiency` | `f32` | Packing efficiency (0.0 to 1.0) |
| `wasted_pixels` | `u32` | Unused pixels |

### 6. Get the GPU Texture

```
goud_atlas_get_texture(context_id, atlas) -> GoudTextureHandle
```

Returns the GPU texture handle after finalization. Returns `GOUD_INVALID_TEXTURE` if the atlas has not been finalized.

### 7. Destroy

```
goud_atlas_destroy(context_id, atlas) -> bool
```

Frees both CPU and GPU resources for the atlas.

## Error Handling

All functions set the last error on failure. Common conditions:

- Invalid context or atlas handle
- Null pointer for string parameters
- Atlas dimensions exceeding 8192
- Entry key not found
- Atlas not finalized when querying GPU texture

Call `goud_last_error_message()` for details.

## FFI

Atlas FFI functions are in `goud_engine/src/ffi/renderer/atlas/`. The `#[repr(C)]` structs `FfiAtlasEntry` and `FfiAtlasStats` are defined in the atlas module.
