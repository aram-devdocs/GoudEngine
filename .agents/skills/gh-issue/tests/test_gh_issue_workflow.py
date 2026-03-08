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

    def test_init_run_creates_plan_and_state(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            payload = self.run_cli(
                "init-run",
                "--repo-root",
                str(repo_root),
                "--primary",
                "101",
                "--slug",
                "fix-bug",
                "--branch",
                "agent/issue-101-fix-bug",
                "--mode",
                "worktree",
                "--working-directory",
                str(repo_root / "worktrees/issue-101-fix-bug"),
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
            run_dir = Path(payload["run_dir"])
            self.assertTrue((run_dir / "plan.md").exists())
            self.assertTrue((run_dir / "state.json").exists())
            state = json.loads((run_dir / "state.json").read_text())
            self.assertEqual(state["phase"], "bootstrapped")
            self.assertEqual(state["todos"][1]["status"], "done")
            self.assertEqual(payload["next_todo"]["id"], "implementation")

    def test_update_state_changes_todo_and_gate(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            run_dir = Path(
                self.run_cli(
                    "init-run",
                    "--repo-root",
                    str(repo_root),
                    "--primary",
                    "101",
                    "--slug",
                    "fix-bug",
                    "--branch",
                    "agent/issue-101-fix-bug",
                    "--mode",
                    "in-place",
                    "--working-directory",
                    str(repo_root),
                    "--issues",
                    "101",
                )["run_dir"]
            )
            payload = self.run_cli(
                "update-state",
                "--run-dir",
                str(run_dir),
                "--phase",
                "reviewing",
                "--todo",
                "implementation=done",
                "--todo-owner",
                "review=quality-lead",
                "--review-gate",
                "spec-reviewer=APPROVED",
            )
            state = json.loads((run_dir / "state.json").read_text())
            self.assertEqual(payload["phase"], "reviewing")
            self.assertEqual(state["review_gates"]["spec-reviewer"], "APPROVED")
            self.assertEqual(state["todos"][2]["status"], "done")
            self.assertEqual(state["todos"][3]["owner"], "quality-lead")

    def test_validate_resume_blocks_wrong_branch(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            run_dir = Path(
                self.run_cli(
                    "init-run",
                    "--repo-root",
                    str(repo_root),
                    "--primary",
                    "101",
                    "--slug",
                    "fix-bug",
                    "--branch",
                    "agent/issue-101-fix-bug",
                    "--mode",
                    "in-place",
                    "--working-directory",
                    str(repo_root),
                    "--issues",
                    "101",
                )["run_dir"]
            )
            payload = self.run_cli(
                "validate-resume",
                "--run-dir",
                str(run_dir),
                "--branch",
                "main",
                expected=2,
            )
            self.assertFalse(payload["ok"])
            self.assertIn("branch mismatch", payload["errors"][0])

    def test_validate_resume_blocks_worktree_mismatch(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            worktree = repo_root / "worktrees/issue-101-fix-bug"
            run_dir = Path(
                self.run_cli(
                    "init-run",
                    "--repo-root",
                    str(repo_root),
                    "--primary",
                    "101",
                    "--slug",
                    "fix-bug",
                    "--branch",
                    "agent/issue-101-fix-bug",
                    "--mode",
                    "worktree",
                    "--working-directory",
                    str(worktree),
                    "--main-repo-path",
                    str(repo_root),
                    "--issues",
                    "101",
                )["run_dir"]
            )
            payload = self.run_cli(
                "validate-resume",
                "--run-dir",
                str(run_dir),
                "--branch",
                "agent/issue-101-fix-bug",
                "--cwd",
                str(repo_root),
                expected=2,
            )
            self.assertFalse(payload["ok"])
            self.assertIn("outside worktree", payload["errors"][0])

    def test_poll_pr_updates_state(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            run_dir = Path(
                self.run_cli(
                    "init-run",
                    "--repo-root",
                    str(repo_root),
                    "--primary",
                    "101",
                    "--slug",
                    "fix-bug",
                    "--branch",
                    "agent/issue-101-fix-bug",
                    "--mode",
                    "in-place",
                    "--working-directory",
                    str(repo_root),
                    "--issues",
                    "101",
                )["run_dir"]
            )
            self.run_cli("update-state", "--run-dir", str(run_dir), "--pr-number", "77")
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
            self.assertTrue(payload["ready_to_merge"])
            self.assertEqual(state["phase"], "cleanup")
            self.assertEqual(state["pr"]["ci_state"], "success")

    def test_cleanup_requires_green_state(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            worktree = repo_root / "worktrees/issue-101-fix-bug"
            run_dir = Path(
                self.run_cli(
                    "init-run",
                    "--repo-root",
                    str(repo_root),
                    "--primary",
                    "101",
                    "--slug",
                    "fix-bug",
                    "--branch",
                    "agent/issue-101-fix-bug",
                    "--mode",
                    "worktree",
                    "--working-directory",
                    str(worktree),
                    "--main-repo-path",
                    str(repo_root),
                    "--issues",
                    "101",
                )["run_dir"]
            )
            payload = self.run_cli("cleanup-worktree", "--run-dir", str(run_dir), expected=2)
            self.assertFalse(payload["ok"])
            self.assertEqual(payload["error"], "cleanup blocked")

    def test_cleanup_dry_run_succeeds_when_ready(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            worktree = repo_root / "worktrees/issue-101-fix-bug"
            run_dir = Path(
                self.run_cli(
                    "init-run",
                    "--repo-root",
                    str(repo_root),
                    "--primary",
                    "101",
                    "--slug",
                    "fix-bug",
                    "--branch",
                    "agent/issue-101-fix-bug",
                    "--mode",
                    "worktree",
                    "--working-directory",
                    str(worktree),
                    "--main-repo-path",
                    str(repo_root),
                    "--issues",
                    "101",
                )["run_dir"]
            )
            self.run_cli(
                "update-state",
                "--run-dir",
                str(run_dir),
                "--todo",
                "implementation=done",
                "--todo",
                "review=done",
                "--todo",
                "pr-loop=done",
                "--pr-number",
                "77",
                "--ci-state",
                "success",
                "--review-state",
                "approved",
            )
            payload = self.run_cli("cleanup-worktree", "--run-dir", str(run_dir))
            self.assertTrue(payload["ok"])
            self.assertTrue(payload["dry_run"])


if __name__ == "__main__":
    unittest.main()
