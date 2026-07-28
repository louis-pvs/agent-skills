import json
import os
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

# Ensure parent script directory is in sys.path for test discovery
script_dir = Path(__file__).resolve().parent.parent
if str(script_dir) not in sys.path:
    sys.path.insert(0, str(script_dir))

from tdd_runner import detect_test_runner, run_test_command


class TestTddRunner(unittest.TestCase):
    def setUp(self):
        self.test_dir = Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.test_dir, ignore_errors=True)

    def test_detect_python_pytest(self):
        (self.test_dir / "pyproject.toml").touch()
        name, cmd = detect_test_runner(self.test_dir)
        self.assertEqual(name, "pytest")
        self.assertEqual(cmd, ["pytest"])

    def test_detect_python_unittest(self):
        (self.test_dir / "test_sample.py").touch()
        name, cmd = detect_test_runner(self.test_dir)
        self.assertEqual(name, "unittest")
        self.assertIn("unittest", cmd)

    def test_detect_node_package_json(self):
        pkg = self.test_dir / "package.json"
        pkg.write_text(json.dumps({"scripts": {"test": "jest"}}))
        name, cmd = detect_test_runner(self.test_dir)
        self.assertEqual(name, "npm test")
        self.assertEqual(cmd, ["npm", "test"])

    def test_detect_go_mod(self):
        (self.test_dir / "go.mod").touch()
        name, cmd = detect_test_runner(self.test_dir)
        self.assertEqual(name, "go test")
        self.assertEqual(cmd, ["go", "test", "./..."])

    def test_detect_cargo_toml(self):
        (self.test_dir / "Cargo.toml").touch()
        name, cmd = detect_test_runner(self.test_dir)
        self.assertEqual(name, "cargo test")
        self.assertEqual(cmd, ["cargo", "test"])

    def test_run_test_command_success(self):
        code, out, err = run_test_command(["python3", "-c", "import sys; sys.exit(0)"], self.test_dir)
        self.assertEqual(code, 0)

    def test_run_test_command_failure(self):
        code, out, err = run_test_command(["python3", "-c", "import sys; sys.exit(1)"], self.test_dir)
        self.assertEqual(code, 1)


if __name__ == "__main__":
    unittest.main()
