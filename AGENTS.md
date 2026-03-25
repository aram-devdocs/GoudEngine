# AGENTS.md

Agent workflow for GoudEngine. Keep sessions fast and direct. Use multi-agent only when it improves the result. ALWAYS use /find-skills to see what is available, but always use subagent driven development and humanizer.

## Essential Commands

```bash
cargo build                              # debug build
cargo test                               # all tests
cargo check && cargo fmt --all -- --check && cargo clippy -- -D warnings
./codegen.sh                             # full SDK codegen pipeline
./dev.sh --game flappy_goud              # run example
```

## Core Invariants

- **Rust first.** Engine behavior lives in Rust. SDKs are thin FFI wrappers.
- **FFI safety.** `#[no_mangle] extern "C"`, `#[repr(C)]`, null checks, `// SAFETY:` comments.
- **Architecture.** Downward-only dependencies. See `tools/lint_layers.rs` for canonical 5-layer model (Foundation/Libs/Services/Engine/FFI). Raw GPU calls stay in `libs/graphics/backend/`.
- **Generated code.** Do not hand-edit `*.g.rs`, `*.g.ts`, `*.g.cs`. Update Rust + FFI + schema + SDKs together.

## Workflow

See `.agents/rules/orchestrator-protocol.md` for dispatch guide, subagent reference, and `/gh-issue` details.

## Agent Config Generation

```bash
python3 scripts/sync-agent-configs.py        # generate
python3 scripts/sync-agent-configs.py --check # validate
```

Source of truth: `.agents/agent-catalog.toml` and `.agents/role-specs/*.md`.

## Rules

Read only rules matching your change area: `.agents/rules/*.md`.

## Local AGENTS Files

repo root, `codegen/`, `goud_engine/src/ffi/`, `sdks/typescript/`, `examples/`.
