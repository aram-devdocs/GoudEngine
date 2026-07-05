# 0002 — Generated Code Is the Single Source of Truth

Status: Accepted

## Context

Ten language SDKs wrap the same engine over FFI. If each SDK's bindings are
written by hand, they drift from the FFI surface and from each other, and
parity becomes a manual audit that is never complete.

## Decision

SDK bindings are generated. The codegen pipeline in `codegen/` reads the FFI
surface (`goud_engine/src/ffi/`) as the single source of truth and emits the
bindings for every SDK. The FFI is the authority; the generated files are its
projection.

Generated files MUST NOT be hand-edited. This covers `*.g.rs`, `*.g.cs`,
`*.g.ts`, and everything under `generated/` directories. To change a binding,
change the FFI (and the schema where applicable) and rerun codegen.

## Consequences

- Parity across SDKs is produced by construction, not by review.
- A hand-edit to a generated file is a defect: the next codegen run overwrites
  it, and the change is lost.
- Adding or changing an FFI export is a coordinated step: update Rust, FFI,
  schema, and rerun codegen so all SDKs move together.
- Coverage tooling (`codegen/validate_coverage.py`) can assert that every FFI
  export is projected into every SDK, because there is a single generator to
  hold accountable.
