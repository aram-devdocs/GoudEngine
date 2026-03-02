# AI Contributors

This document defines conventions for autonomous AI agents working on GoudEngine.

## Agent Identity

- **@claude** — interactive assistant, triggered via mentions in issues/PRs (handled by `claude.yml`)
- **@goudengine-agent** — autonomous agent, picks up `agent-ready` issues and opens PRs (handled by `agent.yml`)

## Branch Naming

Agents create branches following this pattern:

```
agent/issue-<number>-<short-slug>
```

Examples:
- `agent/issue-42-add-sprite-rotation`
- `agent/issue-105-fix-camera-bounds`

## Commit Format

Agents use conventional commits:

```
feat: add sprite rotation support

Closes #42
```

Prefixes: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`

## What Agents Can Do

- Read all repository files
- Create and push branches with `agent/` prefix
- Modify source code (`.rs`, `.cs`, `.py`, `.ts`)
- Run build and test commands (`cargo build`, `cargo test`, `cargo clippy`, etc.)
- Open pull requests with descriptions
- Comment on issues

## What Agents Cannot Do

- Push directly to `main`
- Modify protected infrastructure files (see below)
- Merge pull requests (requires human approval)
- Delete branches, issues, or PRs
- Publish packages (`cargo publish`, `dotnet nuget push`, etc.)
- Modify GitHub Actions workflows or CI configuration

## Protected Paths

These files require human review and cannot be modified by agents:

- `.github/` — workflows, CODEOWNERS, issue templates
- `CLAUDE.md` — agent instructions
- `AI_CONTRIBUTORS.md` — this file
- `release-please-config.json` — release configuration
- `.release-please-manifest.json` — version manifest

## Label Lifecycle

1. **`agent-ready`** — issue is queued for agent pickup
2. **`agent-working`** — agent has started work (automatic swap)
3. **`agent-blocked`** — agent failed, needs human help

## Error Recovery

If an agent gets blocked:

1. Check the failed workflow run linked in the issue comment
2. Fix the underlying problem (missing context, ambiguous requirements, etc.)
3. Remove the `agent-blocked` label
4. Add `agent-ready` to re-queue the issue

## Pipeline Flow

```
Issue created → milestone assigned → phase gate labels "agent-ready"
  → agent picks up → creates branch → implements → opens PR
  → CI passes → human reviews → merges
  → subtask cascade labels next sibling "agent-ready"
  → phase gate checks milestone completion → advances to next phase
```
