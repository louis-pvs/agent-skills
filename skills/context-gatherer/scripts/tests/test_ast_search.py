#!/usr/bin/env python3
"""Tests for ast_search — AST-based structural code search."""

import json
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from ast_search import (
    detect_ast_tool,
    find_classes,
    find_functions,
    format_results,
    parse_pattern,
    search_python_ast,
)


class TestDetectAstTool(unittest.TestCase):
    """Tests for detecting available AST tools."""

    def test_returns_valid_tool(self) -> None:
        tool = detect_ast_tool()
        self.assertIn(tool, ("ast-grep", "python-ast", "regex"))

    @patch("shutil.which", return_value=None)
    def test_falls_back_without_ast_grep(self, mock_which: MagicMock) -> None:
        tool = detect_ast_tool()
        # Without ast-grep, should fall back to python-ast or regex
        self.assertIn(tool, ("python-ast", "regex"))


class TestParsePattern(unittest.TestCase):
    """Tests for parsing search patterns."""

    def test_class_pattern(self) -> None:
        result = parse_pattern("class * (BaseHandler)")
        self.assertEqual(result["type"], "class")
        self.assertEqual(result["name_glob"], "*")
        self.assertEqual(result["base_class"], "BaseHandler")

    def test_function_pattern(self) -> None:
        result = parse_pattern("def test_*")
        self.assertEqual(result["type"], "function")
        self.assertEqual(result["name_glob"], "test_*")

    def test_class_pattern_no_base(self) -> None:
        result = parse_pattern("class My*")
        self.assertEqual(result["type"], "class")
        self.assertEqual(result["name_glob"], "My*")
        self.assertIsNone(result.get("base_class"))

    def test_plain_pattern(self) -> None:
        result = parse_pattern("some_identifier")
        self.assertEqual(result["type"], "identifier")
        self.assertEqual(result["name_glob"], "some_identifier")


class TestSearchPythonAst(unittest.TestCase):
    """Tests for Python AST-based search."""

    def test_finds_class_with_base(self) -> None:
        code = textwrap.dedent("""\
            class BaseHandler:
                pass

            class MyHandler(BaseHandler):
                def handle(self):
                    pass

            class OtherHandler(BaseHandler):
                def handle(self):
                    pass

            class Unrelated:
                pass
        """)
        with tempfile.TemporaryDirectory() as tmpdir:
            test_file = Path(tmpdir) / "handlers.py"
            test_file.write_text(code, encoding="utf-8")
            results = search_python_ast(str(test_file), {"type": "class", "name_glob": "*", "base_class": "BaseHandler"})
            names = [r["name"] for r in results]
            self.assertIn("MyHandler", names)
            self.assertIn("OtherHandler", names)
            self.assertNotIn("Unrelated", names)
            self.assertNotIn("BaseHandler", names)

    def test_finds_function_by_prefix(self) -> None:
        code = textwrap.dedent("""\
            def test_login():
                pass

            def test_logout():
                pass

            def helper():
                pass
        """)
        with tempfile.TemporaryDirectory() as tmpdir:
            test_file = Path(tmpdir) / "tests.py"
            test_file.write_text(code, encoding="utf-8")
            results = search_python_ast(str(test_file), {"type": "function", "name_glob": "test_*"})
            names = [r["name"] for r in results]
            self.assertIn("test_login", names)
            self.assertIn("test_logout", names)
            self.assertNotIn("helper", names)

    def test_handles_syntax_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            test_file = Path(tmpdir) / "broken.py"
            test_file.write_text("def broken(\n", encoding="utf-8")
            results = search_python_ast(str(test_file), {"type": "function", "name_glob": "*"})
            self.assertEqual(results, [])


class TestFindClasses(unittest.TestCase):
    """Tests for the high-level find_classes function."""

    def test_finds_classes_in_directory(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            test_file = Path(tmpdir) / "models.py"
            test_file.write_text(
                textwrap.dedent("""\
                    class User:
                        pass

                    class Admin(User):
                        pass
                """),
                encoding="utf-8",
            )
            results = find_classes("*", tmpdir, base_class="User")
            names = [r["name"] for r in results]
            self.assertIn("Admin", names)
            self.assertNotIn("User", names)

    def test_skips_non_python_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            py_file = Path(tmpdir) / "code.py"
            py_file.write_text("class Foo:\n    pass\n", encoding="utf-8")
            txt_file = Path(tmpdir) / "notes.txt"
            txt_file.write_text("class Foo:\n    pass\n", encoding="utf-8")
            results = find_classes("*", tmpdir)
            files = [r["file"] for r in results]
            self.assertTrue(all(f.endswith(".py") for f in files))


class TestFindFunctions(unittest.TestCase):
    """Tests for the high-level find_functions function."""

    def test_finds_functions_by_glob(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            test_file = Path(tmpdir) / "utils.py"
            test_file.write_text(
                textwrap.dedent("""\
                    def validate_email(e):
                        pass

                    def validate_phone(p):
                        pass

                    def send_email(e):
                        pass
                """),
                encoding="utf-8",
            )
            results = find_functions("validate_*", tmpdir)
            names = [r["name"] for r in results]
            self.assertIn("validate_email", names)
            self.assertIn("validate_phone", names)
            self.assertNotIn("send_email", names)


class TestFormatResults(unittest.TestCase):
    """Tests for formatting AST search output."""

    def test_json_format(self) -> None:
        results = [
            {"file": "src/mod.py", "name": "MyClass", "line": 5, "type": "class"},
        ]
        output = format_results("class *", results, fmt="json")
        parsed = json.loads(output)
        self.assertEqual(parsed["pattern"], "class *")
        self.assertEqual(len(parsed["matches"]), 1)

    def test_text_format(self) -> None:
        results = [
            {"file": "src/mod.py", "name": "MyClass", "line": 5, "type": "class"},
        ]
        output = format_results("class *", results, fmt="text")
        self.assertIn("MyClass", output)
        self.assertIn("mod.py", output)

    def test_empty_results(self) -> None:
        output = format_results("class *", [], fmt="text")
        self.assertIn("No matches", output)


if __name__ == "__main__":
    unittest.main()
