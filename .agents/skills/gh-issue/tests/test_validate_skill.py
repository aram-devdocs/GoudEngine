import json
import subprocess
import unittest
from pathlib import Path

SCRIPT = Path(".agents/skills/gh-issue/scripts/validate_skill.py").resolve()


class ValidateSkillTests(unittest.TestCase):
    def test_validate_skill_passes(self):
        result = subprocess.run(["python3", str(SCRIPT)], text=True, capture_output=True)
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        payload = json.loads(result.stdout)
        self.assertTrue(payload["ok"])
        self.assertIn("SKILL.md", payload["validated_files"])


if __name__ == "__main__":
    unittest.main()
