"""Unit tests for scripts/_template.py template script contract."""

import json
import subprocess
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).parent.parent.parent.resolve()
TEMPLATE_SCRIPT = REPO_ROOT / "scripts" / "_template.py"

# Add repo root to sys.path for direct import testing
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))


class TestTemplateScript(unittest.TestCase):
    """Test suite verifying template script standards."""

    def test_script_exists(self):
        """Verify _template.py exists in scripts/ directory."""
        self.assertTrue(TEMPLATE_SCRIPT.exists(), f"{TEMPLATE_SCRIPT} does not exist")

    def test_help_flag(self):
        """Verify --help flag outputs usage and exits with code 0."""
        res = subprocess.run(
            [sys.executable, str(TEMPLATE_SCRIPT), "--help"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0)
        self.assertIn("usage:", res.stdout.lower())

    def test_dry_run_flag(self):
        """Verify --dry-run flag prints dry-run preview and exits with code 0."""
        res = subprocess.run(
            [sys.executable, str(TEMPLATE_SCRIPT), "--dry-run"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0)
        self.assertIn("dry-run", res.stdout.lower())

    def test_json_flag(self):
        """Verify --json flag outputs valid JSON structure to stdout."""
        res = subprocess.run(
            [sys.executable, str(TEMPLATE_SCRIPT), "--json"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 0)
        data = json.loads(res.stdout)
        self.assertIsInstance(data, dict)
        self.assertIn("status", data)

    def test_invalid_arg_exit_code(self):
        """Verify invalid CLI argument exits with code 2."""
        res = subprocess.run(
            [sys.executable, str(TEMPLATE_SCRIPT), "--invalid-option-xyz"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(res.returncode, 2)


if __name__ == "__main__":
    unittest.main()
