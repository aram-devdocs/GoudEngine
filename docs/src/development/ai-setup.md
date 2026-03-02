# AI Agent Setup

This repository includes configuration for AI coding assistants (Claude Code, Cursor, Gemini) with shared infrastructure across tools.

## Directory Structure

```
.claude/              # Claude Code configuration
├── agents/           # Subagent definitions (implementer, debugger, reviewers, etc.)
├── rules/            # Contextual rules (FFI patterns, SDK development, TDD, etc.)
├── hooks/            # Lifecycle hooks (quality checks, secret scanning, session state)
├── skills/           # -> symlink to .agents/skills/
├── memory/           # Session state (gitignored)
├── specs/            # Feature specs for multi-session work
└── settings.local.json

.agents/skills/       # Cross-tool skills (shared between Claude, Cursor, Gemini)
├── subagent-driven-development/
├── review-changes/
├── code-review/
├── hardening-checklist/
├── tdd-workflow/
├── sdk-parity-check/
└── ...

.cursor/              # Cursor IDE configuration
├── rules/            # Cursor-specific contextual rules (.mdc files)
└── skills/           # -> symlink to .agents/skills/
```

## Key Files

| File | Purpose |
|------|---------|
| `CLAUDE.md` | Root agent instructions (commands, architecture, anti-patterns) |
| `AGENTS.md` | Symlink to `CLAUDE.md` (Copilot/Cursor compatibility) |
| `GEMINI.md` | Symlink to `CLAUDE.md` (Gemini compatibility) |
| `.cursorignore` | Excludes build artifacts from Cursor indexing |

## Distributed CLAUDE.md

Each subdirectory with non-trivial logic has its own `CLAUDE.md` providing module-specific context to agents working in that area. Key locations:

- `goud_engine/CLAUDE.md` — engine core patterns
- `goud_engine/src/ffi/CLAUDE.md` — FFI boundary rules
- `sdks/CLAUDE.md` — SDK development rules
- `codegen/CLAUDE.md` — codegen pipeline details
- `examples/CLAUDE.md` — example game conventions

## Adding New Skills

Skills live at `.agents/skills/<skill-name>/SKILL.md`. They are available to both Claude Code and Cursor through symlinks at `.claude/skills/` and `.cursor/skills/`.

To add a skill:

1. Create `.agents/skills/<skill-name>/SKILL.md`
2. The symlinks pick it up automatically — no further configuration needed.
