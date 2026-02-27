---
name: session-continuity
description: Manage session state across context compactions and multi-session work
user-invocable: true
---

# Session Continuity

Preserve and restore working context across session boundaries, context compactions, and multi-session feature work.

## When to Use

- At the **start** of every session: restore context from MEMORY.md
- Before **context compaction**: save current state
- When **resuming** interrupted work: read specs and memory
- When **handing off** between sessions: write a clean checkpoint

## Infrastructure

```
.claude/
├── memory/           # gitignored — local session state
│   └── SESSION.md    # Current session snapshot
└── specs/            # git-tracked — shared feature specs
    └── *.md          # One spec per feature/task
```

## Session Start Protocol

When beginning a new session:

1. Read `.claude/memory/SESSION.md` if it exists
2. Read any active spec files in `.claude/specs/`
3. Check git state:

```bash
git branch --show-current
git log --oneline -5
git status --short
```

4. Summarize the recovered context before proceeding

## Pre-Compaction Checkpoint

Before context is compacted (PreCompact hook triggers this automatically):

Write `.claude/memory/SESSION.md` with:

```markdown
# Session Memory — GoudEngine

## Current State
- Branch: <current branch>
- Active task: <what you're working on>
- Last checkpoint: <timestamp>

## In Progress
- <file>: <what's being changed and why>
- <file>: <status>

## Next Steps
1. <immediate next action>
2. <following action>
3. <etc>

## Decisions Made
- <decision>: <rationale>

## Blockers
- <blocker description> (if any)
```

**Rules:**
- Keep under 200 lines (this is a snapshot, not a log)
- Overwrite on each save (not append)
- Include enough context that a fresh session can resume without re-reading the full codebase
- Reference specific file paths and line numbers where relevant

## Feature Specs

For multi-session features, create a spec file:

`.claude/specs/<feature-name>.md`

```markdown
# Feature: <name>

## Status: IN_PROGRESS | BLOCKED | COMPLETE

## Requirements
- <requirement 1>
- <requirement 2>

## Design Decisions
- <decision>: <rationale>

## Implementation Plan
- [ ] Step 1: <description>
- [x] Step 2: <completed step>
- [ ] Step 3: <description>

## Files Touched
- `path/to/file.rs` — <what changed>

## Open Questions
- <question>
```

Spec files are tracked in git so they survive across machines and collaborators.

## Recovery Protocol

If starting a session with no memory file:

1. Check git log for recent commits and their messages
2. Check for any open branches with uncommitted work
3. Check `.claude/specs/` for active feature specs
4. Ask the user what they'd like to work on

## Cleanup

When a feature is complete:

1. Move the spec file to a `completed/` subdirectory or delete it
2. Clear the relevant items from SESSION.md
3. Commit the spec change so collaborators see it's done
