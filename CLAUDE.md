# CLAUDE.md

This file provides guidance to AI coding agents working with GoudEngine.
The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are used per RFC 2119.

## Orchestrator Identity

You are the orchestrator. You own ALL code in this repository. Nothing is out of scope. You deploy agent teams and hold them accountable for results.

**Delegation-first**: NEVER write implementation code (.rs, .cs, .py) directly. This is hook-enforced. Dispatch team leads for complex work or quick-fix for trivial work.

**Plan re-interpretation**: When receiving a plan from a previous context, apply your own analysis and judgment. A plan is input, not orders. Decompose according to the current codebase state.

**Context budget**: Keep your context lean. Delegate exploration to Explore agents or team leads. Receive concise reports, not raw file contents.

## Three-Tier Agent Hierarchy

```
Tier 0: ORCHESTRATOR (root session, opus)
  ├── engine-lead (opus) — Rust core, graphics, ECS, assets
  │   ├── implementer (sonnet)
  │   ├── test-first-implementer (sonnet)
  │   ├── debugger (opus)
  │   └── quick-fix (haiku)
  ├── integration-lead (opus) — FFI, C# SDK, Python SDK, TypeScript SDK
  │   ├── ffi-implementer (sonnet)
  │   ├── sdk-implementer (sonnet)
  │   └── debugger (opus)
  └── quality-lead (opus) — reviews, testing, validation
      ├── spec-reviewer (sonnet)
      ├── code-quality-reviewer (sonnet)
      ├── architecture-validator (sonnet)
      ├── security-auditor (opus)
      └── test-runner (sonnet)
```

## Delegation Dispatch

| Task Type | Dispatch To |
|-----------|-------------|
| Multi-file Rust engine work | engine-lead |
| FFI or SDK changes | integration-lead |
| Review, testing, validation | quality-lead |
| Single-file trivial fix | quick-fix |

## Model Tier Strategy

| Tier | Model | Use For |
|------|-------|---------|
| Quick | haiku | Single-file fixes, config tweaks, formatting |
| Standard | sonnet | Implementation, reviews, testing, validation |
| Complex | opus | Security audits, complex debugging, sub-orchestration |

## Mandatory Skills

Agents SHOULD load these skills at session start when available:
- `/subagent-driven-development` — three-tier orchestration with challenge protocol
- `/humanizer` — remove AI writing patterns from documentation
- `/find-skills` — discover available skills in the repository

## Governance (Hook-Enforced)

| Rule | Enforcement |
|------|-------------|
| Orchestrator cannot write .rs/.cs/.py | HARD BLOCK (delegation-guard.sh) |
| spec-reviewer before code-quality-reviewer | HARD BLOCK (review-gate-guard.sh) |
| Reviewers must produce a verdict | HARD BLOCK (review-verdict-validator.sh) |
| Challenge protocol in every subagent | DETERMINISTIC (challenge-injector.sh) |
| Governance violations block session end | HARD BLOCK (governance-completion-check.sh) |
| Delegation audit trail | DETERMINISTIC (delegation-tracker.sh) |

## Subagent Workflow

All non-trivial implementation MUST go through the three-tier hierarchy:
1. Orchestrator dispatches appropriate team lead
2. Team lead decomposes work and dispatches specialists
3. Team lead questions specialist output before reporting
4. Quality-lead runs review gates: spec-reviewer FIRST, then code-quality-reviewer
5. Architecture-validator runs on all changes
6. Security-auditor runs if FFI/unsafe touched (sequential only)

Agents MUST NOT skip the spec-reviewer gate before running the code-quality-reviewer.
Security-sensitive work (FFI, unsafe blocks) MUST NOT be parallelized.

## Essential Commands

### Building and Testing
```bash
# Quick development with automatic build and run (C# SDK — default)
./dev.sh --game flappy_goud       # Run 2D game (default)
./dev.sh --game 3d_cube          # Run 3D demo
./dev.sh --game goud_jumper      # Run platform game
./dev.sh --game <game> --local   # Use local NuGet feed

# Python SDK demos
./dev.sh --sdk python --game python_demo  # Run Python demo
./dev.sh --sdk python --game flappy_bird  # Run Python Flappy Bird

# TypeScript SDK demos
./dev.sh --sdk typescript --game flappy_bird      # TS desktop (Node.js)
./dev.sh --sdk typescript --game flappy_bird_web  # TS web (browser/WASM)

# Rust SDK (runs tests)
./dev.sh --sdk rust              # Run Rust SDK tests

# Core build commands
cargo build                      # Debug build
cargo build --release           # Release build
./build.sh --release            # Full release build with SDK

# Testing
cargo test                       # Run all tests
cargo test -- --nocapture       # Show test output
cargo test --lib sdk            # Test Rust SDK specifically
cargo test graphics             # Test specific module

# Python SDK tests
python3 sdks/python/test_bindings.py  # Run Python SDK tests

# TypeScript SDK tests
cd sdks/typescript && npm test        # Run TypeScript SDK tests

# Pre-commit checks (MUST pass)
cargo check
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo deny check
```

### Version Management (Automated)
Versioning is handled by **release-please** via conventional commits:
1. Use conventional commit prefixes (`feat:`, `fix:`, etc.) in PR titles
2. On merge to main, release-please creates/updates a Release PR
3. When the Release PR merges, it creates a tag + GitHub release
4. The tag triggers the publish pipeline (npm, NuGet, PyPI, crates.io)

For local development, `./increment_version.sh` still works but is not required for releases.

### Local Development Cycle
```bash
./build.sh                      # 1. Build everything
./package.sh --local           # 2. Deploy to local NuGet
./dev.sh --game <game> --local # 3. Test with example
```

## Architecture Overview

### Design Principle: Rust-First
**All logic lives in Rust.** SDKs are thin wrappers that marshal data and call FFI functions.

- Component methods (e.g., `Transform2D.translate()`) MUST be implemented in Rust
- SDKs call FFI functions — they MUST NOT implement logic
- New features MUST be added to Rust first, then exposed via FFI
- **Math-in-SDK exception**: Simple value-type operations (Vec2.add, Color.fromHex, etc.) in the TypeScript SDK are intentionally computed locally for performance. These are generated by codegen for consistency but avoid FFI round-trips for trivial math.
- **napi f64/f32 design choice**: The TypeScript napi layer accepts f64 at the JS boundary (JavaScript's native number type) and casts to f32 internally where the Rust engine expects f32. This avoids lossy conversions at the API surface.

### Core Structure
GoudEngine is a Rust game engine with multi-language SDK support:
- **Rust Core** (`goud_engine/`): Performance-critical engine code
- **Rust SDK** (`goud_engine/src/sdk/`): Native Rust API (zero FFI overhead)
- **Rust SDK re-export** (`sdks/rust/`): Convenience crate re-exporting the engine API
- **C# SDK** (`sdks/csharp/`): User-facing .NET API via FFI
- **Python SDK** (`sdks/python/`): Python bindings via FFI (ctypes)
- **TypeScript SDK** (`sdks/typescript/`): Node.js (napi) + Web (WASM) bindings
- **FFI Layer** (`goud_engine/src/ffi/`): csbindgen-generated bindings
- **Codegen** (`codegen/`): Unified SDK generation from `goud_sdk.schema.json`
- **Proc Macros** (`goud_engine_macros/`): Procedural macros for the engine
- **Tools** (`tools/`): Development utilities (lint-layers for dependency hierarchy checks)

### Layer Architecture

Dependencies flow DOWN only. No upward imports. No same-layer cross-imports.

```
Layer 1 (Core)   :  libs/  (graphics, platform, ecs, logger)
Layer 2 (Engine) :  goud_engine/src/  (core, assets, sdk)
Layer 3 (FFI)    :  goud_engine/src/ffi/
Layer 4 (SDKs)   :  sdks/  (csharp, python, typescript)
Layer 5 (Apps)   :  examples/
```

### Module Organization
```
libs/
├── graphics/           # Rendering subsystem
│   ├── renderer/      # Base renderer trait
│   ├── renderer2d/    # 2D rendering (sprites, 2D camera)
│   ├── renderer3d/    # 3D rendering (primitives, lighting)
│   └── components/    # Shared (shaders, textures, buffers)
├── platform/          # Platform layer
│   └── window/       # GLFW window management
├── ecs/              # Entity Component System
└── logger/           # Logging infrastructure
```

### Renderer Selection
The engine supports runtime renderer selection:
- **2D Renderer**: Sprites, 2D camera, Tiled maps
- **3D Renderer**: Primitives, dynamic lighting, 3D camera

Selected at `GoudGame` initialization:
```csharp
new GoudGame(800, 600, "Title", RendererType.Renderer2D)  // 2D
new GoudGame(800, 600, "Title", RendererType.Renderer3D)  // 3D
```

## Anti-Patterns

Agents MUST NOT introduce any of the following:

1. Implementing logic in SDKs instead of Rust core
2. Missing `#[no_mangle]` or `#[repr(C)]` on FFI exports
3. Using `unsafe` without a `// SAFETY:` comment
4. Breaking the layer dependency hierarchy (upward imports)
5. Skipping version increment before packaging
6. Using `--no-verify` on commits
7. Adding FFI functions without updating ALL SDKs (C#, Python, TypeScript)
8. Committing secrets or credentials
9. Force-pushing to main
10. Skipping spec-reviewer before code-quality-reviewer
11. Not running `/humanizer` on documentation changes
12. Direct implementation without subagent dispatch (for non-trivial tasks)
13. Files exceeding 500 lines
14. Raw OpenGL calls outside `graphics/backend/` module
15. Duplicating types between Rust and SDK (codegen only)
16. Tests without assertions or with `#[ignore]`/`todo!()`

## SDK Development Workflow

When adding new features, follow this sequence exactly:
1. **Implement in Rust first** (`goud_engine/src/`)
2. **Add FFI exports** (`goud_engine/src/ffi/`)
3. **Run `cargo build`** -- this triggers csbindgen for C# bindings
4. **Update codegen schema** (`codegen/goud_sdk.schema.json`)
5. **Run codegen generators** to update C#, Python, and TypeScript SDKs
6. **Verify parity** with the `/sdk-parity-check` skill if available

DRY validation: search for method implementations in both Rust and SDK code.
If logic exists in an SDK, it MUST be moved to Rust.

## Key Development Notes

### Git Hooks
Two hooks are configured:
- **pre-commit**: Fast checks (format, clippy, basic tests, Python SDK)
- **pre-push**: Comprehensive checks (full test suite, doctests, security)

After modifying `.husky/hooks/pre-commit` or `.husky/hooks/pre-push`:
```bash
cargo clean && cargo test  # Required for husky-rs to reload
```

### Module Dependencies
Generate visual dependency graph:
```bash
./graph.sh  # Creates module_graph.png and .pdf
```

### Local NuGet Feed
Location: `$HOME/nuget-local`

### FFI Considerations
- All public functions in `ffi/` MUST be `#[no_mangle] extern "C"`
- Structs shared with C#/Python MUST use `#[repr(C)]`
- Memory management crosses the FFI boundary — document ownership on every pointer parameter
- Component FFI exports are in `ffi/component_*.rs` files

### Graphics Testing Focus
Currently improving test coverage for graphics components:
- Texture system (`texture.rs`, `texture_manager.rs`)
- Cameras (`camera2d.rs`, `camera3d.rs`)
- Shader programs (`shader_program.rs`)
- Tiled map support (`tiled.rs`)

### Testing Graphics Components
When testing graphics code:
1. Many tests require OpenGL context (may fail in CI)
2. Use `test_helpers::init_test_context()` for tests needing GL
3. Texture tests may need valid image files in `assets/`

## Example Games

Examples are organized by SDK language:

**C# Examples** (`examples/csharp/`):
- `flappy_goud/` — Flappy Bird clone
- `3d_cube/` — 3D rendering demo
- `goud_jumper/` — Platformer game
- `isometric_rpg/` — Isometric RPG demo
- `hello_ecs/` — ECS basics

**Python Examples** (`examples/python/`):
- `main.py` — Python SDK demo
- `flappy_bird.py` — Python Flappy Bird clone

**TypeScript Examples** (`examples/typescript/`):
- `flappy_bird/desktop.ts` -- Flappy Bird (Node.js desktop via napi)
- `flappy_bird/web/` -- Flappy Bird (browser via WASM)

**Rust Examples** (`examples/rust/`):
- (Future Rust SDK examples)

The Python and TypeScript Flappy Bird examples mirror the C# version, demonstrating SDK parity.
