# AGENTS.md

This file defines the default agent workflow for GoudEngine.

## Goals

- Keep ordinary sessions fast and direct.
- Use multi-agent workflows only when they improve the result.
- Preserve the engine's real correctness constraints: architecture, FFI safety, SDK parity, and testing.
- ALWAYS use /find-skills to see what is available, but always use subagent driven development and humanizer.

## Essential Commands

```bash
# Core build
cargo build
cargo build --release
./build.sh --release

# Tests
cargo test
cargo test -- --nocapture
cargo check
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo deny check

# SDK checks
python3 sdks/python/test_bindings.py
cd sdks/typescript && npm test

# Examples
./dev.sh --game flappy_goud
./dev.sh --game 3d_cube
./dev.sh --game goud_jumper
./dev.sh --sdk python --game python_demo
./dev.sh --sdk typescript --game flappy_bird
```

## Core Invariants

### Rust First

- Engine behavior lives in Rust.
- SDKs are thin wrappers over FFI or generated bindings.
- Simple generated TypeScript math helpers are the only intentional local-logic exception.

### FFI Safety

- Public FFI exports must be `#[no_mangle] extern "C"`.
- Shared FFI structs must use `#[repr(C)]`.
- Pointer parameters need null checks before dereference.
- Every `unsafe` block needs a `// SAFETY:` comment.
- Memory ownership across FFI must be explicit.

### Architecture

- Dependency flow stays downward only: `libs -> goud_engine -> ffi -> sdks -> examples`.
- Do not add upward imports or same-layer cross-module shortcuts.
- Raw OpenGL calls stay in the graphics backend.

### Generated Code

- Do not hand-edit generated `*.g.rs`, `*.g.ts`, or `*.g.cs` surfaces.
- Do not commit napi loader outputs such as `index.js` and `index.d.ts`.
- Update Rust, FFI, schema, and generated SDKs together when public engine APIs change.

## Default Workflow

- Trivial fix: root session or `quick-fix`.
- Multi-file engine work: `engine-lead`.
- FFI, SDK, or codegen work: `integration-lead`.
- Substantive changes get one `reviewer` pass after implementation.
- Use `security-auditor` only for FFI, unsafe, pointer, or memory-boundary changes.
- Use `debugger` when failures need focused diagnosis.

This repo still supports multi-agent work, but the default shape is intentionally small:

1. Root scopes the task.
2. One implementation agent does the work.
3. One reviewer checks the result.
4. Security review is added only when the change warrants it.

Do not stack multiple review gates or nested specialist waves by default.

### Subagent Dispatch Reference

| Role | Model | Use For |
|------|-------|---------|
| engine-lead | opus | Direct implementation for Rust engine and core modules |
| integration-lead | opus | Direct implementation for FFI, SDK, and codegen work |
| quick-fix | haiku | Tightly scoped low-risk fixes |
| reviewer | sonnet | Single review pass for requirements, tests, and major regressions |
| security-auditor | opus | FFI, unsafe, pointer, and ownership-boundary review |
| debugger | sonnet | Root-cause analysis for failing verification or runtime issues |

## `/gh-issue`

- `/gh-issue` is opt-in.
- It is the strict delivery path when explicitly invoked.
- It owns worktree/run-state/PR tracking, PR template usage, Claude review follow-through, CI waiting, and worktree cleanup only when explicitly invoked.
- Ordinary sessions should not behave like `/gh-issue` runs.

## Agent Config Generation

Agent wrappers are generated from the canonical catalog:

```bash
python3 scripts/sync-agent-configs.py
python3 scripts/sync-agent-configs.py --check
```

Source of truth:

- `.agents/agent-catalog.toml`
- `.agents/role-specs/*.md`

Provider intent:

- Claude is the primary orchestration target.
- Codex stays available as a thinner fallback configuration.

## Rules Worth Reading

Read only the rules that match the area you are changing:

- `.agents/rules/dependency-hierarchy.md`
- `.agents/rules/ffi-patterns.md`
- `.agents/rules/testing.md`
- `.agents/rules/sdk-development.md`
- `.agents/rules/graphics-patterns.md`
- `.agents/rules/ecs-patterns.md`
- `.agents/rules/asset-patterns.md`
- `.agents/rules/examples.md`

## Local AGENTS Files

Only a small set of local `AGENTS.md` files should remain:

- repo root
- `codegen/`
- `goud_engine/src/ffi/`
- `sdks/typescript/`
- `examples/`
