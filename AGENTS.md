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

See `.agents/rules/orchestrator-protocol.md` for full workflow and `/gh-issue` details.

### Subagent Dispatch Reference

| Role | Model | Use For |
|------|-------|---------|
| engine-lead | opus | Direct implementation for Rust engine and core modules |
| integration-lead | opus | Direct implementation for FFI, SDK, and codegen work |
| quick-fix | haiku | Tightly scoped low-risk fixes |
| reviewer | sonnet | Single review pass for requirements, tests, and major quality regressions |
| security-auditor | opus | FFI, unsafe, pointer, and ownership-boundary review |
| debugger | sonnet | Root-cause analysis for failing verification or runtime issues |

This table is a curated subset. The full 10-role catalog lives in `.agents/agent-catalog.toml`; the `default` orchestrator and the three strict `/gh-issue` gate reviewers (spec-reviewer, code-quality-reviewer, architecture-validator) are omitted here on purpose.

## Agent Config Generation

```bash
python3 scripts/sync-agent-configs.py        # generate
python3 scripts/sync-agent-configs.py --check # validate
```

Source of truth: `.agents/agent-catalog.toml` and `.agents/role-specs/*.md`.

## Rules

Read only rules matching your change area: `.agents/rules/*.md`.

## Local AGENTS Files

Nested `AGENTS.md` files exist throughout the tree (repo root, `codegen/`, `goud_engine/src/ffi/`, `sdks/`, `examples/`, and more). The nearest-ancestor file wins for any given path.
