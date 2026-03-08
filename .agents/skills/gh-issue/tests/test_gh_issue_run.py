import subprocess
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[4]
SCRIPT = ROOT / ".agents/skills/gh-issue/scripts/gh_issue_run.py"


class GhIssueRunCompatibilityTests(unittest.TestCase):
    def test_wrapper_exposes_help(self):
        proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--help"],
            cwd=ROOT,
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, msg=proc.stderr)
        self.assertIn("Deterministic gh-issue workflow helper", proc.stdout)


if __name__ == "__main__":
    unittest.main()
