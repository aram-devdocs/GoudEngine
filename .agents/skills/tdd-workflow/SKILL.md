---
name: tdd-workflow
description: RED-GREEN-REFACTOR TDD pipeline with agent dispatch for Rust engine code
context: fork
user-invocable: true
---

# TDD Workflow

Test-Driven Development pipeline using the RED-GREEN-REFACTOR cycle with subagent dispatch.

## When to Use

Use for all new feature implementation and bug fixes. Skip only for trivial documentation changes or configuration updates.

## The Cycle

### RED Phase — Write Failing Tests First

**Agent**: `test-first-implementer`

1. Read the feature requirements or bug report
2. Write test functions in the appropriate location:
   - Unit tests: `#[cfg(test)]` module in the same file
   - Integration tests: `goud_engine/tests/`
   - SDK tests: `sdks/python/test_bindings.py` or `sdks/csharp.tests/`
3. Run tests and verify they FAIL:

```bash
cargo test <test_name> -- --nocapture
```

4. Confirm failure is for the RIGHT reason (missing function, wrong result — not compilation error unrelated to the feature)

**Rules**:
- One assertion per concept
- Arrange-Act-Assert pattern
- Descriptive test names: `test_transform2d_translate_updates_position`
- GL-dependent tests MUST use `test_helpers::init_test_context()`
- Math/logic tests MUST NOT require GL context
- No `#[ignore]`, no `todo!()`, no `unimplemented!()`

### GREEN Phase — Minimal Implementation

**Agent**: `implementer` (or `ffi-implementer` / `sdk-implementer` as appropriate)

1. Write the minimal code to make the failing test pass
2. Do not add extra functionality beyond what the test requires
3. Run tests and verify they PASS:

```bash
cargo test <test_name> -- --nocapture
```

4. If tests still fail, iterate on the implementation (not the test)

### REFACTOR Phase — Improve Quality

**Agent**: `implementer` (same agent, continued)

1. With all tests passing, refactor for:
   - Clarity and readability
   - Removing duplication
   - Proper error handling (`Result` instead of `unwrap`)
   - Performance (remove unnecessary allocations)
2. Run full test suite after each refactor step:

```bash
cargo test
cargo clippy -- -D warnings
```

3. Tests MUST remain green throughout refactoring

## Multi-Module TDD

When a feature spans multiple modules (e.g., new component → FFI → SDK):

```
RED:   Write Rust unit test for the component
GREEN: Implement the Rust component
RED:   Write Rust test for the FFI export
GREEN: Implement the FFI export
RED:   Write Python test for the binding
GREEN: Implement the Python binding
RED:   Write C# test for the wrapper
GREEN: Implement the C# wrapper
REFACTOR: Clean up all layers
```

## Verification Checklist

Before marking a TDD cycle complete:

- [ ] All new tests pass
- [ ] Full `cargo test` suite passes
- [ ] `cargo clippy -- -D warnings` clean
- [ ] No `#[ignore]` or `todo!()` added
- [ ] Test names are descriptive
- [ ] Each test has exactly one logical assertion
- [ ] GL-dependent tests properly initialize context

## Anti-Patterns

- Writing implementation before tests (defeats the purpose)
- Writing tests that can never fail (tautological assertions)
- Using `#[ignore]` to skip inconvenient tests
- Testing implementation details instead of behavior
- Tests that depend on execution order
