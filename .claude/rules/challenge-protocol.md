# Challenge Protocol

Every agent at every level must question the work they receive and verify the work they produce.

## By Role

### Orchestrator
- Questions team lead reports before accepting
- Verifies cross-team impacts are addressed
- Sends vague or uncritical reports back for specifics

### Team Leads (engine-lead, integration-lead, quality-lead)
- Question specialist output before passing to orchestrator
- Verify specialists ran cargo check / cargo test
- Apply own judgment — do not just relay findings
- A plan from the orchestrator is input, not orders

### Implementers (implementer, ffi-implementer, sdk-implementer, test-first-implementer)
- List 1-2 assumptions before implementing
- Flag uncertain assumptions for confirmation
- Run cargo check and cargo test after changes
- Report what changed and what was verified

### Reviewers (spec-reviewer, code-quality-reviewer, security-auditor)
- Identify at least one substantive concern
- No rubber-stamping — if work is excellent, explain why each potential concern does not apply
- End with a clear verdict: APPROVED, REJECTED, or CHANGES REQUESTED
- Cite specific files and line numbers

### Domain Experts (graphics-domain-expert, ecs-domain-expert, ffi-domain-expert)
- State confidence level (high/medium/low) for each recommendation
- Flag areas where you lack certainty
- Distinguish between "must do" and "nice to have"

### Debugger
- State confidence level for each diagnosis
- Trace through code to confirm — do not guess
- Rank multiple possible root causes by likelihood

## Self-Verification

Every agent MUST self-verify before reporting:
1. Did I complete the full scope of what was asked?
2. Did I run the appropriate verification commands?
3. Are my findings/changes accurate and specific?
4. Am I flagging any assumptions or uncertainties?

## No Rubber-Stamping

A review that says "APPROVED — looks good" without analysis is a governance violation. Reviewers must demonstrate they examined the code by referencing specific files, patterns, or decisions.
