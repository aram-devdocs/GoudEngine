# Engineering Constitution

This page separates the practices that do not change from the tools that do. When
a tool in the substitutable column conflicts with a practice in the permanent
column, the practice wins and the tool is replaced.

## Permanent practices

These hold regardless of language, framework, or vendor. Each links to where it
is enforced.

| Practice | What it means | Enforced by |
|---|---|---|
| Layered dependencies | The engine is a strict 5-layer hierarchy (Foundation → Libs → Services → Engine → FFI). Dependencies flow down only; no cycles. | `tools/lint_layers.rs` (`cargo run -p lint-layers`); `.agents/rules/dependency-hierarchy.md` |
| FFI safety | Every export is `#[no_mangle] extern "C"`, every pointer is null-checked, every `unsafe` block has a `// SAFETY:` comment, shared types are `#[repr(C)]`. | `.agents/rules/ffi-patterns.md`; `codegen/validate_c_header.py`; clippy `-D warnings` |
| Generated code is single-source | SDK bindings are generated from the FFI surface. Never hand-edit `*.g.rs`, `*.g.cs`, `*.g.ts`, or `generated/` files. | `./codegen.sh`; `scripts/check-generated-artifacts.sh`; `docs/adr/0002-generated-code-single-source-of-truth.md` |
| One verification gate | A single data-driven pipeline runs at pre-commit, pre-push, and CI so "passes locally" means "passes CI". Bypassing it is not allowed. | `scripts/verify.sh`; `scripts/check-gate-parity.py` |
| Review before merge | Substantive changes get a review pass; FFI/unsafe/ownership changes get a security pass. Reviewers raise a concrete, located concern or justify none, and end with an explicit verdict. | `.agents/rules/challenge-protocol.md`; the review-verdict hook |
| Rust-first | Engine behavior lives in Rust; SDKs are thin FFI wrappers. Logic found in an SDK moves to Rust. | `.agents/rules/sdk-development.md` |

## Substitutable tooling

These are current choices, swappable when a better fit appears. Swapping one does
not touch the practices above.

| Area | Current choice |
|---|---|
| Engine language | Rust |
| Default render backend | wgpu (OpenGL 3.3 as a legacy fallback) |
| SDK languages | C#, Python, TypeScript, Go, Swift, C, C++, Kotlin, Lua, Rust |
| Release automation | release-please |
| Docs | mdBook |
| Agent/hook tooling | the `.agents/` catalog and `.claude/` runtime |

## The tie-breaker

Adopting a standard means adding it enforcer-first: wire the check, then document
it. Removing a tool means checking that no permanent practice depended on it, then
replacing the enforcement. A doc that disagrees with the code or a validator is
wrong — fix the doc.
