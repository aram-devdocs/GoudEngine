# RFCs

RFCs (Request for Comments) are the mechanism for proposing and deciding on significant changes to GoudEngine. Use an RFC when a change affects public API, architecture, cross-cutting concerns, or long-term direction.

Small bug fixes, refactors that do not change behavior, and documentation updates do not need RFCs.

## Numbering

RFCs use zero-padded 4-digit numbers: `RFC-0001`, `RFC-0002`, etc. Assign the next available number when opening the PR.

## File Format

Each RFC lives at `docs/rfcs/RFC-NNNN-short-title.md` and starts with YAML front matter:

```yaml
---
rfc: "0001"
title: "Short descriptive title"
status: draft
created: YYYY-MM-DD
authors: ["github-username"]
---
```

## Status Lifecycle

```
draft → proposed → accepted → implemented → superseded
```

| Status | Meaning |
|---|---|
| `draft` | Work in progress, not ready for review |
| `proposed` | PR open, ready for review |
| `accepted` | PR merged with review approval |
| `implemented` | Code is shipped; RFC is complete |
| `superseded` | Replaced by a later RFC (link to successor) |

Acceptance requires at least one PR review approval from a maintainer. The `proposed` → `accepted` transition is automated: a GitHub Action (`rfc-approve.yml`) updates the front matter status when the PR merges. The `implemented` and `superseded` transitions remain manual.

## Writing an RFC

1. Copy the template: `docs/rfcs/RFC-0000-template.md`
2. Assign the next number and rename the file
3. Fill in: motivation, detailed design, drawbacks, alternatives considered
4. Open a PR; set `status: proposed` in the front matter
5. Address review feedback on the PR
6. On merge: `rfc-approve.yml` automatically sets `status: accepted`; update the index below

## Index

| RFC | Title | Status |
|---|---|---|
| [RFC-0001](RFC-0001-provider-trait-pattern.md) | Provider Trait Pattern | proposed |
