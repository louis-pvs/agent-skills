#!/usr/bin/env python3
"""Tests for Architecture Auditor main script."""

import importlib.util
import os
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

    def test_file_flag_resolves_nested_path(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            nested_dir = Path(tmp_dir) / "subdir"
            nested_dir.mkdir()
            (nested_dir / "sample.py").write_text("def foo():\n    pass\n", encoding="utf-8")

            cwd_before = os.getcwd()
            os.chdir(tmp_dir)
            try:
                exit_code = main(["--file", "subdir/sample.py"])
            finally:
                os.chdir(cwd_before)

            self.assertEqual(exit_code, 0)

    def test_file_flag_rejects_path_traversal(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            cwd_before = os.getcwd()
            os.chdir(tmp_dir)
            try:
                exit_code = main(["--file", "../outside.py"])
            finally:
                os.chdir(cwd_before)

            self.assertEqual(exit_code, 1)


if __name__ == "__main__":
    unittest.main()
