# components_transform2d/ — Transform2D FFI Operations

Factory constructors, heap-allocated builder, and pointer-based mutation/query. All methods tagged with `#[goud_api]` for automatic FFI wrapper generation.

## Key Types

- `FfiTransform2D` — C-compatible struct: position, rotation (radians), scale
- `FfiTransform2DBuilder` — heap-allocated builder pattern
- `FfiVec2`, `FfiMat3x3` — return types for getters

## Modules

- `factory.rs` — Constructors: `new_default`, `from_position`, `lerp`, `normalize_angle`
- `builder.rs` — Builder pattern: `builder_new` → chain → `builder_build`
- `ptr_ops.rs` — Pointer operations: translate, rotate, look_at, matrix math

## Patterns

- Factory functions return by value (no allocation)
- Builder uses `Box::into_raw`/`Box::from_raw` for FFI ownership
- All pointer ops null-check and return identity defaults
- Rotation in radians; degree variants convert

## Anti-Patterns

- Never skip null checks on pointers
- Never implement transform math here — delegate to `Transform2D`
- Never forget to pair `builder_new` with `builder_build` or `builder_free`
