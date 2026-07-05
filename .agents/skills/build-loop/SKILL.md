---
name: build-loop
description: Pick the fastest build/verify command for the change at hand, from cargo check to the full verify gate
user-invocable: true
---

# Build Loop

Match the command to the size of the change. Reserve the heavy pipelines for when you
actually need them so the inner loop stays fast.

## When to Use

Any time you are iterating on a change and deciding what to run to check it.

## The Ladder (fastest first)

1. **`cargo check -p <crate>`** — fast type/borrow check of one crate while editing Rust.
   Use `goud-engine-core` or `goud-engine` for engine work. No linking, no tests.
   Follow with `cargo clippy -p <crate> -- -D warnings` before you consider it done —
   clippy warnings are errors in the gate.
2. **`./build.sh`** — full workspace build plus artifact staging: copies the generated
   C header into the SDK include dirs and the native library into the C# runtime folders,
   then builds the C# SDK. Defaults to debug; `--release`/`--prod` optimizes. Scope flags:
   `--core-only`, `--host-runtime-only`, `--skip-csharp-sdk-build`, `--local`. Use when you
   need runnable SDK artifacts, not just a compile.
3. **`./codegen.sh`** — only when you changed FFI exports, `codegen/goud_sdk.schema.json`,
   or `codegen/ffi_mapping.json`. Regenerates all ten SDKs from the schema. See the
   `codegen-pipeline` skill.
4. **`./dev.sh --game <name>`** — build and run an example end to end. Add `--sdk <lang>`
   to pick the SDK (csharp is the default). Examples:
   `./dev.sh --game flappy_goud`, `./dev.sh --sdk python --game flappy_bird`,
   `./dev.sh --sdk typescript --game flappy_bird_web`. Use to confirm real runtime behavior.

## The Verify Gate

`scripts/verify.sh` is the canonical gate — one step table drives the local git hooks
and CI, so "passes locally" means "passes CI".

- **`scripts/verify.sh --staged`** — fast pre-commit subset (fmt, clippy, lint-layers,
  line limits, AI-config, AGENTS.md, gate parity). A strict subset of the full gate. Run
  this before every commit.
- **`scripts/verify.sh`** — full pre-push / CI gate: adds `clippy --all-targets`, the
  workspace build, `cargo test`, SDK tests, `cargo deny`, the codegen drift check, the
  TypeScript typecheck, and advisory doc/security lints.
- **`scripts/verify.sh --lane <rust|arch|meta|codegen|sdk|docs>`** scopes to one lane;
  **`scripts/verify.sh --list`** prints the step table as TSV.

## Steps (typical inner loop)

1. Edit Rust → `cargo check -p goud-engine-core` (and `cargo clippy -p ...`).
2. Need SDK artifacts or an example run → `./build.sh` or `./dev.sh --game <name>`.
3. Changed FFI/schema → `./codegen.sh`.
4. Before commit → `scripts/verify.sh --staged`.
5. Before push / PR → `scripts/verify.sh`.

## Verification

Green `scripts/verify.sh --staged` is the minimum bar for a commit; green full
`scripts/verify.sh` is the bar for a push. If the full gate passes locally, CI passes.
