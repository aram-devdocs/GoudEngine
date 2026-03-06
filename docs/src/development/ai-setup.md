# AI Agent Setup

This repository includes configuration for AI coding assistants (Claude Code, Codex, Cursor, Gemini) with shared infrastructure across tools.

## Directory Structure

```
.agents/              # Shared cross-tool configuration (source of truth)
├── rules/            # Coding/domain rules (dependency hierarchy, FFI, TDD, etc.)
└── skills/           # Cross-tool skills (shared between Claude, Codex, Cursor, Gemini)
    ├── subagent-driven-development/
    ├── review-changes/
    ├── code-review/
    ├── gh-issue/
    ├── hardening-checklist/
    ├── tdd-workflow/
    ├── sdk-parity-check/
    └── ...

.claude/              # Claude Code configuration
├── agents/           # Subagent definitions (implementer, debugger, reviewers, etc.)
├── rules/            # -> symlinks to .agents/rules/
├── hooks/            # Lifecycle hooks (quality checks, secret scanning, session state)
├── skills/           # -> symlinks to .agents/skills/
├── memory/           # Session state (gitignored)
├── specs/            # Feature specs for multi-session work
└── settings.local.json

.codex/               # OpenAI Codex configuration
└── config.toml       # Agent roles pointing to shared .agents/rules/

.cursor/              # Cursor IDE configuration
├── rules/            # Cursor-specific contextual rules (.mdc files)
└── skills/           # -> symlink to .agents/skills/
```

## Key Files

| File | Purpose |
|------|---------|
| `AGENTS.md` | Root agent instructions (commands, architecture, anti-patterns) |
| `CLAUDE.md` | Symlink to `AGENTS.md` (Claude Code compatibility) |
| `GEMINI.md` | Symlink to `AGENTS.md` (Gemini compatibility) |
| `.cursorignore` | Excludes build artifacts from Cursor indexing |

## Distributed AGENTS.md

Each subdirectory with non-trivial logic has its own `AGENTS.md` providing module-specific context to agents working in that area. A `CLAUDE.md` symlink exists alongside each for Claude Code compatibility. Key locations:

- `goud_engine/AGENTS.md` -- engine core patterns
- `goud_engine/src/ffi/AGENTS.md` -- FFI boundary rules
- `sdks/AGENTS.md` -- SDK development rules
- `codegen/AGENTS.md` -- codegen pipeline details
- `examples/AGENTS.md` -- example game conventions

## Adding New Skills

Skills live at `.agents/skills/<skill-name>/SKILL.md`. They are available to both Claude Code and Cursor through symlinks at `.claude/skills/` and `.cursor/skills/`.

To add a skill:

1. Create `.agents/skills/<skill-name>/SKILL.md`
2. The symlinks pick it up automatically -- no further configuration needed.
