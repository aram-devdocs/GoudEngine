# GoudEngine Proc Macros

Procedural macro crate providing the `#[goud_api]` attribute for auto-generating
FFI wrapper functions from annotated `impl` blocks.

## Purpose

Eliminates hand-written `#[no_mangle] extern "C"` boilerplate. Annotate an impl
block and the macro generates the FFI wrappers, null checks, context lookups,
and a JSON manifest consumed by the codegen pipeline.

## Usage

```rust
#[goud_api(module = "window")]
impl GoudGame {
    pub fn should_close(&self) -> bool { /* ... */ }
    pub fn poll_events(&mut self) -> GoudResult<f32> { /* ... */ }
}
```

This generates:
- `goud_window_should_close(ctx: GoudContextId) -> bool`
- `goud_window_poll_events(ctx: GoudContextId, out: *mut f32) -> GoudResult`
- A JSON manifest entry for each function (collected into `codegen/ffi_manifest.json`)

## Attributes

### Block-level (on `impl`)

| Attribute | Required | Effect |
|-----------|----------|--------|
| `module = "name"` | Yes | Sets FFI function prefix: `goud_<module>_<method>` |
| `feature = "native"` | No | Wraps generated FFI in `#[cfg(feature = "...")]` |

### Method-level (on `fn`)

| Attribute | Effect |
|-----------|--------|
| `#[goud_api(skip)]` | Excludes method from FFI generation |
| `#[goud_api(name = "custom")]` | Overrides the method name in the FFI function |

## Build Integration

1. The macro emits manifest metadata as `const` strings per module
2. `build.rs` in `goud_engine` collects these manifest constants
3. Combined output is written to `codegen/ffi_manifest.json`
4. Codegen scripts (`gen_csharp.py`, `gen_python.py`, etc.) read the manifest

## Crate Type

This is a `proc-macro = true` crate. It depends on `syn`, `quote`, `proc_macro2`,
`serde`, and `serde_json`. It has no runtime dependency on the engine.
