#!/usr/bin/env python3
"""Unit tests for what-if-analysis blast radius calculator (impact_analyzer.py)."""

import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS_DIR = Path(__file__).parent.parent.resolve()
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from impact_analyzer import (  # noqa: E402
    _is_doc_file,
    _is_test_file,
    analyze_blast_radius,
    find_symbol_callers,
    generate_impact_report,
)


class TestImpactAnalyzer(unittest.TestCase):
    def test_is_test_file_classification(self) -> None:
        """Verify strict path component checking for test files without false substring matches."""
        # True test files
        self.assertTrue(_is_test_file("tests/test_core.py"))
        self.assertTrue(_is_test_file("skills/foo/test_bar.py"))
        self.assertTrue(_is_test_file("skills/foo/bar_test.py"))
        self.assertTrue(_is_test_file("skills/foo/test/helper.py"))

        # False positive candidates that should NOT be classified as test files
        self.assertFalse(_is_test_file("skills/foo/docker_setup.sh"))
        self.assertFalse(_is_test_file("skills/foo/latest_release.py"))
        self.assertFalse(_is_test_file("skills/foo/contest_scores.py"))

    def test_is_doc_file_classification(self) -> None:
        """Verify strict extension and path component checking for doc files."""
        # True doc files
        self.assertTrue(_is_doc_file("README.md"))
        self.assertTrue(_is_doc_file("skills/foo/SKILL.md"))
        self.assertTrue(_is_doc_file("docs/setup.txt"))

        # False positive candidates that should NOT be classified as doc files
        self.assertFalse(_is_doc_file("skills/foo/docker_setup.sh"))
        self.assertFalse(_is_doc_file("skills/foo/dock_loader.py"))

    def test_find_symbol_callers_and_tests(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)

            # Create source file
            src_dir = root / "src"
            src_dir.mkdir()
            (src_dir / "core.py").write_text("def core_func():\n    return 42\n", encoding="utf-8")
            (src_dir / "caller.py").write_text(
                "from src.core import core_func\n\ndef run():\n    return core_func()\n",
                encoding="utf-8",
            )

            # Create test file
            test_dir = root / "tests"
            test_dir.mkdir()
            (test_dir / "test_core.py").write_text(
                "from src.core import core_func\n\ndef test_core():\n    assert core_func() == 42\n",
                encoding="utf-8",
            )

            callers = find_symbol_callers("core_func", root)
            self.assertGreaterEqual(len(callers), 2)

    def test_pipe_escaping_in_report(self) -> None:
        """Verify that pipes inside source code snippets are escaped for markdown table safety."""
        analysis = {
            "symbol": "target_fn",
            "project_dir": "/tmp",
            "risk_level": "LOW",
            "total_impacted_files": 1,
            "code_callers": [
                {
                    "file": "src/core.py",
                    "line": 10,
                    "content": "def target_fn(x: int | None = None) -> bool | None:",
                }
            ],
            "impacted_code_files": ["src/core.py"],
            "impacted_test_files": [],
            "impacted_doc_files": [],
        }

        report = generate_impact_report(analysis)
        self.assertIn(r"int \| None", report)
        self.assertIn(r"bool \| None", report)

    def test_analyze_blast_radius_and_report(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "module.py").write_text("def target_fn():\n    pass\n", encoding="utf-8")

            analysis = analyze_blast_radius("target_fn", root)
            self.assertEqual(analysis["symbol"], "target_fn")
            self.assertIn("risk_level", analysis)

            report = generate_impact_report(analysis)
            self.assertIn("target_fn", report)
            self.assertIn("Blast Radius Report", report)


if __name__ == "__main__":
    unittest.main()
