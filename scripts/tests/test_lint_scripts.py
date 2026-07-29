"""Unit tests for scripts/lint_scripts.py script compliance linter."""

import json
import subprocess
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).parent.parent.parent.resolve()
LINT_SCRIPT = REPO_ROOT / "scripts" / "lint_scripts.py"

# Add repo root to sys.path
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))


class TestLintScripts(unittest.TestCase):
    """Test suite verifying lint_scripts.py logic and CLI contract."""

    def test_script_exists(self):
        """Verify lint_scripts.py exists in scripts/ directory."""
        self.assertTrue(LINT_SCRIPT.exists(), f"{LINT_SCRIPT} does not exist")

    def test_help_flag(self):
        """Verify --help flag outputs usage and exits with code 0."""
        res = subprocess.run(
            [sys.executable, str(LINT_SCRIPT), "--help"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0)
        self.assertIn("usage:", res.stdout.lower())

    def test_dry_run_flag(self):
        """Verify --dry-run flag prints dry-run preview and exits with code 0."""
        res = subprocess.run(
            [sys.executable, str(LINT_SCRIPT), "--dry-run"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0)
        self.assertIn("dry-run", res.stdout.lower())

    def test_repository_scripts_compliance(self):
        """Verify all python scripts in the repository pass lint compliance cleanly."""
        res = subprocess.run(
            [sys.executable, str(LINT_SCRIPT), "--json"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0, f"Lint scripts failed with output:\n{res.stdout}\n{res.stderr}")
        data = json.loads(res.stdout)
        self.assertTrue(data.get("compliant"), f"Non-compliant scripts found: {data.get('issues')}")

    def test_is_mutating_script_detection(self):
        """Verify is_mutating_script correctly identifies mutating operations via AST."""
        import ast

        from scripts.lint_scripts import is_mutating_script

        mutating_code = ast.parse("from pathlib import Path\nPath('foo.txt').write_text('bar')")
        self.assertTrue(is_mutating_script(mutating_code))

        non_mutating_code = ast.parse("target = 'foo.txt'.replace('a', 'b')")
        self.assertFalse(is_mutating_script(non_mutating_code))


if __name__ == "__main__":
    unittest.main()
