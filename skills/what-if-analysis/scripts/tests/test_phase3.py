#!/usr/bin/env python3
"""Unit tests for Phase 3 AST parsing, counterfactual generation, and failure pre-emption."""

import ast
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS_DIR = Path(__file__).parent.parent.resolve()
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from ast_graph_parser import extract_ast_call_sites  # noqa: E402
from counterfactual_generator import generate_counterfactual_test  # noqa: E402
from failure_preemptor import preempt_failure_modes  # noqa: E402


class TestPhase3Engine(unittest.TestCase):
    def test_extract_ast_call_sites(self) -> None:
        """Verify AST NodeVisitor distinguishes function calls from comments and strings."""
        code = """
# This is a comment referring to target_fn
def target_fn(x, y):
    return x + y

def caller_one():
    val = "target_fn" # String literal
    return target_fn(1, 2) # Actual AST Call
"""
        call_sites = extract_ast_call_sites("target_fn", code)
        self.assertEqual(len(call_sites), 1)
        self.assertEqual(call_sites[0]["line"], 8)
        self.assertEqual(call_sites[0]["caller_func"], "caller_one")

    def test_generate_counterfactual_test(self) -> None:
        """Verify counterfactual RED test case synthesis."""
        test_code = generate_counterfactual_test("calculate_score", ["val", "multiplier"], "pkg.core")
        self.assertIn("def test_counterfactual_calculate_score_null_check", test_code)
        self.assertIn("calculate_score(None, None)", test_code)
        self.assertIn("from pkg.core import calculate_score", test_code)
        self.assertIn("class TestCounterfactualCalculateScore", test_code)

    def test_generated_test_is_valid_python_with_real_assertions(self) -> None:
        """Verify the emitted test parses and asserts rather than swallowing exceptions."""
        test_code = generate_counterfactual_test("calculate_score", ["val"], "pkg.core")
        ast.parse(test_code)
        self.assertIn("with self.assertRaises(", test_code)
        self.assertNotIn("from module import", test_code)

    def test_generated_test_actually_fails_on_silent_symbol(self) -> None:
        """A symbol that silently accepts null input must make the generated test go RED."""
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "silent_mod.py").write_text("def calc(v):\n    return v\n", encoding="utf-8")
            (root / "test_gen.py").write_text(generate_counterfactual_test("calc", ["v"], "silent_mod"), encoding="utf-8")
            proc = subprocess.run(
                [sys.executable, "-m", "unittest", "test_gen", "-v"],
                cwd=root,
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(proc.returncode, 0, "generated test must fail against a non-validating symbol")
            self.assertIn("assertRaises", proc.stderr + proc.stdout)

    def test_preempt_failure_modes(self) -> None:
        """Verify pre-emptive failure risk assessment detects the planted zero division."""
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "sample.py").write_text("def target_fn(a):\n    return a / 0\n", encoding="utf-8")

            preemption = preempt_failure_modes("target_fn", root)
            self.assertEqual(preemption["total_risks"], 1)
            risk = preemption["preemptive_risks"][0]
            self.assertEqual(risk["risk_type"], "ZeroDivisionRisk")
            self.assertEqual(risk["severity"], "HIGH")
            self.assertEqual(risk["line"], 2)

    def test_preempt_ignores_division_outside_target_symbol(self) -> None:
        """A '/ 0' in an unrelated function must not be attributed to the target symbol."""
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "sample.py").write_text(
                "def target_fn(a):\n    return a + 1\n\n\ndef unrelated():\n    return 5 / 0\n",
                encoding="utf-8",
            )
            self.assertEqual(preempt_failure_modes("target_fn", root)["total_risks"], 0)

    def test_preempt_ignores_symbol_mentioned_only_in_comment(self) -> None:
        """A symbol appearing only in a comment must not be flagged."""
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "sample.py").write_text("# note: target_fn was removed\ndef other():\n    return 1 / 0\n", encoding="utf-8")
            self.assertEqual(preempt_failure_modes("target_fn", root)["total_risks"], 0)

    def test_preempt_flags_unguarded_parameter_divisor(self) -> None:
        """Division by an unguarded parameter is reported as a MEDIUM risk."""
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "sample.py").write_text("def target_fn(a, b):\n    return a / b\n", encoding="utf-8")
            risks = preempt_failure_modes("target_fn", root)["preemptive_risks"]
            self.assertEqual([r["risk_type"] for r in risks], ["UnguardedDivisionRisk"])

    def test_preempt_respects_existing_zero_guard(self) -> None:
        """A parameter checked against zero must not be reported as unguarded."""
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "sample.py").write_text(
                "def target_fn(a, b):\n    if b == 0:\n        return 0\n    return a / b\n", encoding="utf-8"
            )
            self.assertEqual(preempt_failure_modes("target_fn", root)["total_risks"], 0)


if __name__ == "__main__":
    unittest.main()
