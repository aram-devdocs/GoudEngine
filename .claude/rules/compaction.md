---
alwaysApply: true
---

# Session Compaction Protocol

When context compaction occurs (or is about to occur), preserve session state so work can resume without loss.

## Before Compaction (PreCompact)

Write a snapshot to `.claude/memory/SESSION.md` containing:

- **Branch**: current git branch name
- **Active task**: what you're working on right now
- **Files modified**: list of files changed in this session
- **Next steps**: what remains to be done
- **Blockers**: anything preventing progress
- **Decisions made**: key choices and their rationale

Keep SESSION.md under 200 lines. Overwrite rather than append — it's a current-state snapshot, not a log.

## After Compaction

Read `.claude/memory/SESSION.md` to restore context. Also check:

- `git status` for uncommitted changes
- `git log -5 --oneline` for recent commits
- Any active spec files in `.claude/specs/`

## MEMORY.md vs SESSION.md

- SESSION.md is auto-managed by compaction hooks
- MEMORY.md is for manual long-lived context that persists across sessions
- Both live in `.claude/memory/` (gitignored)
