# Skills

Skills are reusable, task-scoped playbooks for agents working in this repo. Each skill
lives in its own directory under `.agents/skills/<skill-name>/` with a `SKILL.md` at its
root. The canonical location is `.agents/skills/`; it is symlinked into `.claude/skills/`
and `.cursor/skills/` so every tool sees the same set.

Use `/find-skills` to list what exists. This document is the authoring standard for new
skills.

## Frontmatter Contract

Every `SKILL.md` opens with a YAML frontmatter block:

```yaml
---
name: my-skill
description: One line saying what the skill does and when it helps
user-invocable: true
---
```

- **`name` is required and MUST equal the directory name.** `.agents/skills/my-skill/`
  must declare `name: my-skill`.
- **`description` is required and non-empty.** One line; it is what `/find-skills` and the
  skill picker show.
- Optional keys already in use: `user-invocable`, `context: fork`, `argument-hint`,
  `disable-model-invocation`.

`scripts/validate-skills.py` enforces the contract in CI (frontmatter present, `name` and
`description` non-empty, `name == dir`).

## Naming

Skill directory and `name` are **kebab-case**: `codegen-pipeline`, `wasm-web`,
`release-process`. Lowercase, hyphen-separated, no spaces or underscores.

## Required Sections

A skill body should be concise and cover, at minimum:

- **When to Use** — the trigger. When does an agent reach for this skill, and when not.
- **Steps** — the ordered procedure to follow.
- **Verification** — how to confirm the work is actually done (the command to run, the
  state to check).

Add scenario checklists, troubleshooting, or reference tables as the task warrants, but
keep the skill focused on one job.

## Only Reference Files That Exist

If a skill mentions a resource path — `scripts/...`, `references/...`, `assets/...`, or
`tests/...` — that path MUST exist, either relative to the skill directory or relative to
the repo root. `scripts/validate-skills.py` checks every such reference and fails the
build on a broken one, so keep paths honest and update them when files move. Skill-local
resources go in `references/`, `assets/`, `scripts/`, or `tests/` inside the skill dir.

## Ship Tests for Scripts

If a skill ships an executable helper (in its `scripts/` dir), ship a test for it in the
skill's `tests/` dir. Scripts that agents run on autopilot need the same verification bar
as engine code — a helper that silently breaks is worse than no helper.

## Adding a Skill

1. Create `.agents/skills/<skill-name>/SKILL.md` with valid frontmatter (`name` == dir).
2. Write the When to Use / Steps / Verification sections.
3. Put any shipped resources under the skill dir; reference only paths that exist.
4. Run `python3 scripts/validate-skills.py` — it must pass.
5. Add the skill to the `/find-skills` table.
