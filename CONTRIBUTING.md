# Contributing to GoudEngine

## Development Environment

**Prerequisites**: Rust (stable, edition 2021), .NET 8 SDK, Python 3.9+ (3.11 recommended), Node.js 16+ (20 recommended), cbindgen, cargo-deny, CMake. Only Rust and the .NET SDK are required for core development; Python and Node.js support is optional. GLFW is needed only when building the legacy `legacy-glfw-opengl` backend — the default wgpu/winit backend does not require it. See [docs/src/development/dev-setup.md](docs/src/development/dev-setup.md) for the full per-platform setup, including the MSRV toolchain.

```bash
git clone https://github.com/aram-devdocs/GoudEngine
cd GoudEngine
cargo build
```

## Minimum Supported Rust Version

The toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml). rustup reads this file and resolves the same Rust channel and components (`rustfmt`, `clippy`) for every developer, CI runner, and container, so builds are reproducible. Treat a toolchain bump as a deliberate change: update `rust-toolchain.toml` and this note together in the same PR.

Run an example to verify the setup works:

```bash
./dev.sh --game flappy_goud       # C# example (requires .NET)
./dev.sh --sdk python --game flappy_bird  # Python example
cargo run -p flappy-bird          # Rust example
```

## Rust-First Principle

All game logic lives in Rust. Language SDKs (under `sdks/`) are thin wrappers that marshal data and call FFI functions. They contain no logic, math, or state.

If you want to add a feature:

1. Implement it in `goud_engine/src/` or `libs/`
2. Export it via `goud_engine/src/ffi/`
3. Run `cargo build` to refresh the generated FFI surfaces (`NativeMethods.g.cs`, `ffi_manifest.json`, and `codegen/generated/goud_engine.h`)
4. Update `codegen/goud_sdk.schema.json`
5. Run `./codegen.sh` to regenerate all SDK wrappers
6. Write tests for the Rust implementation

Never put logic in an SDK. If you find logic in an SDK, move it to Rust.

**Exception**: Simple value-type math (Vec2.add, Color.fromHex) in the TypeScript SDK is intentionally local to avoid FFI round-trips. These are code-generated for consistency.

## Ownership Rule

Follow the campsite rule: leave the code better than you found it. Nothing in this repo is "pre-existing" or someone else's problem. If you touch a file and find a broken test, a dead code path, a stale comment, or a lint the gate should have caught, fix it as part of your change. Do not route around a defect and defer it — a defect you can see is a defect you own.

## Layer Architecture

GoudEngine enforces a 5-layer dependency hierarchy within `goud_engine/src/`. Dependencies flow down only. No upward imports. No same-layer cross-imports. See [ARCHITECTURE.md](ARCHITECTURE.md#layer-hierarchy) for the full model. The canonical definition is in `tools/lint_layers.rs`.

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

Tests that need a GPU context must call `test_helpers::init_test_context()`. Pure math/logic tests must not require a GPU context.

## Verification Gate

The same checks run at three gates. `scripts/verify.sh` is the single source of truth; the pre-commit gate runs a strict subset of the full suite so that anything passing pre-commit still passes pre-push and CI.

| Gate | Command | Scope |
|------|---------|-------|
| pre-commit hook | `scripts/verify.sh --staged` | Fast subset over staged changes |
| pre-push hook | `scripts/verify.sh` | Full suite |
| CI | `scripts/verify.sh` | Full suite |

Bypassing the gate is NOT allowed. Do not commit with `--no-verify`, do not push with `--no-verify`, and do not add `[skip ci]` to land code past CI. If a rule fires incorrectly, fix the rule (or the code it flags) — never disable the gate to get a change through.

If your local git hooks do not run, git is probably pointing at a stale hooks path from another tool. Clear it so the repo's hooks take effect:

```bash
git config --unset core.hooksPath
```

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
- No `#[ignore]` in committed code except for GPU-context-dependent tests (OpenGL, audio, sprite batch, render system) that cannot run in headless CI. Run them locally with `cargo test -- --ignored`.
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
| Architecture docs | `docs/src/architecture/` |
| Development guide | `docs/src/development/dev-setup.md` |
| Build guide | `docs/src/development/building.md` |
