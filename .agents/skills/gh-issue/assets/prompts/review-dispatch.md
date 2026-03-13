Working directory: <absolute worktree path>
Diff base: origin/main

Check only the gate you were dispatched for:
- `spec-reviewer`: requirements coverage and behavioral completeness
- `code-quality-reviewer`: code quality, anti-patterns, and test quality
- `architecture-validator`: layer boundaries and dependency direction
- `security-auditor`: FFI safety, unsafe handling, and ownership boundaries

Requirements:
- Cite exact files and lines for every issue.
- End with the gate verdict.
- Use this report structure:
  - Task understanding
  - Findings
  - Verification examined
  - Residual risks
  - Next required action
