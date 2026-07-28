#!/usr/bin/env python3
"""Tests for symbol_nav — symbol/reference navigation."""

import json
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from symbol_nav import (
    detect_search_tool,
    find_definitions,
    find_references,
    format_results,
    search_files,
)


class TestDetectSearchTool(unittest.TestCase):
    """Tests for detecting available search tools."""

    def test_returns_string(self) -> None:
        tool = detect_search_tool()
        self.assertIn(tool, ("rg", "grep"))

    @patch("shutil.which", return_value=None)
    def test_falls_back_to_grep(self, mock_which: MagicMock) -> None:
        tool = detect_search_tool()
        self.assertEqual(tool, "grep")

    @patch("shutil.which", return_value="/usr/bin/rg")
    def test_prefers_ripgrep(self, mock_which: MagicMock) -> None:
        tool = detect_search_tool()
        self.assertEqual(tool, "rg")


class TestSearchFiles(unittest.TestCase):
    """Tests for searching files with a pattern."""

    def test_finds_pattern_in_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            test_file = Path(tmpdir) / "example.py"
            test_file.write_text(
                textwrap.dedent("""\
                    def my_function():
                        pass

                    class MyClass:
                        def method(self):
                            my_function()
                """),
                encoding="utf-8",
            )
            results = search_files("my_function", tmpdir)
            self.assertGreater(len(results), 0)
            # Should find both the definition and the usage
            self.assertTrue(any("my_function" in r["line"] for r in results))

    def test_returns_empty_for_no_matches(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            test_file = Path(tmpdir) / "example.py"
            test_file.write_text("x = 1\n", encoding="utf-8")
            results = search_files("nonexistent_symbol", tmpdir)
            self.assertEqual(results, [])

    def test_respects_file_extensions(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            py_file = Path(tmpdir) / "code.py"
            py_file.write_text("target = 1\n", encoding="utf-8")
            bin_file = Path(tmpdir) / "data.bin"
            bin_file.write_bytes(b"\x00target\x00")
            results = search_files("target", tmpdir)
            files = [r["file"] for r in results]
            self.assertTrue(any("code.py" in f for f in files))


class TestFindDefinitions(unittest.TestCase):
    """Tests for finding symbol definitions."""

    def test_finds_python_function_def(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            test_file = Path(tmpdir) / "mod.py"
            test_file.write_text(
                textwrap.dedent("""\
                    def my_function(x):
                        return x + 1

                    result = my_function(5)
                """),
                encoding="utf-8",
            )
            results = find_definitions("my_function", tmpdir)
            self.assertGreater(len(results), 0)
            self.assertTrue(any("def my_function" in r["line"] for r in results))

    def test_finds_python_class_def(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            test_file = Path(tmpdir) / "mod.py"
            test_file.write_text("class MyClass:\n    pass\n", encoding="utf-8")
            results = find_definitions("MyClass", tmpdir)
            self.assertGreater(len(results), 0)
            self.assertTrue(any("class MyClass" in r["line"] for r in results))


class TestFindReferences(unittest.TestCase):
    """Tests for finding symbol references (usages excluding definitions)."""

    def test_finds_usages(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            test_file = Path(tmpdir) / "mod.py"
            test_file.write_text(
                textwrap.dedent("""\
                    def my_func():
                        pass

                    my_func()
                    x = my_func
                """),
                encoding="utf-8",
            )
            results = find_references("my_func", tmpdir)
            # Should include usages but NOT the def line
            for r in results:
                self.assertNotIn("def my_func", r["line"])
            self.assertGreater(len(results), 0)


class TestFormatResults(unittest.TestCase):
    """Tests for formatting output."""

    def test_json_format(self) -> None:
        results = [
            {"file": "src/mod.py", "line_number": 10, "line": "def foo():"},
        ]
        output = format_results("foo", results, fmt="json")
        parsed = json.loads(output)
        self.assertEqual(parsed["symbol"], "foo")
        self.assertEqual(len(parsed["matches"]), 1)

    def test_text_format(self) -> None:
        results = [
            {"file": "src/mod.py", "line_number": 10, "line": "def foo():"},
        ]
        output = format_results("foo", results, fmt="text")
        self.assertIn("mod.py", output)
        self.assertIn("10", output)

    def test_empty_results(self) -> None:
        output = format_results("foo", [], fmt="text")
        self.assertIn("No matches", output)


if __name__ == "__main__":
    unittest.main()
