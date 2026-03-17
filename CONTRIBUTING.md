# Contributing to GoudEngine

## Development Environment

**Prerequisites**: Rust (stable), .NET 8 SDK, Python 3.x, Node.js 18+, CMake, GLFW.

```bash
git clone https://github.com/aram-devdocs/GoudEngine
cd GoudEngine
cargo build
```

Run an example to verify the setup works:

```bash
./dev.sh --game flappy_goud       # C# example (requires .NET)
./dev.sh --sdk python --game flappy_bird  # Python example
cargo run -p flappy-bird          # Rust example
```

## Rust-First Principle

All game logic lives in Rust. Language SDKs (C#, Python, TypeScript) are thin wrappers that marshal data and call FFI functions. They contain no logic, math, or state.

If you want to add a feature:

1. Implement it in `goud_engine/src/` or `libs/`
2. Export it via `goud_engine/src/ffi/`
3. Run `cargo build` to refresh the generated FFI surfaces (`NativeMethods.g.cs`, `ffi_manifest.json`, and `codegen/generated/goud_engine.h`)
4. Update `codegen/goud_sdk.schema.json`
5. Run the codegen generators to update C#, Python, and TypeScript SDK wrappers
6. Write tests for the Rust implementation

Never put logic in an SDK. If you find logic in an SDK, move it to Rust.

**Exception**: Simple value-type math (Vec2.add, Color.fromHex) in the TypeScript SDK is intentionally local to avoid FFI round-trips. These are code-generated for consistency.

## Layer Architecture

Dependencies flow down only. No upward imports. No same-layer cross-imports.

```
Layer 1 (Core)   :  libs/              — graphics, platform, ecs, logger
Layer 2 (Engine) :  goud_engine/src/   — core, assets, sdk
Layer 3 (FFI)    :  goud_engine/src/ffi/
Layer 4 (SDKs)   :  sdks/             — csharp, python, typescript
Layer 5 (Apps)   :  examples/
```

A `use goud_engine::` in `libs/` is a hierarchy violation. Check `use` statements when touching module boundaries.

## Building and Testing

```bash
# Build
cargo build                      # debug build
cargo build --release            # release build
./build.sh --release             # full release build with SDK packaging

# Test
cargo test                       # all tests
cargo test -- --nocapture        # show stdout
cargo test --lib sdk             # specific module
cargo test graphics              # filter by name

# Pre-commit checks (all must pass before opening a PR)
cargo check
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo deny check

# SDK tests
python3 sdks/python/test_bindings.py
cd sdks/typescript && npm test
```

Tests that need an OpenGL context must call `test_helpers::init_test_context()`. Pure math/logic tests must not require a GL context.

## Code Style

- `cargo fmt` — all code must be formatted
- `cargo clippy -- -D warnings` — no warnings allowed
- 500 line limit per file — split large files before they exceed this
- `unsafe` blocks require a `// SAFETY:` comment explaining soundness
- FFI functions must be `#[no_mangle] extern "C"`
- FFI structs shared across the boundary must be `#[repr(C)]`
- Pointer parameters must be null-checked before dereferencing

## Testing Requirements

Follow red-green-refactor:

1. Write a failing test first
2. Write minimal code to make it pass
3. Refactor, keeping tests green

Rules:
- No `#[ignore]` on committed tests
- No `todo!()` or `unimplemented!()` in production code
- Test names describe behavior: `test_transform2d_translate_updates_position`, not `test1`
- Aim for 80%+ line coverage and 70%+ branch coverage on new code
- Arrange-Act-Assert pattern

## Commit Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/). PR titles must follow the format:

```
feat: add physics provider trait
fix: null-check pointer in ffi/renderer.rs
chore: bump version
docs: update CONTRIBUTING.md
refactor: split schedule.rs into subsystem files
test: add camera2d projection tests
```

`release-please` reads PR titles to determine version bumps and generate changelogs. A `feat:` triggers a minor bump. A `fix:` triggers a patch bump. A `feat!:` or `BREAKING CHANGE:` footer triggers a major bump.

## PR Process

1. Fork the repo and create a branch from `main`
2. Make your changes, run `cargo check && cargo fmt --all -- --check && cargo clippy -- -D warnings`
3. Write tests — new code without tests will not be merged
4. Open a PR with a conventional commit title
5. CI must pass (build, test, clippy, security audit)
6. At least one maintainer review required before merge

Keep PRs focused. One logical change per PR. Large refactors should be discussed in an issue first.

## Where to Find Docs

| Resource | Location |
|----------|----------|
| AI agent guidance | `AGENTS.md` |
| Alpha development roadmap | `ALPHA_ROADMAP.md` |
| Architecture rules | `.agents/rules/` |
| Graphics subsystem | `.agents/rules/graphics-patterns.md` |
| FFI boundary patterns | `.agents/rules/ffi-patterns.md` |
| ECS patterns | `.agents/rules/ecs-patterns.md` |
| SDK development | `.agents/rules/sdk-development.md` |
| Architecture docs | `docs/architecture/` |
| Development guide | `docs/DEVELOPMENT.md` |
| Build guide | `docs/BUILDING.md` |
