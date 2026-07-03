# ffi/ — Foreign Function Interface Layer

## Purpose

C-compatible FFI exports consumed by the C# and Python SDKs. csbindgen auto-generates
C# bindings from these exports on `cargo build`.

## Layout

Most domains are directories (a `mod.rs` plus focused submodules), not flat `.rs` files. Run `ls` here for the current set. Key entries:

- `context/`, `entity/` — engine context and entity lifecycle FFI
- `renderer/`, `renderer3d/` — 2D and 3D rendering FFI
- `component/`, `component_transform2d/`, `component_sprite/`, `component_sprite_animator/`, `component_text/` — component FFI
- `input/`, `window/`, `physics/`, `animation/`, `audio/`, `network/`, `ui/` — subsystem FFI
- `collision.rs`, `types.rs` — flat modules; `types.rs` holds `#[repr(C)]` types shared across FFI
- `mod.rs` — FFI module registration

## Rules (MANDATORY)

- All public functions MUST be `#[no_mangle] extern "C"`
- Structs shared across FFI MUST be `#[repr(C)]`
- Every pointer parameter MUST have a null check
- Every `unsafe` block MUST have a `// SAFETY:` comment
- Error codes returned as `i32` (0 = success, negative = error)
- Memory ownership: document who allocates and who frees

## After Changes

1. Run `cargo build` to trigger csbindgen (updates C# `NativeMethods.g.cs`)
2. Run `./codegen.sh` to regenerate all SDK bindings — never hand-edit generated files such as `sdks/python/goudengine/generated/_ffi.py`
3. Update the hand-written SDK wrappers that call the regenerated bindings

## Dependencies

Layer 5 (FFI). May import from core/, ecs/, assets/, libs/, and Layer 4 engine modules. This is the outermost boundary; no engine module imports from here.
