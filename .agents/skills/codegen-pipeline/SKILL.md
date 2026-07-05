---
name: codegen-pipeline
description: Run and troubleshoot the SDK code generation pipeline (./codegen.sh) and the generated-artifact drift gate
user-invocable: true
---

# Codegen Pipeline

All ten language SDKs are generated from one source of truth: `codegen/goud_sdk.schema.json`
plus the FFI manifest that the Rust build extracts. `./codegen.sh` regenerates every SDK,
and CI fails the build if the checked-in generated files drift from what the pipeline
would produce.

## When to Use

Run this whenever you change any of:

- FFI exports under `goud_engine/src/ffi/` (or the WASM exports under `goud_engine/src/wasm/`)
- `codegen/goud_sdk.schema.json` (the universal SDK schema)
- `codegen/ffi_mapping.json` (maps schema entries to FFI symbols)
- Any generator under `codegen/` or the C header template

If you touched none of those, you do not need to run codegen.

## Never Hand-Edit Generated Files

Files ending in `.g.rs`, `.g.cs`, `.g.ts`, `.g.hpp`, and the copied `goud_engine.h`
headers are output. Editing them directly is always wrong — the next `./codegen.sh`
run overwrites the change and the drift gate rejects it in between. Change the Rust
FFI, the schema, and `ffi_mapping.json` together, then regenerate.

## Steps

1. Make the Rust/FFI/schema change.
2. From the repo root, run `./codegen.sh`. The pipeline (16 stages) builds the Rust
   engine to extract a fresh `codegen/ffi_manifest.json` and C header, validates FFI
   coverage (`codegen/validate_coverage.py`), checks layer dependencies
   (`cargo run -p lint-layers`), then regenerates C, C++, C#, Python, Go, TypeScript
   (node + web), Swift, Lua, and Kotlin.
3. Review the diff. Every regenerated SDK file should reflect your API change and
   nothing else.
4. Run the affected SDK smoke tests (see `.agents/rules/sdk-development.md`), e.g.
   `python3 sdks/python/test_bindings.py`, `dotnet test sdks/csharp.tests/`,
   `cd sdks/typescript && npm test`.
5. Commit the source change and the regenerated files in the same commit.

## Verification

- `scripts/check-generated-artifacts.sh` reproduces the CI drift gate: it confirms every
  expected generated artifact is present, revalidates the C header, and runs
  `git diff --exit-code` over the tracked generated paths. A clean exit means no drift.
- The same check runs in the full verify gate as the `codegen-drift` step
  (`scripts/verify.sh`, lane `codegen`).

## Troubleshooting Drift

- **Drift gate fails after your change** — you edited a generated file by hand, or you
  changed Rust/schema without rerunning `./codegen.sh`. Rerun it and commit the output.
- **`codegen.sh` aborts at stage 2** — the Rust build failed, so no fresh manifest was
  produced. Fix the compile error first (`cargo build -p goud-engine-core -p goud-engine`).
- **FFI coverage gap** — a new FFI export has no entry in `codegen/ffi_mapping.json`.
  Add the mapping, then rerun.
- **Schema mismatch at the final stage** — `codegen/validate.py` found the schema and
  generated output disagree; reconcile `goud_sdk.schema.json` with the FFI change.
- **Untracked generated artifacts detected** — a generator emitted a new file that is
  not committed. Add it to git.
