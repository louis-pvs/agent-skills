#!/usr/bin/env python3
"""Tests for Architecture Auditor main script."""

import importlib.util
import tempfile
import unittest
from pathlib import Path

# Load main module dynamically since parent directory contains hyphens
main_script_path = Path(__file__).parent.parent / "main.py"
spec = importlib.util.spec_from_file_location("main", main_script_path)
main_module = importlib.util.module_from_spec(spec)  # type: ignore
spec.loader.exec_module(main_module)  # type: ignore

analyze_file = main_module.analyze_file
main = main_module.main


class TestMain(unittest.TestCase):
    def test_basic_pass(self) -> None:
        self.assertEqual(main(["--check"]), 0)

    def test_analyze_file_metrics(self) -> None:
        with tempfile.NamedTemporaryFile("w+", suffix=".py", delete=False) as f:
            f.write(
                "class Dummy:\n"
                "    def method_one(self):\n"
                "        pass\n"
                "    def method_two(self):\n"
                "        if True:\n"
                "            pass\n"
            )
            f_path = Path(f.name)

        try:
            metrics = analyze_file(f_path)
            self.assertEqual(metrics["lines"], 6)
            self.assertEqual(metrics["classes"], 1)
            self.assertEqual(metrics["functions"], 2)
        finally:
            f_path.unlink()


if __name__ == "__main__":
    unittest.main()
