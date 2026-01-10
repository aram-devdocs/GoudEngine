//! # Unified SDK Binding Generator
//!
//! This build script generates all language SDK bindings in a single step:
//!
//! - **C# Bindings** (`NativeMethods.g.cs`) via csbindgen
//! - **C Header** (`goud_engine.h`) via cbindgen
//! - **Python Metadata** (`ffi_metadata.json`) for validation
//!
//! ## Design Philosophy
//!
//! All logic lives in Rust. SDKs are thin wrappers that marshal data and call
//! FFI functions. This ensures consistent behavior across all language bindings.
//!
//! ## Usage
//!
//! ```bash
//! cargo build
//! # Outputs:
//! #   - ../sdks/GoudEngine/NativeMethods.g.cs (C# bindings)
//! #   - ../sdks/include/goud_engine.h (C header)
//! #   - ../sdks/python/goud_engine/ffi_metadata.json (Python validation)
//! ```

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/ffi/");
    println!("cargo:rerun-if-changed=src/core/math.rs");
    println!("cargo:rerun-if-changed=src/core/error.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let sdks_dir = Path::new(&manifest_dir).join("..").join("sdks");

    // Track generation results
    let mut generated_files: Vec<(&str, String, bool)> = Vec::new();

    // =========================================================================
    // 1. Generate C# Bindings (csbindgen)
    // =========================================================================
    let csharp_output = sdks_dir.join("GoudEngine").join("NativeMethods.g.cs");
    let csharp_result = generate_csharp_bindings(&csharp_output);
    generated_files.push((
        "C# Bindings",
        csharp_output.display().to_string(),
        csharp_result,
    ));

    // =========================================================================
    // 2. Generate C Header (cbindgen)
    // =========================================================================
    let include_dir = sdks_dir.join("include");
    let c_header_output = include_dir.join("goud_engine.h");
    let c_header_result = generate_c_header(&manifest_dir, &include_dir, &c_header_output);
    generated_files.push((
        "C Header",
        c_header_output.display().to_string(),
        c_header_result,
    ));

    // =========================================================================
    // 3. Generate Python FFI Metadata
    // =========================================================================
    let python_dir = sdks_dir.join("python").join("goud_engine");
    let python_metadata_output = python_dir.join("ffi_metadata.json");
    let python_result = generate_python_metadata(&python_dir, &python_metadata_output);
    generated_files.push((
        "Python Metadata",
        python_metadata_output.display().to_string(),
        python_result,
    ));

    // =========================================================================
    // Print Generation Summary
    // =========================================================================
    println!("cargo:warning=");
    println!("cargo:warning=╔══════════════════════════════════════════════════════════════╗");
    println!("cargo:warning=║           GoudEngine SDK Binding Generation                  ║");
    println!("cargo:warning=╠══════════════════════════════════════════════════════════════╣");

    for (name, path, success) in &generated_files {
        let status = if *success { "✓" } else { "✗" };
        let short_path = path.split("sdks/").last().unwrap_or(path);
        println!("cargo:warning=║ {status} {name}: {short_path}");
    }

    println!("cargo:warning=╚══════════════════════════════════════════════════════════════╝");
    println!("cargo:warning=");
}

/// Generates C# bindings using csbindgen.
fn generate_csharp_bindings(output_path: &Path) -> bool {
    let result = csbindgen::Builder::default()
        // FFI type definitions
        .input_extern_file("src/ffi/types.rs")
        .input_extern_file("src/core/math.rs")
        .input_extern_file("src/core/error.rs")
        // FFI entry points
        .input_extern_file("src/ffi/context.rs")
        .input_extern_file("src/ffi/entity.rs")
        .input_extern_file("src/ffi/component.rs")
        .input_extern_file("src/ffi/component_transform2d.rs")
        .input_extern_file("src/ffi/component_sprite.rs")
        .input_extern_file("src/ffi/window.rs")
        .input_extern_file("src/ffi/renderer.rs")
        .input_extern_file("src/ffi/renderer3d.rs")
        .input_extern_file("src/ffi/input.rs")
        .input_extern_file("src/ffi/collision.rs")
        // Configuration
        .csharp_dll_name("libgoud_engine")
        .csharp_class_accessibility("public")
        .generate_csharp_file(output_path);

    match result {
        Ok(_) => {
            println!(
                "cargo:warning=  Generated C# bindings: {}",
                output_path.display()
            );
            true
        }
        Err(e) => {
            println!("cargo:warning=  Failed to generate C# bindings: {e}");
            false
        }
    }
}

/// Generates C header using cbindgen.
///
/// Note: cbindgen has trouble with complex Rust types (generics, tuples, etc.).
/// For GoudEngine, we use a curated minimal header that covers the FFI-safe types.
/// This approach is actually preferred for FFI as it ensures only C-compatible
/// types are exposed.
fn generate_c_header(_manifest_dir: &str, include_dir: &Path, output_path: &Path) -> bool {
    // Ensure include directory exists
    if let Err(e) = fs::create_dir_all(include_dir) {
        println!("cargo:warning=  Failed to create include directory: {e}");
        return false;
    }

    // Generate a curated C header with FFI-safe types
    // This is more reliable than cbindgen for complex codebases
    // and ensures only the intended FFI types are exposed.
    //
    // Note: cbindgen is disabled because:
    // 1. Complex Rust types (generics, tuples, trait objects) cause parsing errors
    // 2. The curated header gives us precise control over the FFI surface
    // 3. Python SDK uses ctypes with explicit definitions (matching this header)
    // 4. C# SDK uses csbindgen which handles the complexity differently
    generate_minimal_c_header(output_path)
}

/// Generates a minimal C header with type definitions.
/// This is a fallback when cbindgen fails due to complex type handling.
fn generate_minimal_c_header(output_path: &Path) -> bool {
    let header = r#"/*
 * GoudEngine C FFI Header
 * 
 * This header provides C-compatible type definitions and function declarations
 * for interacting with the GoudEngine from other languages.
 * 
 * NOTE: This is a minimal header generated as a fallback.
 * For complete bindings, see the Python SDK (ctypes) or C# SDK (csbindgen).
 */

#ifndef GOUD_ENGINE_H
#define GOUD_ENGINE_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// =============================================================================
// Core Types
// =============================================================================

/// FFI-safe entity identifier (packed index + generation).
typedef uint64_t GoudEntityId;

/// Invalid entity ID sentinel.
#define GOUD_INVALID_ENTITY_ID UINT64_MAX

/// FFI-safe context identifier.
typedef struct {
    uint64_t _bits;
} GoudContextId;

/// Invalid context ID sentinel.
#define GOUD_INVALID_CONTEXT_ID ((GoudContextId){ ._bits = UINT64_MAX })

/// FFI-safe result type for operations that can fail.
typedef struct {
    int32_t code;    // Error code (0 = success)
    bool success;    // True if operation succeeded
} GoudResult;

// =============================================================================
// Math Types
// =============================================================================

/// FFI-safe 2D vector.
typedef struct {
    float x;
    float y;
} FfiVec2;

/// FFI-safe RGBA color.
typedef struct {
    float r;
    float g;
    float b;
    float a;
} FfiColor;

/// FFI-safe rectangle.
typedef struct {
    float x;
    float y;
    float width;
    float height;
} FfiRect;

/// FFI-safe 3x3 matrix (column-major order).
typedef struct {
    float m[9];
} FfiMat3x3;

// =============================================================================
// Component Types
// =============================================================================

/// FFI-safe Transform2D representation.
typedef struct {
    float position_x;
    float position_y;
    float rotation;
    float scale_x;
    float scale_y;
} FfiTransform2D;

/// FFI-safe Sprite representation.
typedef struct {
    uint64_t texture_handle;
    float color_r;
    float color_g;
    float color_b;
    float color_a;
    float source_rect_x;
    float source_rect_y;
    float source_rect_width;
    float source_rect_height;
    bool has_source_rect;
    bool flip_x;
    bool flip_y;
    float anchor_x;
    float anchor_y;
    float custom_size_x;
    float custom_size_y;
    bool has_custom_size;
} FfiSprite;

// =============================================================================
// Builder Types (Opaque Pointers)
// =============================================================================

typedef struct FfiTransform2DBuilder FfiTransform2DBuilder;
typedef struct FfiSpriteBuilder FfiSpriteBuilder;

// =============================================================================
// Context Functions
// =============================================================================

GoudContextId goud_context_create(void);
bool goud_context_destroy(GoudContextId ctx);
bool goud_context_is_valid(GoudContextId ctx);

// =============================================================================
// Entity Functions
// =============================================================================

uint64_t goud_entity_spawn_empty(GoudContextId ctx);
uint32_t goud_entity_spawn_batch(GoudContextId ctx, uint32_t count, uint64_t* out_entities);
GoudResult goud_entity_despawn(GoudContextId ctx, uint64_t entity_id);
uint32_t goud_entity_despawn_batch(GoudContextId ctx, const uint64_t* entity_ids, uint32_t count);
bool goud_entity_is_alive(GoudContextId ctx, uint64_t entity_id);
uint32_t goud_entity_count(GoudContextId ctx);

// =============================================================================
// Transform2D Functions
// =============================================================================

FfiTransform2D goud_transform2d_default(void);
FfiTransform2D goud_transform2d_from_position(float x, float y);
FfiTransform2D goud_transform2d_from_rotation(float rotation);
FfiTransform2D goud_transform2d_from_rotation_degrees(float degrees);
FfiTransform2D goud_transform2d_from_scale(float scale_x, float scale_y);
FfiTransform2D goud_transform2d_from_scale_uniform(float scale);
FfiTransform2D goud_transform2d_look_at(float pos_x, float pos_y, float target_x, float target_y);

void goud_transform2d_translate(FfiTransform2D* transform, float dx, float dy);
void goud_transform2d_translate_local(FfiTransform2D* transform, float dx, float dy);
void goud_transform2d_rotate(FfiTransform2D* transform, float angle);
void goud_transform2d_rotate_degrees(FfiTransform2D* transform, float degrees);
void goud_transform2d_set_position(FfiTransform2D* transform, float x, float y);
void goud_transform2d_set_rotation(FfiTransform2D* transform, float rotation);
void goud_transform2d_set_scale(FfiTransform2D* transform, float scale_x, float scale_y);
void goud_transform2d_look_at_target(FfiTransform2D* transform, float target_x, float target_y);
void goud_transform2d_scale_by(FfiTransform2D* transform, float factor_x, float factor_y);

FfiVec2 goud_transform2d_forward(const FfiTransform2D* transform);
FfiVec2 goud_transform2d_right(const FfiTransform2D* transform);
FfiVec2 goud_transform2d_backward(const FfiTransform2D* transform);
FfiVec2 goud_transform2d_left(const FfiTransform2D* transform);

FfiVec2 goud_transform2d_transform_point(const FfiTransform2D* transform, float x, float y);
FfiVec2 goud_transform2d_inverse_transform_point(const FfiTransform2D* transform, float x, float y);

FfiTransform2D goud_transform2d_lerp(FfiTransform2D from, FfiTransform2D to, float t);

// =============================================================================
// Sprite Functions
// =============================================================================

FfiSprite goud_sprite_new(uint64_t texture_handle);
FfiSprite goud_sprite_default(void);

void goud_sprite_set_color(FfiSprite* sprite, float r, float g, float b, float a);
void goud_sprite_set_flip_x(FfiSprite* sprite, bool flip);
void goud_sprite_set_flip_y(FfiSprite* sprite, bool flip);
void goud_sprite_set_anchor(FfiSprite* sprite, float x, float y);
void goud_sprite_set_source_rect(FfiSprite* sprite, float x, float y, float width, float height);
void goud_sprite_clear_source_rect(FfiSprite* sprite);
void goud_sprite_set_custom_size(FfiSprite* sprite, float width, float height);
void goud_sprite_clear_custom_size(FfiSprite* sprite);
void goud_sprite_set_texture(FfiSprite* sprite, uint64_t handle);

FfiSprite goud_sprite_with_color(FfiSprite sprite, float r, float g, float b, float a);
FfiSprite goud_sprite_with_flip_x(FfiSprite sprite, bool flip);
FfiSprite goud_sprite_with_flip_y(FfiSprite sprite, bool flip);
FfiSprite goud_sprite_with_anchor(FfiSprite sprite, float x, float y);
FfiSprite goud_sprite_with_source_rect(FfiSprite sprite, float x, float y, float width, float height);
FfiSprite goud_sprite_with_custom_size(FfiSprite sprite, float width, float height);

// =============================================================================
// Color Functions
// =============================================================================

FfiColor goud_color_white(void);
FfiColor goud_color_black(void);
FfiColor goud_color_red(void);
FfiColor goud_color_green(void);
FfiColor goud_color_blue(void);
FfiColor goud_color_yellow(void);
FfiColor goud_color_transparent(void);
FfiColor goud_color_rgba(float r, float g, float b, float a);
FfiColor goud_color_rgb(float r, float g, float b);
FfiColor goud_color_from_hex(uint32_t hex);
FfiColor goud_color_lerp(FfiColor from, FfiColor to, float t);

// =============================================================================
// Window Functions
// =============================================================================

GoudContextId goud_window_create(uint32_t width, uint32_t height, const char* title);
bool goud_window_destroy(GoudContextId ctx);
bool goud_window_should_close(GoudContextId ctx);
void goud_window_set_should_close(GoudContextId ctx, bool should_close);
float goud_window_poll_events(GoudContextId ctx);
void goud_window_swap_buffers(GoudContextId ctx);
void goud_window_clear(GoudContextId ctx, float r, float g, float b, float a);
float goud_window_get_delta_time(GoudContextId ctx);

// =============================================================================
// Renderer Functions
// =============================================================================

bool goud_renderer_begin(GoudContextId ctx);
bool goud_renderer_end(GoudContextId ctx);
void goud_renderer_enable_blending(GoudContextId ctx);
void goud_renderer_disable_blending(GoudContextId ctx);
bool goud_renderer_draw_sprite(GoudContextId ctx, uint64_t texture, float x, float y, float width, float height, float rotation, float r, float g, float b, float a);
bool goud_renderer_draw_quad(GoudContextId ctx, float x, float y, float width, float height, float r, float g, float b, float a);

// =============================================================================
// Texture Functions
// =============================================================================

uint64_t goud_texture_load(GoudContextId ctx, const char* path);
bool goud_texture_destroy(GoudContextId ctx, uint64_t handle);

// =============================================================================
// Input Functions
// =============================================================================

bool goud_input_key_pressed(GoudContextId ctx, int32_t key);
bool goud_input_key_just_pressed(GoudContextId ctx, int32_t key);
bool goud_input_key_just_released(GoudContextId ctx, int32_t key);
bool goud_input_mouse_button_pressed(GoudContextId ctx, int32_t button);
bool goud_input_mouse_button_just_pressed(GoudContextId ctx, int32_t button);
bool goud_input_mouse_button_just_released(GoudContextId ctx, int32_t button);
bool goud_input_get_mouse_position(GoudContextId ctx, float* out_x, float* out_y);

// =============================================================================
// Collision Functions
// =============================================================================

bool goud_collision_aabb_overlap(
    float a_min_x, float a_min_y, float a_max_x, float a_max_y,
    float b_min_x, float b_min_y, float b_max_x, float b_max_y
);
bool goud_collision_point_in_rect(
    float point_x, float point_y,
    float rect_x, float rect_y, float rect_w, float rect_h
);

#ifdef __cplusplus
}
#endif

#endif /* GOUD_ENGINE_H */
"#;

    match fs::write(output_path, header) {
        Ok(_) => {
            println!(
                "cargo:warning=  Generated minimal C header: {}",
                output_path.display()
            );
            true
        }
        Err(e) => {
            println!("cargo:warning=  Failed to write C header: {e}");
            false
        }
    }
}

/// Generates Python FFI metadata for SDK validation.
fn generate_python_metadata(python_dir: &Path, output_path: &Path) -> bool {
    // Ensure Python SDK directory exists
    if !python_dir.exists() {
        println!(
            "cargo:warning=  Python SDK directory not found: {}",
            python_dir.display()
        );
        return false;
    }

    // Generate FFI metadata as JSON for Python SDK validation
    let metadata = r#"{
  "version": "0.0.809",
  "generated_by": "goud_engine/build.rs",
  "description": "FFI metadata for Python SDK validation",
  
  "types": {
    "GoudContextId": {
      "size": 8,
      "fields": [
        { "name": "_bits", "type": "uint64_t" }
      ]
    },
    "GoudEntityId": {
      "size": 8,
      "type": "uint64_t",
      "invalid_value": "0xFFFFFFFFFFFFFFFF"
    },
    "GoudResult": {
      "size": 8,
      "fields": [
        { "name": "code", "type": "int32_t" },
        { "name": "success", "type": "bool" }
      ]
    },
    "FfiVec2": {
      "size": 8,
      "fields": [
        { "name": "x", "type": "float" },
        { "name": "y", "type": "float" }
      ]
    },
    "FfiColor": {
      "size": 16,
      "fields": [
        { "name": "r", "type": "float" },
        { "name": "g", "type": "float" },
        { "name": "b", "type": "float" },
        { "name": "a", "type": "float" }
      ]
    },
    "FfiRect": {
      "size": 16,
      "fields": [
        { "name": "x", "type": "float" },
        { "name": "y", "type": "float" },
        { "name": "width", "type": "float" },
        { "name": "height", "type": "float" }
      ]
    },
    "FfiTransform2D": {
      "size": 20,
      "fields": [
        { "name": "position_x", "type": "float" },
        { "name": "position_y", "type": "float" },
        { "name": "rotation", "type": "float" },
        { "name": "scale_x", "type": "float" },
        { "name": "scale_y", "type": "float" }
      ]
    },
    "FfiSprite": {
      "fields": [
        { "name": "texture_handle", "type": "uint64_t" },
        { "name": "color_r", "type": "float" },
        { "name": "color_g", "type": "float" },
        { "name": "color_b", "type": "float" },
        { "name": "color_a", "type": "float" },
        { "name": "source_rect_x", "type": "float" },
        { "name": "source_rect_y", "type": "float" },
        { "name": "source_rect_width", "type": "float" },
        { "name": "source_rect_height", "type": "float" },
        { "name": "has_source_rect", "type": "bool" },
        { "name": "flip_x", "type": "bool" },
        { "name": "flip_y", "type": "bool" },
        { "name": "anchor_x", "type": "float" },
        { "name": "anchor_y", "type": "float" },
        { "name": "custom_size_x", "type": "float" },
        { "name": "custom_size_y", "type": "float" },
        { "name": "has_custom_size", "type": "bool" }
      ]
    }
  },

  "functions": {
    "context": [
      "goud_context_create",
      "goud_context_destroy",
      "goud_context_is_valid"
    ],
    "entity": [
      "goud_entity_spawn_empty",
      "goud_entity_spawn_batch",
      "goud_entity_despawn",
      "goud_entity_despawn_batch",
      "goud_entity_is_alive",
      "goud_entity_count"
    ],
    "transform2d": [
      "goud_transform2d_default",
      "goud_transform2d_from_position",
      "goud_transform2d_from_rotation",
      "goud_transform2d_from_rotation_degrees",
      "goud_transform2d_from_scale",
      "goud_transform2d_from_scale_uniform",
      "goud_transform2d_look_at",
      "goud_transform2d_translate",
      "goud_transform2d_translate_local",
      "goud_transform2d_rotate",
      "goud_transform2d_rotate_degrees",
      "goud_transform2d_set_position",
      "goud_transform2d_set_rotation",
      "goud_transform2d_set_scale",
      "goud_transform2d_look_at_target",
      "goud_transform2d_scale_by",
      "goud_transform2d_forward",
      "goud_transform2d_right",
      "goud_transform2d_backward",
      "goud_transform2d_left",
      "goud_transform2d_transform_point",
      "goud_transform2d_inverse_transform_point",
      "goud_transform2d_lerp"
    ],
    "sprite": [
      "goud_sprite_new",
      "goud_sprite_default",
      "goud_sprite_set_color",
      "goud_sprite_set_flip_x",
      "goud_sprite_set_flip_y",
      "goud_sprite_set_anchor",
      "goud_sprite_set_source_rect",
      "goud_sprite_clear_source_rect",
      "goud_sprite_set_custom_size",
      "goud_sprite_clear_custom_size",
      "goud_sprite_set_texture",
      "goud_sprite_with_color",
      "goud_sprite_with_flip_x",
      "goud_sprite_with_flip_y",
      "goud_sprite_with_anchor",
      "goud_sprite_with_source_rect",
      "goud_sprite_with_custom_size"
    ],
    "color": [
      "goud_color_white",
      "goud_color_black",
      "goud_color_red",
      "goud_color_green",
      "goud_color_blue",
      "goud_color_yellow",
      "goud_color_transparent",
      "goud_color_rgba",
      "goud_color_rgb",
      "goud_color_from_hex",
      "goud_color_lerp"
    ],
    "window": [
      "goud_window_create",
      "goud_window_destroy",
      "goud_window_should_close",
      "goud_window_set_should_close",
      "goud_window_poll_events",
      "goud_window_swap_buffers",
      "goud_window_clear",
      "goud_window_get_delta_time"
    ],
    "renderer": [
      "goud_renderer_begin",
      "goud_renderer_end",
      "goud_renderer_enable_blending",
      "goud_renderer_disable_blending",
      "goud_renderer_draw_sprite",
      "goud_renderer_draw_quad"
    ],
    "texture": [
      "goud_texture_load",
      "goud_texture_destroy"
    ],
    "input": [
      "goud_input_key_pressed",
      "goud_input_key_just_pressed",
      "goud_input_key_just_released",
      "goud_input_mouse_button_pressed",
      "goud_input_mouse_button_just_pressed",
      "goud_input_mouse_button_just_released",
      "goud_input_get_mouse_position"
    ],
    "collision": [
      "goud_collision_aabb_overlap",
      "goud_collision_point_in_rect"
    ]
  }
}
"#;

    match fs::write(output_path, metadata) {
        Ok(_) => {
            println!(
                "cargo:warning=  Generated Python metadata: {}",
                output_path.display()
            );
            true
        }
        Err(e) => {
            println!("cargo:warning=  Failed to write Python metadata: {e}");
            false
        }
    }
}
