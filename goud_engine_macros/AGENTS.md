# goud_engine_macros/ — Proc Macro Crate

## Purpose

Provides the `#[goud_api]` attribute macro that auto-generates FFI wrapper functions
from annotated `impl` blocks. Eliminates hand-written `#[no_mangle] extern "C"` boilerplate
and produces a JSON manifest consumed by the codegen pipeline.

## How It Works

Annotate an impl block:

```rust
#[goud_api(module = "window")]
impl GoudGame {
    pub fn should_close(&self) -> bool { ... }
    pub fn poll_events(&mut self) -> GoudResult<f32> { ... }
}
```

The macro generates:
- `goud_window_should_close(ctx: GoudContextId) -> bool`
- `goud_window_poll_events(ctx: GoudContextId, out: *mut f32) -> GoudResult`
- A hidden `const` containing JSON manifest metadata for each function

## Files

- `lib.rs` — Macro entry point. Parses `#[goud_api(...)]` attributes, dispatches to
  `ffi_gen`, emits the cleaned impl block + generated wrappers + manifest const.
- `ffi_gen.rs` — Generates the actual `extern "C"` wrapper functions. Handles context
  lookup, parameter conversion, return value marshalling, null checks, and SAFETY comments.
- `type_mapping.rs` — Maps Rust types to FFI-compatible types. Handles flattening
  (e.g., `Vec2` becomes two `f32` params), pointer-based returns, and `GoudResult` wrapping.
- `manifest.rs` — Serializable structs (`SdkManifest`, `SdkModule`, `SdkMethod`) that
  capture method metadata as JSON. Emitted as const strings, collected by `build.rs` into
  `codegen/ffi_manifest.json`.
- `codegen_helpers.rs` — Shared helpers for parameter extraction, receiver detection,
  and return handling code generation.

## Attributes

| Level | Attribute | Effect |
|-------|-----------|--------|
| Block | `module = "name"` | Required. Sets FFI function name prefix (`goud_<module>_<method>`) |
| Block | `feature = "native"` | Optional. Wraps generated FFI in `#[cfg(feature = "...")]` |
| Method | `#[goud_api(skip)]` | Excludes method from FFI generation |
| Method | `#[goud_api(name = "custom")]` | Overrides the method name portion of the FFI function |

## Build Integration

1. `#[goud_api]` emits `const __GOUD_MANIFEST_<MODULE>: &str = "<json>";` per module
2. `build.rs` in `goud_engine` collects these manifest consts
3. Combined output written to `codegen/ffi_manifest.json`
4. Codegen scripts (`gen_csharp.py`, etc.) read the manifest to generate SDK wrappers

## Dependencies

This is a proc-macro crate (`proc-macro = true`). It depends on `syn`, `quote`,
`proc_macro2`, `serde`, and `serde_json`. It has no runtime dependency on the engine.

## Gotchas

- Private methods and methods marked `#[goud_api(skip)]` are silently excluded
- Generic methods, closures, and builder patterns cannot be auto-wrapped -- use `skip`
- The generated FFI module is `__goud_generated_ffi` (hidden, doc-hidden)
- Changing type mappings here requires re-running codegen for all SDKs
