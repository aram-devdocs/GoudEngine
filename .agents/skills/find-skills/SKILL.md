---
name: find-skills
description: Discover and list available skills in the repository
user-invocable: true
---

# Find Skills

Discover all available skills in the GoudEngine repository and present them with descriptions and usage instructions.

## When to Use

Run when you need to know what skills are available, when starting a new session, or when unsure which skill applies to a task.

## Skill Locations

Skills are stored in `.agents/skills/` and symlinked to tool-specific directories:

```
.agents/skills/          # Canonical location (cross-tool)
.claude/skills/          # Symlink for Claude Code
.cursor/skills/          # Symlink for Cursor
```

## Discovery Process

1. Search for all `SKILL.md` files:

```bash
find .agents/skills/ -name "SKILL.md" -type f 2>/dev/null
```

2. For each skill, read the YAML frontmatter to extract:
   - `name`: Skill identifier
   - `description`: One-line summary
   - `user-invocable`: Whether it can be called directly
   - `context`: How it runs (fork, inline)

3. Present the results as a table.

## Available Skills

| Skill | Description | Invocable |
|-------|-------------|-----------|
| `/subagent-driven-development` | Orchestrate parallel subagent batches with two-stage review | Yes |
| `/review-changes` | Dispatch 5 parallel review agents to analyze pending changes | Yes |
| `/code-review` | 7-phase structured code review | Yes |
| `/hardening-checklist` | 12-area audit checklist for project health | Yes |
| `/tdd-workflow` | RED-GREEN-REFACTOR TDD pipeline with agent dispatch | Yes |
| `/integration-testing` | Integration test patterns for Rust engine with GL context and FFI | Yes |
| `/architecture-review` | Dependency flow audit and 5-layer hierarchy validation | Yes |
| `/humanizer` | Remove AI writing patterns from documentation | Yes |
| `/find-skills` | Discover available skills (this skill) | Yes |
| `/sdk-parity-check` | Verify FFI exports have matching C# and Python SDK wrappers | Yes |
| `/session-continuity` | Manage session state across context compactions | Yes |
| `/goudengine-debugging` | Debugging workflow and diagnostic checklists for runtime via MCP tools | Yes |
| `/goudengine-mcp-server` | Setup, tool reference, and troubleshooting for the MCP debugger server | Yes |

## Matching Skills to Tasks

| Task Type | Recommended Skill |
|-----------|-------------------|
| Implementing a new feature | `/subagent-driven-development` + `/tdd-workflow` |
| Reviewing before commit | `/review-changes` |
| Deep PR review | `/code-review` |
| Project health check | `/hardening-checklist` |
| Writing tests | `/tdd-workflow` + `/integration-testing` |
| Checking module structure | `/architecture-review` |
| Writing documentation | `/humanizer` |
| Adding FFI functions | `/sdk-parity-check` |
| Starting/resuming a session | `/session-continuity` |
| Debugging a running game | `/goudengine-debugging` |
| Setting up MCP server | `/goudengine-mcp-server` |
| Finding a skill | `/find-skills` (this one) |

## Adding New Skills

To add a new skill:

1. Create `.agents/skills/<skill-name>/SKILL.md`
2. Include YAML frontmatter (name, description, user-invocable)
3. Symlinks in `.claude/skills/` and `.cursor/skills/` will pick it up automatically
4. Update this skill's "Available Skills" table
