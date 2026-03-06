---
globs:
  - "**/*test*"
  - "**/tests/**"
  - "**/benches/**"
---

# Testing Conventions

## Test Organization

- **Unit tests**: colocated in `#[cfg(test)]` modules within the source file
- **Integration tests**: in `goud_engine/tests/` directory
- **SDK tests**: `sdks/csharp.tests/` (C#), `sdks/python/test_bindings.py` (Python)
- **Benchmarks**: in `goud_engine/benches/` using criterion

## GL Context

- Tests requiring OpenGL MUST use `test_helpers::init_test_context()` to set up a valid GL context
- Math and logic tests MUST NOT require a GL context — keep them pure
- GL-dependent tests may fail in CI; mark them appropriately

## Running Tests

```
cargo test                       # All tests
cargo test -- --nocapture        # With output (for debugging)
cargo test --lib sdk             # Specific module
cargo test graphics              # By name filter
cargo bench                      # Benchmarks
```

## Writing Tests

- Follow Arrange-Act-Assert pattern
- One logical assertion per test (multiple `assert!` calls for the same concept are fine)
- Meaningful test names: `test_transform2d_translate_updates_position`, not `test1`
- No `#[ignore]` in committed code
- No `todo!()` or `unimplemented!()` in test helpers

## Coverage Targets

- 80%+ line coverage on new code
- 70%+ branch coverage on new code
- Focus benchmarks on hot paths (rendering loops, ECS iteration, asset loading)
