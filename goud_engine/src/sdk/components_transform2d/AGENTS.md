# components_transform2d/ -- SDK Transform2D Component Operations

## Purpose

FFI-ready Transform2D operations: factory constructors, a heap-allocated builder,
and pointer-based mutation/query functions. All methods use `#[goud_api]` for
automatic C FFI wrapper generation.

## Files

- `mod.rs` -- Module declaration, re-exports submodules
- `factory.rs` -- `Transform2DOps`: value constructors (`new_default`, `from_position`, `from_rotation`, `lerp`, `look_at`, `normalize_angle`)
- `builder.rs` -- `Transform2DBuilderOps`: heap-allocated builder (`builder_new` -> chain -> `builder_build`)
- `ptr_ops.rs` -- `Transform2DPtrOps`: get/set/mutate on `*mut FfiTransform2D` pointers

## Key Types

- `FfiTransform2D` -- `#[repr(C)]` struct with position (x,y), rotation (radians), scale (x,y)
- `FfiTransform2DBuilder` -- wrapper holding an `FfiTransform2D` for builder pattern
- `FfiVec2`, `FfiMat3x3` -- C-compatible return types for getters

## ptr_ops Highlights

Beyond basic get/set, `Transform2DPtrOps` provides:
- `translate` / `translate_local` -- world-space and local-space movement
- `rotate` / `rotate_degrees` -- incremental rotation
- `look_at_target` -- orient toward a point
- `forward` / `right` / `backward` / `left` -- direction vectors
- `matrix` / `matrix_inverse` -- 3x3 transformation matrices
- `transform_point` / `inverse_transform_point` -- space conversions

These convert between `FfiTransform2D` and the internal `Transform2D` ECS
component to reuse core math, then write back to the FFI struct.

## Patterns

- Factory functions return `FfiTransform2D` by value (no allocation)
- Builder uses `Box::into_raw` / `Box::from_raw` for heap ownership across FFI
- All pointer ops check null and return identity/zero defaults on null input
- Rotation stored in radians internally; degree variants do conversion

## Anti-Patterns

- NEVER skip null checks on pointer parameters
- NEVER implement transform math here -- delegate to `Transform2D` from `ecs::components`
- NEVER forget to pair `builder_new` with `builder_build` or `builder_free`

## Dependencies

Layer 2 (Engine/SDK). Imports from `crate::core::math`, `crate::core::types`,
and `crate::ecs::components::Transform2D`.
