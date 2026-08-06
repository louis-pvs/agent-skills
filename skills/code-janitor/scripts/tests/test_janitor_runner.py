#!/usr/bin/env python3
"""Tests for Code Janitor runner script.

Validates each smell detector with explicit element-level assertions
(item types, counts, specific string values) per project testing standards.
"""

import ast
import sys
import textwrap
import unittest
from pathlib import Path

# Resolve repo root so we can import the janitor_runner module
_REPO_ROOT = Path(__file__).resolve().parents[4]
if str(_REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT))

_SCRIPTS_DIR = Path(__file__).resolve().parent.parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

from janitor_runner import (  # noqa: E402
    SmellFinding,
    detect_dead_code,
    detect_deep_nesting,
    detect_high_complexity,
    detect_missing_docstrings,
    detect_missing_type_annotations,
    detect_oversized_functions,
    detect_stale_todos,
    detect_too_many_parameters,
    detect_unused_imports,
    format_findings_text,
    main,
    scan_file,
)


class TestSmellFinding(unittest.TestCase):
    """Tests for SmellFinding data class."""

    def test_to_dict_returns_correct_keys(self) -> None:
        finding = SmellFinding(
            file_path="test.py",
            line=10,
            category="dispensables",
            smell="unused-import",
            severity="critical",
            message="Unused import: 'os'.",
        )
        result = finding.to_dict()
        self.assertIsInstance(result, dict)
        expected_keys = {
            "file",
            "line",
            "end_line",
            "category",
            "smell",
            "severity",
            "message",
        }
        self.assertEqual(set(result.keys()), expected_keys)
        self.assertEqual(result["file"], "test.py")
        self.assertEqual(result["line"], 10)
        self.assertEqual(result["severity"], "critical")
        self.assertEqual(result["smell"], "unused-import")

    def test_repr_contains_key_info(self) -> None:
        finding = SmellFinding(
            file_path="test.py",
            line=5,
            category="bloaters",
            smell="oversized-function",
            severity="warning",
            message="Too big.",
        )
        repr_str = repr(finding)
        self.assertIn("test.py", repr_str)
        self.assertIn("L5", repr_str)
        self.assertIn("oversized-function", repr_str)


class TestUnusedImports(unittest.TestCase):
    """Tests for unused import detection."""

    def test_detects_unused_import(self) -> None:
        source = textwrap.dedent("""\
            import os
            import sys
            print(sys.argv)
        """)
        tree = ast.parse(source)
        findings = detect_unused_imports(tree, "test.py")
        self.assertEqual(len(findings), 1)
        self.assertIsInstance(findings[0], SmellFinding)
        self.assertEqual(findings[0].smell, "unused-import")
        self.assertEqual(findings[0].severity, "critical")
        self.assertIn("os", findings[0].message)

    def test_no_false_positive_for_used_import(self) -> None:
        source = textwrap.dedent("""\
            import os
            print(os.getcwd())
        """)
        tree = ast.parse(source)
        findings = detect_unused_imports(tree, "test.py")
        self.assertEqual(len(findings), 0)

    def test_detects_unused_from_import(self) -> None:
        source = textwrap.dedent("""\
            from pathlib import Path, PurePath
            p = Path(".")
        """)
        tree = ast.parse(source)
        findings = detect_unused_imports(tree, "test.py")
        self.assertEqual(len(findings), 1)
        self.assertIn("PurePath", findings[0].message)

    def test_handles_aliased_import(self) -> None:
        source = textwrap.dedent("""\
            import numpy as np
            print("hello")
        """)
        tree = ast.parse(source)
        findings = detect_unused_imports(tree, "test.py")
        self.assertEqual(len(findings), 1)
        self.assertIn("numpy", findings[0].message)


class TestDeadCode(unittest.TestCase):
    """Tests for dead/unreachable code detection."""

    def test_detects_code_after_return(self) -> None:
        source = textwrap.dedent("""\
            def foo():
                return 1
                x = 2
        """)
        tree = ast.parse(source)
        findings = detect_dead_code(tree, "test.py")
        self.assertEqual(len(findings), 1)
        self.assertIsInstance(findings[0], SmellFinding)
        self.assertEqual(findings[0].smell, "dead-code")
        self.assertEqual(findings[0].severity, "critical")
        self.assertIn("Return", findings[0].message)

    def test_detects_code_after_raise(self) -> None:
        source = textwrap.dedent("""\
            def bar():
                raise ValueError("fail")
                print("never runs")
        """)
        tree = ast.parse(source)
        findings = detect_dead_code(tree, "test.py")
        self.assertEqual(len(findings), 1)
        self.assertIn("Raise", findings[0].message)

    def test_no_false_positive_for_conditional_return(self) -> None:
        source = textwrap.dedent("""\
            def baz(x):
                if x:
                    return 1
                return 2
        """)
        tree = ast.parse(source)
        findings = detect_dead_code(tree, "test.py")
        self.assertEqual(len(findings), 0)


class TestOversizedFunctions(unittest.TestCase):
    """Tests for oversized function detection."""

    def test_detects_long_function(self) -> None:
        # Generate a function with 35 lines
        lines = ["def big_func():"]
        lines.extend([f"    x_{i} = {i}" for i in range(34)])
        source = "\n".join(lines) + "\n"
        tree = ast.parse(source)
        findings = detect_oversized_functions(tree, "test.py", max_lines=30)
        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].smell, "oversized-function")
        self.assertEqual(findings[0].severity, "warning")
        self.assertIn("big_func", findings[0].message)

    def test_accepts_short_function(self) -> None:
        source = textwrap.dedent("""\
            def small_func():
                return 1
        """)
        tree = ast.parse(source)
        findings = detect_oversized_functions(tree, "test.py", max_lines=30)
        self.assertEqual(len(findings), 0)


class TestTooManyParameters(unittest.TestCase):
    """Tests for too many parameters detection."""

    def test_detects_excessive_params(self) -> None:
        source = textwrap.dedent("""\
            def func(a, b, c, d, e, f, g):
                pass
        """)
        tree = ast.parse(source)
        findings = detect_too_many_parameters(tree, "test.py", max_params=5)
        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].smell, "too-many-parameters")
        self.assertIn("7", findings[0].message)

    def test_excludes_self_from_count(self) -> None:
        source = textwrap.dedent("""\
            class Foo:
                def method(self, a, b, c, d, e):
                    pass
        """)
        tree = ast.parse(source)
        findings = detect_too_many_parameters(tree, "test.py", max_params=5)
        self.assertEqual(len(findings), 0)


class TestDeepNesting(unittest.TestCase):
    """Tests for deep nesting detection."""

    def test_detects_deep_nesting(self) -> None:
        source = textwrap.dedent("""\
            def func():
                if True:
                    if True:
                        if True:
                            if True:
                                if True:
                                    pass
        """)
        tree = ast.parse(source)
        findings = detect_deep_nesting(tree, "test.py", max_depth=4)
        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].smell, "deep-nesting")
        self.assertIn("func", findings[0].message)

    def test_accepts_shallow_nesting(self) -> None:
        source = textwrap.dedent("""\
            def func():
                if True:
                    if True:
                        pass
        """)
        tree = ast.parse(source)
        findings = detect_deep_nesting(tree, "test.py", max_depth=4)
        self.assertEqual(len(findings), 0)


class TestHighComplexity(unittest.TestCase):
    """Tests for cyclomatic complexity detection."""

    def test_detects_high_complexity(self) -> None:
        # Build a function with many branches
        lines = ["def complex_func(x):"]
        for i in range(12):
            lines.append(f"    if x == {i}:")
            lines.append(f"        return {i}")
        lines.append("    return -1")
        source = "\n".join(lines) + "\n"
        tree = ast.parse(source)
        findings = detect_high_complexity(tree, "test.py", max_complexity=10)
        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].smell, "high-cyclomatic-complexity")

    def test_accepts_simple_function(self) -> None:
        source = textwrap.dedent("""\
            def simple(x):
                if x > 0:
                    return x
                return 0
        """)
        tree = ast.parse(source)
        findings = detect_high_complexity(tree, "test.py", max_complexity=10)
        self.assertEqual(len(findings), 0)


class TestMissingDocstrings(unittest.TestCase):
    """Tests for missing docstring detection."""

    def test_detects_missing_docstring(self) -> None:
        source = textwrap.dedent("""\
            def no_docs():
                return 1
        """)
        tree = ast.parse(source)
        findings = detect_missing_docstrings(tree, "test.py")
        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].smell, "missing-docstring")
        self.assertEqual(findings[0].severity, "advisory")

    def test_accepts_function_with_docstring(self) -> None:
        source = textwrap.dedent('''\
            def documented():
                """This function is documented."""
                return 1
        ''')
        tree = ast.parse(source)
        findings = detect_missing_docstrings(tree, "test.py")
        self.assertEqual(len(findings), 0)


class TestMissingTypeAnnotations(unittest.TestCase):
    """Tests for missing return type annotation detection."""

    def test_detects_missing_annotation(self) -> None:
        source = textwrap.dedent("""\
            def no_annotation():
                return 1
        """)
        tree = ast.parse(source)
        findings = detect_missing_type_annotations(tree, "test.py")
        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].smell, "missing-type-annotation")

    def test_accepts_annotated_function(self) -> None:
        source = textwrap.dedent("""\
            def annotated() -> int:
                return 1
        """)
        tree = ast.parse(source)
        findings = detect_missing_type_annotations(tree, "test.py")
        self.assertEqual(len(findings), 0)


class TestStaleTodos(unittest.TestCase):
    """Tests for stale TODO/FIXME marker detection."""

    def test_detects_todo(self) -> None:
        lines = ["x = 1", "# TODO: fix this later", "y = 2"]
        findings = detect_stale_todos(lines, "test.py")
        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].smell, "stale-todo")
        self.assertEqual(findings[0].line, 2)
        self.assertIn("TODO", findings[0].message)

    def test_detects_fixme(self) -> None:
        lines = ["# FIXME: broken thing"]
        findings = detect_stale_todos(lines, "test.py")
        self.assertEqual(len(findings), 1)
        self.assertIn("FIXME", findings[0].message)

    def test_no_false_positive_on_normal_comment(self) -> None:
        lines = ["# This is a normal comment", "x = 1"]
        findings = detect_stale_todos(lines, "test.py")
        self.assertEqual(len(findings), 0)


class TestScanFile(unittest.TestCase):
    """Integration tests for scan_file function."""

    def test_scan_file_returns_findings(self) -> None:
        # Create a temporary file with known smells
        import tempfile

        source = textwrap.dedent("""\
            import os
            import sys

            def undocumented_func(a, b, c, d, e, f):
                return 1
                print("dead")

            # TODO: clean this up
        """)
        with tempfile.NamedTemporaryFile(mode="w", suffix=".py", delete=False, encoding="utf-8") as f:
            f.write(source)
            tmp_path = Path(f.name)

        try:
            findings = scan_file(tmp_path, max_parameters=5)
            self.assertGreater(len(findings), 0)

            # Verify specific smell types are detected
            smell_types = {f.smell for f in findings}
            self.assertIn("unused-import", smell_types)
            self.assertIn("dead-code", smell_types)
            self.assertIn("too-many-parameters", smell_types)
            self.assertIn("missing-docstring", smell_types)
            self.assertIn("missing-type-annotation", smell_types)
            self.assertIn("stale-todo", smell_types)

            # Verify all findings are SmellFinding instances
            for finding in findings:
                self.assertIsInstance(finding, SmellFinding)
                self.assertIsInstance(finding.line, int)
                self.assertIn(
                    finding.severity,
                    ("critical", "warning", "advisory"),
                )
        finally:
            tmp_path.unlink()


class TestFormatFindings(unittest.TestCase):
    """Tests for text output formatting."""

    def test_empty_findings_shows_clean(self) -> None:
        result = format_findings_text([])
        self.assertIn("No code smells detected", result)

    def test_findings_show_severity_icons(self) -> None:
        findings = [
            SmellFinding(
                "test.py",
                1,
                "dispensables",
                "unused-import",
                "critical",
                "Unused import: 'os'.",
            ),
            SmellFinding(
                "test.py",
                10,
                "bloaters",
                "oversized-function",
                "warning",
                "Function too big.",
            ),
        ]
        result = format_findings_text(findings)
        self.assertIn("🚨", result)
        self.assertIn("⚠️", result)
        self.assertIn("Summary:", result)


class TestMainCLI(unittest.TestCase):
    """Tests for CLI entrypoint."""

    def test_no_args_returns_error(self) -> None:
        exit_code = main([])
        self.assertEqual(exit_code, 1)

    def test_nonexistent_file_returns_error(self) -> None:
        exit_code = main(["--file", "/tmp/nonexistent_file_xyz.py"])
        self.assertEqual(exit_code, 1)

    def test_scan_real_file_succeeds(self) -> None:
        import tempfile

        source = textwrap.dedent("""\
            def hello() -> None:
                \"\"\"Say hello.\"\"\"
                print("hello")
        """)
        with tempfile.NamedTemporaryFile(mode="w", suffix=".py", delete=False, encoding="utf-8") as f:
            f.write(source)
            tmp_path = Path(f.name)

        try:
            exit_code = main(["--file", str(tmp_path)])
            self.assertEqual(exit_code, 0)
        finally:
            tmp_path.unlink()

    def test_json_output(self) -> None:
        import io
        import tempfile

        source = textwrap.dedent("""\
            def hello() -> None:
                \"\"\"Say hello.\"\"\"
                print("hello")
        """)
        with tempfile.NamedTemporaryFile(mode="w", suffix=".py", delete=False, encoding="utf-8") as f:
            f.write(source)
            tmp_path = Path(f.name)

        try:
            # Capture stdout
            old_stdout = sys.stdout
            sys.stdout = io.StringIO()
            exit_code = main(["--file", str(tmp_path), "--json"])
            output = sys.stdout.getvalue()
            sys.stdout = old_stdout

            self.assertEqual(exit_code, 0)
            import json

            parsed = json.loads(output)
            self.assertIn("total", parsed)
            self.assertIn("findings", parsed)
            self.assertIsInstance(parsed["findings"], list)
        finally:
            tmp_path.unlink()


if __name__ == "__main__":
    unittest.main()
