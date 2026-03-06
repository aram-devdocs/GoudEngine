# docs/rfcs/ -- Request for Comments

## Purpose

Design documents for significant architectural decisions in GoudEngine. Each RFC
goes through a review process before implementation begins.

## Key Files

- `README.md` -- RFC process documentation, numbering scheme, status lifecycle, index
- `RFC-0000-template.md` -- Template for new RFCs; copy and rename to start a new one
- `RFC-NNNN-short-title.md` -- Individual RFC documents with YAML front matter

## RFC Process

1. Create a new RFC file with the next available number
2. Set `status: draft` in front matter while writing
3. Open a PR and set `status: proposed`
4. On merge with review approval, set `status: accepted`
5. After implementation ships, set `status: implemented`

## When Working Here

- RFCs are design documents, not implementation code
- No `.rs`, `.cs`, or `.py` files belong in this directory
- Follow the front matter schema defined in `README.md`
- Reference existing engine code by repo-relative path (e.g., `goud_engine/src/libs/...`)
- Keep each RFC under 500 lines
- Use direct, technical prose -- no marketing language
