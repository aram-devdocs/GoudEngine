# Development Guide

> **Alpha** — GoudEngine is under active development. APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues) · [Contact](mailto:aram.devdocs@gmail.com)

## Quick Start

Use `dev.sh` to build and run examples in one step:

```sh
# C# SDK (default)
./dev.sh --game flappy_goud         # 2D game example
./dev.sh --game 3d_cube             # 3D game example
./dev.sh --game goud_jumper         # Platform game example
./dev.sh --game flappy_goud --local # Use local NuGet feed

# Python SDK
./dev.sh --sdk python --game python_demo  # SDK demo
./dev.sh --sdk python --game flappy_bird  # Flappy Bird

# TypeScript SDK
./dev.sh --sdk typescript --game flappy_bird      # Desktop (Node.js)
./dev.sh --sdk typescript --game flappy_bird_web  # Web (WASM)

# Rust SDK
./dev.sh --sdk rust  # Run Rust SDK tests
```

## Local Development Cycle

```sh
./build.sh                       # 1. Build engine and SDKs
./package.sh --local             # 2. Deploy to local NuGet feed
./dev.sh --game <game> --local   # 3. Test with example
```

## Git Hooks

This project uses [husky-rs](../reference/husky-rs.md) for Git hook management.

**Pre-commit** (fast): format, clippy, basic tests, Python SDK checks.

**Pre-push** (thorough): full test suite, doctests, security audit.

After editing `.husky/hooks/pre-commit` or `.husky/hooks/pre-push`, run:

```sh
cargo clean && cargo test
```

This is required for husky-rs to reload the hooks via `build.rs`.

## Version Management

Versioning is automated through [release-please](https://github.com/googleapis/release-please) via conventional commits.

1. Use conventional commit prefixes (`feat:`, `fix:`, `chore:`) in PR titles.
2. On merge to main, release-please creates or updates a Release PR.
3. When the Release PR merges, it creates a tag and GitHub release.
4. The tag triggers the publish pipeline (npm, NuGet, PyPI, crates.io).

For local testing, `./increment_version.sh` updates versions manually:

```sh
./increment_version.sh           # Patch version (0.0.X)
./increment_version.sh --minor   # Minor version (0.X.0)
./increment_version.sh --major   # Major version (X.0.0)
```

The script updates `goud_engine/Cargo.toml` (source of truth), `sdks/csharp/GoudEngine.csproj`, and all `.csproj` files under `examples/`.

## Pre-commit Checks

Run these before pushing to confirm the build is clean:

```sh
cargo check
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo deny check
cargo test
```
