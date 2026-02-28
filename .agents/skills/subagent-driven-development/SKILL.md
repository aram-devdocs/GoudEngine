---
name: subagent-driven-development
description: Orchestrate parallel subagent batches with two-stage review for GoudEngine tasks
context: fork
user-invocable: true
---

# Subagent-Driven Development

Three-tier agent orchestration with challenge protocol and hook-enforced governance.

## When to Use

Invoke this skill for any non-trivial implementation task that touches multiple files or modules. Single-file fixes use `quick-fix` directly — no orchestration needed.

## Three-Tier Dispatch Model

```
Tier 0: ORCHESTRATOR (you)
  ├── engine-lead     — Rust core, graphics, ECS, assets
  ├── integration-lead — FFI, C# SDK, Python SDK
  └── quality-lead    — reviews, testing, validation
```

The orchestrator dispatches **team leads**, not individual specialists. Team leads manage their own specialist waves internally.

## Workflow

### 1. Assess the Task

Classify the work:
- **Engine-only**: Dispatch engine-lead
- **FFI + SDK**: Dispatch integration-lead (or engine-lead + integration-lead if Rust core also changes)
- **Cross-cutting**: Dispatch engine-lead AND integration-lead in parallel, then quality-lead for review
- **Trivial fix**: Dispatch quick-fix (haiku) directly

### 2. Dispatch Team Leads

Team leads receive the objective and decompose it independently. They:
- Explore relevant code
- Dispatch specialists (parallel if independent, sequential if dependent)
- Question specialist output before reporting
- Run verification (cargo check, cargo test)
- Report concise summary

### 3. Team Lead Internal Waves

Each team lead manages their own wave plan. Example for engine-lead:

```
Internal Wave 1 (parallel):
  - implementer: Rust core implementation
  - test-first-implementer: Write failing tests

Internal Wave 2 (sequential):
  - test-runner: Verify tests pass
  - debugger: If tests fail, diagnose
```

### 4. Quality Gate (via quality-lead)

After implementation is complete, dispatch quality-lead:

```
Quality Wave 1:
  - spec-reviewer (MUST run first, MUST APPROVE before next step)

Quality Wave 2:
  - code-quality-reviewer (only after spec-reviewer APPROVED)
  - architecture-validator (parallel with code-quality-reviewer)

Quality Wave 3 (conditional):
  - security-auditor (only if FFI/unsafe touched, ALWAYS sequential)
```

The spec-reviewer -> code-quality-reviewer gate is hook-enforced (review-gate-guard.sh).

### 5. Challenge Protocol

Every agent at every tier:
- **Implementers**: List assumptions before implementing, verify after
- **Reviewers**: Identify at least one concern, no rubber-stamping, end with verdict
- **Team leads**: Question specialist output, apply own judgment
- **Orchestrator**: Question team lead reports before accepting

This is injected automatically via challenge-injector.sh (SubagentStart hook).

### 6. Verification

After all team leads report:

```bash
cargo test                              # All Rust tests
cargo clippy -- -D warnings             # Lint check
python3 sdks/python/test_bindings.py    # Python SDK tests
```

## Model Tier Selection

| Tier | Model | Agents |
|------|-------|--------|
| Quick | haiku | quick-fix |
| Standard | sonnet | implementer, ffi-implementer, sdk-implementer, test-first-implementer, spec-reviewer, code-quality-reviewer, architecture-validator, test-runner |
| Complex | opus | engine-lead, integration-lead, quality-lead, security-auditor, debugger |

## Governance (Hook-Enforced)

| Rule | Hook | Enforcement |
|------|------|-------------|
| Orchestrator cannot write .rs/.cs/.py | delegation-guard.sh | HARD BLOCK |
| spec-reviewer before code-quality-reviewer | review-gate-guard.sh | HARD BLOCK |
| Reviewers must produce a verdict | review-verdict-validator.sh | HARD BLOCK |
| Challenge protocol injected | challenge-injector.sh | DETERMINISTIC |
| Delegation audit trail | delegation-tracker.sh | DETERMINISTIC |
| Governance violations block session end | governance-completion-check.sh | HARD BLOCK |

## Batch Size Limits

- Maximum 3-5 tasks per parallel batch within a team lead
- Security-sensitive work (FFI, unsafe) is ALWAYS sequential
- Review agents run sequentially (spec-reviewer THEN code-quality-reviewer)

## Anti-Patterns

- Orchestrator implementing directly (hook-blocked)
- Dispatching specialists directly instead of team leads (for non-trivial work)
- Running code-quality-reviewer before spec-reviewer (hook-blocked)
- Running security-auditor in parallel with other agents
- Rubber-stamping reviews without analysis (hook-blocked)
- Batches larger than 5 tasks
- Team leads passing specialist output through uncritically
