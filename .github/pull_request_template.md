## Overview

**Type:** <!-- feature | fix | refactor | docs | test | chore | ci | build | perf -->

**Summary:**
<!-- Briefly describe what this PR does and why. -->

**Related Issues:** <!-- Fixes #123, Closes #456 -->

---

## Changes Made

### Engine Core (`goud_engine/src/`)
<!-- List Rust engine changes, or write "No changes" -->

### FFI Layer (`goud_engine/src/ffi/`)
<!-- List FFI boundary changes, or write "No changes" -->

### C# SDK (`sdks/GoudEngine/`)
<!-- List C# SDK changes, or write "No changes" -->

### Python SDK (`sdks/python/`)
<!-- List Python SDK changes, or write "No changes" -->

### Examples (`examples/`)
<!-- List example game changes, or write "No changes" -->

### Documentation
<!-- List doc changes, or write "No changes" -->

---

## Architectural Compliance

- [ ] **Rust-first**: All logic lives in Rust; SDKs are thin wrappers
- [ ] **FFI boundary**: New exports use `#[no_mangle] extern "C"` and `#[repr(C)]` where needed
- [ ] **Dependency flow**: Imports follow layer hierarchy (down only)
- [ ] **SDK parity**: Changes exposed via FFI are wrapped in both C# and Python SDKs
- [ ] **Unsafe discipline**: No `unsafe` block without a `// SAFETY:` comment
- [ ] **File size**: No file exceeds 500 lines

---

## Testing

- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` is clean
- [ ] `cargo fmt --all -- --check` passes
- [ ] Python SDK tests pass (`python3 sdks/python/test_bindings.py`) — if SDK changed
- [ ] C# SDK tests pass (`dotnet test sdks/GoudEngine.Tests/`) — if SDK changed
- [ ] Pre-commit hooks pass

---

## Code Quality

- [ ] No `todo!()` or `unimplemented!()` in production code
- [ ] No `#[allow(unused)]` without justification comment
- [ ] Error handling uses `Result`, not `unwrap()`/`expect()` in library code
- [ ] Public items have doc comments

---

## Documentation

- [ ] Updated relevant `CLAUDE.md` files (if architecture changed)
- [ ] Updated `README.md` (if user-facing behavior changed)
- [ ] Added or updated doc comments on new public APIs

---

## Breaking Changes

<!-- Describe any breaking changes, or write "None" -->

- API changes:
- FFI signature changes:
- SDK interface changes:

---

## Version Bump

<!-- patch | minor | major | none -->

**Bump type:**
**Justification:**

---

## Security

- [ ] No new `unsafe` blocks — or each one has a `// SAFETY:` comment and is necessary
- [ ] No new FFI pointer parameters without null checks
- [ ] No new dependencies with known advisories (`cargo deny check`)
- [ ] No secrets or credentials in committed files

---

## Performance

<!-- Describe any allocation changes, hot-path modifications, or benchmark results. Write "N/A" if not applicable. -->

---

## Deployment

- [ ] NuGet package version updated (if SDK changed)
- [ ] Python package version updated (if SDK changed)
- [ ] Native library builds on all targets (macOS, Linux, Windows)

---

## Reviewer Notes

<!-- Call out areas that need careful review, known trade-offs, or open questions. -->
