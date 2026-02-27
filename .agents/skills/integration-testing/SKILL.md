---
name: integration-testing
description: Integration test patterns for Rust game engine with GL context and FFI boundaries
user-invocable: true
---

# Integration Testing

Patterns and conventions for writing integration tests in GoudEngine, covering cross-module interactions, GL context management, and FFI boundary testing.

## When to Use

Use when writing tests that exercise multiple modules together, test FFI boundaries end-to-end, or validate SDK wrappers against the Rust core.

## Test Organization

```
goud_engine/
├── src/
│   ├── ecs/
│   │   └── mod.rs          # Unit tests in #[cfg(test)] module
│   ├── ffi/
│   │   └── mod.rs          # Unit tests for FFI functions
│   └── libs/graphics/
│       └── mod.rs          # Unit tests (may need GL context)
├── tests/                   # Integration tests
│   ├── ecs_integration.rs
│   ├── ffi_integration.rs
│   └── graphics_integration.rs
└── benches/                 # Benchmarks (criterion)
    └── ecs_bench.rs

sdks/
├── GoudEngine.Tests/        # C# SDK tests (xUnit)
└── python/
    └── test_bindings.py     # Python SDK tests
```

## GL Context Management

Many graphics tests require an OpenGL context. Use the helper:

```rust
use crate::test_helpers::init_test_context;

#[test]
fn test_renderer_initialization() {
    let _ctx = init_test_context();
    // GL-dependent test code here
}
```

**Rules**:
- Tests that need GL MUST call `init_test_context()` at the start
- Tests that do NOT need GL (math, ECS logic, data structures) MUST NOT call it
- GL tests may fail in CI environments without a display — mark test expectations accordingly

## Test Factory Patterns

Create reusable factories for common test objects:

```rust
#[cfg(test)]
mod test_helpers {
    use super::*;

    pub fn create_test_entity(world: &mut World) -> Entity {
        let entity = world.spawn();
        world.add_component(entity, Transform2D::default());
        entity
    }

    pub fn create_test_sprite(world: &mut World, entity: Entity) {
        world.add_component(entity, Sprite::new("test_texture"));
    }
}
```

## FFI Integration Testing

Test the full path: Rust → FFI → SDK wrapper.

```rust
#[test]
fn test_ffi_create_entity_roundtrip() {
    // 1. Create context via FFI
    let ctx = unsafe { ffi::create_context() };
    assert!(!ctx.is_null());

    // 2. Create entity via FFI
    let entity_id = unsafe { ffi::create_entity(ctx) };
    assert!(entity_id > 0);

    // 3. Clean up
    unsafe { ffi::destroy_context(ctx) };
}
```

**FFI test rules**:
- Always test null pointer handling (pass null, expect error code)
- Test memory lifecycle (create → use → destroy)
- Verify error codes match expected values
- Test string marshaling (CStr roundtrips)

## Cross-SDK Parity Tests

Verify the same operation produces the same result in both SDKs:

1. Write the test in Rust (ground truth)
2. Write equivalent test in Python (`test_bindings.py`)
3. Write equivalent test in C# (`GoudEngine.Tests/`)
4. Results must match across all three

## Test Categories

| Category | Location | Needs GL | Run Command |
|----------|----------|----------|-------------|
| Unit (Rust) | `#[cfg(test)]` in source | Depends | `cargo test` |
| Integration (Rust) | `goud_engine/tests/` | Often | `cargo test --test <name>` |
| FFI boundary | `goud_engine/tests/` | Sometimes | `cargo test --test ffi_*` |
| Python SDK | `sdks/python/test_bindings.py` | No | `python3 sdks/python/test_bindings.py` |
| C# SDK | `sdks/GoudEngine.Tests/` | No | `dotnet test sdks/GoudEngine.Tests/` |
| Benchmarks | `goud_engine/benches/` | Sometimes | `cargo bench` |

## Checklist

Before submitting integration tests:

- [ ] Tests are in the correct location (unit vs integration)
- [ ] GL-dependent tests use `init_test_context()`
- [ ] Non-GL tests verified to run without display
- [ ] FFI tests check null pointer cases
- [ ] FFI tests verify memory cleanup
- [ ] Test names describe the scenario being tested
- [ ] No `#[ignore]` or `todo!()` in test code
- [ ] Arrange-Act-Assert pattern followed
