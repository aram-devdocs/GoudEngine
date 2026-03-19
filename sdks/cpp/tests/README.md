# C++ SDK Tests

CTest-based test suite using Catch2 v3 for the GoudEngine C++ SDK.

## Prerequisites

- CMake 3.14+
- C++17 compiler
- Native library built (`cargo build` or `cargo build --release` from repo root)

## Build

```bash
cmake -B build sdks/cpp/tests
cmake --build build
```

## Run

```bash
# All tests (including GL-required)
ctest --test-dir build

# Non-GL tests only (safe for headless CI)
ctest --test-dir build --label-exclude gl_required

# Verbose output
ctest --test-dir build --output-on-failure
```

## Test Tags

| Tag | Description |
|-----|-------------|
| `[error]` | `goud::Error` default construction and last-error retrieval |
| `[config]` | `goud::EngineConfig` create, setters, move, reset, unique_ptr |
| `[context]` | `goud::Context` validity, move, entity spawn/destroy |
| `[engine]` | `goud::Engine` creation, shared_ptr factory |
| `[constants]` | Flappy Bird game constants parity check |
| `[gl_required]` | Requires a live GPU/GL context; skip in headless CI |
