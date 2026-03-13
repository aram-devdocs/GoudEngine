import json
import subprocess
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(".agents/skills/gh-issue/scripts/gh_issue_workflow.py").resolve()
FIXTURES = Path(".agents/skills/gh-issue/tests/fixtures").resolve()


class WorkflowCliTests(unittest.TestCase):
    def run_cli(self, *args, expected=0):
        result = subprocess.run(["python3", str(SCRIPT), *args], text=True, capture_output=True)
        self.assertEqual(result.returncode, expected, msg=result.stderr or result.stdout)
        return json.loads(result.stdout)

    def init_run(self, repo_root: Path, *, mode: str = "worktree") -> Path:
        working_directory = repo_root if mode == "in-place" else repo_root / "worktrees/issue-101-fix-bug"
        payload = self.run_cli(
            "init-run",
            "--repo-root",
            str(repo_root),
            "--primary",
            "101",
            "--slug",
            "fix-bug",
            "--branch",
            "codex/issue-101-fix-bug",
            "--mode",
            mode,
            "--working-directory",
            str(working_directory),
            "--main-repo-path",
            str(repo_root),
            "--issues",
            "101",
            "205",
            "--issue-title",
            "#101 fix rendering bug",
            "--issue-title",
            "#205 clean shader error path",
        )
        return Path(payload["run_dir"])

    @staticmethod
    def find_todo(state: dict, todo_id: str) -> dict:
        return next(todo for todo in state["todos"] if todo["id"] == todo_id)

    def test_init_run_creates_strict_plan_and_state(self):
        with tempfile.TemporaryDirectory() as tmp:
            run_dir = self.init_run(Path(tmp))
            plan = (run_dir / "plan.md").read_text()
            state = json.loads((run_dir / "state.json").read_text())
            self.assertIn("## Claude Review Loop", plan)
            self.assertIn("## CI Loop", plan)
            self.assertTrue(state["naming"]["branch_valid"])
            self.assertEqual(state["phase"], "bootstrapped")
            self.assertEqual(state["review_gates"]["spec-reviewer"], "pending")
            self.assertEqual(self.find_todo(state, "bootstrap")["status"], "done")

    def test_init_run_rejects_invalid_branch_name(self):
        with tempfile.TemporaryDirectory() as tmp:
            payload = self.run_cli(
                "init-run",
                "--repo-root",
                tmp,
                "--primary",
                "101",
                "--slug",
                "fix-bug",
                "--branch",
                "agent/issue-101-fix-bug",
                "--mode",
                "worktree",
                "--working-directory",
                str(Path(tmp) / "worktrees/issue-101-fix-bug"),
                "--main-repo-path",
                tmp,
                "--issues",
                "101",
                expected=2,
            )
            self.assertFalse(payload["ok"])
            self.assertIn("codex/issue-<primary>-<slug>", payload["details"][0])

    def test_update_state_enforces_review_gate_order(self):
        with tempfile.TemporaryDirectory() as tmp:
            run_dir = self.init_run(Path(tmp))
            payload = self.run_cli(
                "update-state",
                "--run-dir",
                str(run_dir),
                "--review-gate",
                "code-quality-reviewer=APPROVED",
                expected=2,
            )
            self.assertFalse(payload["ok"])
            self.assertIn("code-quality-reviewer cannot run before spec-reviewer", payload["error"])

    def test_update_state_requires_conventional_pr_title_and_justification(self):
        with tempfile.TemporaryDirectory() as tmp:
            run_dir = self.init_run(Path(tmp))
            payload = self.run_cli(
                "update-state",
                "--run-dir",
                str(run_dir),
                "--pr-title",
                "bad title",
                expected=2,
            )
            self.assertFalse(payload["ok"])
            payload = self.run_cli(
                "update-state",
                "--run-dir",
                str(run_dir),
                "--claude-warnings-status",
                "justified",
                expected=2,
            )
            self.assertFalse(payload["ok"])

    def test_poll_pr_advances_to_cleanup_when_strict_requirements_are_met(self):
        with tempfile.TemporaryDirectory() as tmp:
            run_dir = self.init_run(Path(tmp))
            self.run_cli(
                "update-state",
                "--run-dir",
                str(run_dir),
                "--review-gate",
                "spec-reviewer=APPROVED",
                "--review-gate",
                "code-quality-reviewer=APPROVED",
                "--review-gate",
                "architecture-validator=VALID",
                "--pr-number",
                "77",
                "--pr-title",
                "refactor: tighten gh-issue orchestration",
                "--pr-template-used",
                "--pr-template-sections-complete",
                "--commit-titles-state",
                "valid",
            )
            payload = self.run_cli(
                "poll-pr",
                "--run-dir",
                str(run_dir),
                "--checks-json-file",
                str(FIXTURES / "checks-success.json"),
                "--reviews-json-file",
                str(FIXTURES / "reviews-approved.json"),
            )
            state = json.loads((run_dir / "state.json").read_text())
            self.assertTrue(payload["ready_for_cleanup"])
            self.assertEqual(payload["claude_review_state"], "approved")
            self.assertEqual(state["phase"], "cleanup")

    def test_cleanup_stays_blocked_until_ci_and_feedback_are_complete(self):
        with tempfile.TemporaryDirectory() as tmp:
            run_dir = self.init_run(Path(tmp))
            self.run_cli(
                "update-state",
                "--run-dir",
                str(run_dir),
                "--review-gate",
                "spec-reviewer=APPROVED",
                "--review-gate",
                "code-quality-reviewer=APPROVED",
                "--review-gate",
                "architecture-validator=VALID",
                "--pr-number",
                "77",
                "--pr-title",
                "refactor: tighten gh-issue orchestration",
                "--pr-template-used",
                "--pr-template-sections-complete",
                "--commit-titles-state",
                "valid",
            )
            self.run_cli(
                "poll-pr",
                "--run-dir",
                str(run_dir),
                "--checks-json-file",
                str(FIXTURES / "checks-pending.json"),
                "--reviews-json-file",
                str(FIXTURES / "reviews-commented.json"),
            )
            payload = self.run_cli("cleanup-worktree", "--run-dir", str(run_dir), expected=2)
            self.assertFalse(payload["ok"])
            self.assertIn("cleanup blocked", payload["error"])

    def test_cleanup_dry_run_succeeds_when_ready(self):
        with tempfile.TemporaryDirectory() as tmp:
            run_dir = self.init_run(Path(tmp))
            self.run_cli(
                "update-state",
                "--run-dir",
                str(run_dir),
                "--review-gate",
                "spec-reviewer=APPROVED",
                "--review-gate",
                "code-quality-reviewer=APPROVED",
                "--review-gate",
                "architecture-validator=VALID",
                "--pr-number",
                "77",
                "--pr-title",
                "refactor: tighten gh-issue orchestration",
                "--pr-template-used",
                "--pr-template-sections-complete",
                "--commit-titles-state",
                "valid",
                "--claude-review-state",
                "commented",
                "--claude-blockers-status",
                "fixed",
                "--claude-warnings-status",
                "justified",
                "--claude-justification-recorded",
                "--ci-state",
                "success",
            )
            payload = self.run_cli("cleanup-worktree", "--run-dir", str(run_dir))
            self.assertTrue(payload["ok"])
            self.assertTrue(payload["dry_run"])

    def test_validate_resume_blocks_done_without_cleanup(self):
        with tempfile.TemporaryDirectory() as tmp:
            run_dir = self.init_run(Path(tmp))
            payload = self.run_cli(
                "update-state",
                "--run-dir",
                str(run_dir),
                "--phase",
                "done",
                expected=2,
            )
            self.assertFalse(payload["ok"])


if __name__ == "__main__":
    unittest.main()
