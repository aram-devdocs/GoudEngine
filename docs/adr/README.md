# Architecture Decision Records

An Architecture Decision Record (ADR) captures one significant architecture
decision, the context that forced it, and the consequences it commits the
project to.

## Format

Each ADR is a numbered file (`NNNN-short-slug.md`) with three sections:

- **Context** — the forces and constraints that made a decision necessary.
- **Decision** — what was decided, stated in the present tense.
- **Consequences** — what follows, including the costs and the things now
  ruled out.

Keep them short. An ADR is a record, not a design document.

## Immutability

An accepted ADR is immutable. Do not rewrite it to reflect a later change of
mind. When a decision is reversed or refined, write a new ADR that supersedes
the old one and note the supersession in both. The history of decisions is
itself information; editing it in place destroys that.

## When to Write One

Write an ADR when a decision:

- changes the ABI or the FFI boundary,
- changes a public API surface an SDK depends on,
- resolves a layer-boundary question (what may depend on what),
- adds or removes a verification gate, or
- changes the project's security posture.

Routine changes do not need an ADR.

## ADR vs RFC

ADRs record architecture decisions already made. Feature designs, which are
proposals under discussion, live in `docs/src/rfcs/`. An RFC argues for a
direction; an ADR records the direction that was taken.

## Index

- [0001 — Canonical verify pipeline](0001-canonical-verify-pipeline.md)
- [0002 — Generated code is the single source of truth](0002-generated-code-single-source-of-truth.md)
- [0003 — Canonical layer numbering](0003-canonical-layer-numbering.md)
