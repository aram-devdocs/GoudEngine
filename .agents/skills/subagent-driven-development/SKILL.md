---
name: subagent-driven-development
description: Orchestrate parallel subagent batches with two-stage review for GoudEngine tasks
context: fork
user-invocable: true
---

# Subagent-Driven Development

Bounded multi-agent orchestration with shared Claude/Codex policy and hook-enforced governance where available.

## When to Use

Invoke this skill for any non-trivial implementation task that touches multiple files or modules. Single-file fixes use `quick-fix` directly.

## Shared Team Shape

```
Tier 0: ORCHESTRATOR (you)
  ├── engine-lead      — one active Rust/core implementation workstream
  ├── integration-lead — one active FFI/SDK implementation workstream
  ├── direct reviewers — spec-reviewer -> code-quality-reviewer -> architecture-validator
  └── quality-lead     — exceptional audit-heavy path only
```

The orchestrator runs exactly one active implementation team at a time. A lead may use at most one specialist wave, capped at 2 specialists total for that batch.

## Workflow

### 1. Assess the Task

Classify the work:
- **Engine-only**: Dispatch engine-lead
- **FFI + SDK**: Dispatch integration-lead
- **Cross-cutting**: Dispatch the first implementation lead, wait for completion, then dispatch the next lead if still needed
- **Trivial fix**: Dispatch quick-fix directly

### 2. Dispatch Team Leads

Team leads receive the objective and decompose it independently. They:
- Explore relevant code
- Dispatch at most one specialist wave (max 2 specialists total)
- Keep dependent work sequential
- Question specialist output before reporting
- Run verification (cargo check, cargo test)
- Report concise summary

### 3. Team Lead Internal Waves

Each team lead manages one bounded wave plan. Example for engine-lead:

```
Specialist Wave (optional, bounded):
  - implementer: Rust core implementation
  - test-first-implementer: Write failing tests
Follow-up (sequential):
  - test-runner: Verify tests pass
  - debugger: If tests fail, diagnose
```

### 4. Default Review Gate (direct from root)

After implementation is complete, root dispatches reviewers directly:

```
Review 1:
  - spec-reviewer
Review 2:
  - code-quality-reviewer
Review 3:
  - architecture-validator
Review 4 (conditional):
  - security-auditor
```

The spec-reviewer -> code-quality-reviewer gate is hook-enforced (review-gate-guard.sh).

Use `quality-lead` only when the session genuinely needs an aggregated audit or a dedicated testing/review sub-orchestrator.

### 5. Challenge Protocol

Every agent at every tier:
- **Implementers**: List assumptions before implementing, verify after
- **Reviewers**: Identify at least one concern, no rubber-stamping, end with verdict
- **Team leads**: Question specialist output, apply own judgment
- **Orchestrator**: Question team lead reports before accepting

This is injected automatically via challenge-injector.sh (SubagentStart hook).

### 6. Verification

After implementation and direct reviews report:

```bash
cargo test                              # All Rust tests
cargo clippy -- -D warnings             # Lint check
python3 sdks/python/test_bindings.py    # Python SDK tests
```

## Failure Ladder

When agent dispatch fails due to capacity, timeout, or hang:
1. Stop nested dispatch immediately
2. Do not retry the same fan-out shape
3. Fall back to `quick-fix` for single-file work, or one direct worker/specialist from root
4. Continue the direct root review sequence after implementation completes

## Model Tier Selection

Provider-specific model assignments live in `.agents/agent-catalog.toml` and the generated wrappers.

| Tier | Agents | Use For |
|------|--------|---------|
| Fast | quick-fix, architecture-validator, explorer, monitor, domain experts | Single-file fixes, read-heavy scans, lightweight validation |
| Standard | implementer, ffi-implementer, sdk-implementer, test-first-implementer, spec-reviewer, code-quality-reviewer, test-runner, debugger, worker, documentation-writer | Implementation, reviews, testing, and debugging |
| High | engine-lead, integration-lead, quality-lead, security-auditor, default | Bounded sub-orchestration, security review, and highest-judgment work |

## Governance (Hook-Enforced)

| Rule | Hook | Enforcement |
|------|------|-------------|
| Orchestrator cannot write .rs/.cs/.py | delegation-guard.sh | HARD BLOCK |
| spec-reviewer before code-quality-reviewer | review-gate-guard.sh | HARD BLOCK |
| Reviewers must produce a verdict | review-verdict-validator.sh | HARD BLOCK |
| Challenge protocol injected | challenge-injector.sh | DETERMINISTIC |
| Delegation audit trail | delegation-tracker.sh | DETERMINISTIC |

## Batch Size Limits

- Maximum 1 specialist wave per lead
- Maximum 2 specialists total within that wave
- Security-sensitive work (FFI, unsafe) is ALWAYS sequential
- Root runs one implementation lead at a time
- Review agents run sequentially (spec-reviewer THEN code-quality-reviewer)

## Anti-Patterns

- Orchestrator implementing directly (hook-blocked)
- Running multiple implementation leads in parallel by default
- Retrying the same nested fan-out after a capacity failure
- Running code-quality-reviewer before spec-reviewer (hook-blocked)
- Running security-auditor in parallel with other agents
- Rubber-stamping reviews without analysis (hook-blocked)
- Specialist waves larger than 2 agents
- Team leads passing specialist output through uncritically
