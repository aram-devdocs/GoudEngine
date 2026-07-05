# External Standards

GoudEngine builds on established external standards instead of inventing local
conventions. Each standard below is paired with the gate or tool that enforces
or cites it, so a claim of compliance maps to something that runs.

| Standard | Governs | Enforcer / citation |
| --- | --- | --- |
| [SemVer](https://semver.org/) | Version numbers and compatibility promises | release-please (`release-please-config.json`) |
| [Conventional Commits](https://www.conventionalcommits.org/) | Commit message and PR title format | commit-msg hook (`.husky/hooks/commit-msg`) + PR-title check (`.github/workflows/pr-validation.yml`) |
| [Keep a Changelog](https://keepachangelog.com/) | Changelog structure and grouping | release-please (generates `CHANGELOG.md`) |
| [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119) | MUST/SHOULD/MAY keywords in normative docs | Instruction and rule docs (`.agents/rules/`, `CLAUDE.md`) |
| [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) | Idiomatic public Rust API surface | `cargo clippy -- -D warnings` |
| C-ABI / FFI safety | The `extern "C"` boundary and generated C headers | FFI rules (`.agents/rules/ffi-patterns.md`) + `scripts/validate_c_header.py` |

## Rules

- **Link the canonical source; never restate it.** A row cites the upstream
  standard and names the gate. It does not paraphrase the standard's contents,
  which drift out of date the moment the standard changes.
- **Adopt enforcer-first.** Adding a standard means adding a row only once a
  gate or tool enforces or cites it. A standard with no enforcer is a wish, not
  a practice; wire the check first, then record the row.
