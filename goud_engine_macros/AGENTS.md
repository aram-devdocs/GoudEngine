# goud_engine_macros/ — Proc Macro Crate

Auto-generates FFI wrapper functions from `#[goud_api]` annotations. Produces JSON manifest for SDK codegen.

## Usage

```rust
#[goud_api(module = "window")]
impl GoudGame {
    pub fn should_close(&self) -> bool { ... }
}
```

Generates: `goud_window_should_close(ctx: GoudContextId) -> bool`

## Key Files

- `lib.rs` — Macro entry, parses attributes, dispatches to codegen
- `ffi_gen.rs` — Generates `extern "C"` wrappers
- `type_mapping.rs` — Rust → FFI type conversion, flattening
- `manifest.rs` — JSON metadata structs
- `codegen_helpers.rs` — Shared codegen utilities

## Attributes

| Attr | Effect |
|------|--------|
| `module = "name"` | Required. Sets FFI prefix (`goud_<module>_<method>`) |
| `feature = "native"` | Optional. Wraps in `#[cfg(feature = "...")]` |
| `#[goud_api(skip)]` | Exclude from FFI |
| `#[goud_api(name = "...")]` | Custom FFI name |

## Build Integration

1. Macro emits manifest consts per module
2. `build.rs` collects consts → `codegen/ffi_manifest.json`
3. Codegen scripts read manifest → generate SDK wrappers

## Gotchas

- Private methods and `skip` are silently excluded
- Generic/closure methods cannot auto-wrap — use `skip`
- Changing type mappings requires re-running all SDK codegen
