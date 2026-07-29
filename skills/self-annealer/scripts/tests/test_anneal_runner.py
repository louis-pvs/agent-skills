#!/usr/bin/env python3
"""Unit tests for anneal_runner.py script."""

import sys
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch

# Add scripts directory to path
SCRIPTS_DIR = Path(__file__).parent.parent.resolve()
sys.path.insert(0, str(SCRIPTS_DIR))

from anneal_runner import (  # noqa: E402
    execute_anneal_loop,
    parse_args,
    run_cmd,
)


class TestAnnealRunner(unittest.TestCase):
    def test_parse_args_defaults(self) -> None:
        args = parse_args([])
        self.assertEqual(args.max_iterations, 3)
        self.assertTrue(args.auto_rollback)

    def test_run_cmd_empty_command(self) -> None:
        code, stdout, stderr = run_cmd("")
        self.assertEqual(code, 1)
        self.assertIn("Empty command", stderr)

    def test_run_cmd_invalid_quotes(self) -> None:
        code, stdout, stderr = run_cmd('python3 -c "unclosed quote')
        self.assertEqual(code, 1)
        self.assertIn("Error parsing command string", stderr)

    def test_run_cmd_invalid_type(self) -> None:
        code, stdout, stderr = run_cmd(12345)  # type: ignore[arg-type]
        self.assertEqual(code, 1)
        self.assertIn("Invalid command type", stderr)

    def test_run_cmd_null_bytes(self) -> None:
        code, stdout, stderr = run_cmd("python3\x00 -c 'print(1)'")
        self.assertEqual(code, 1)
        self.assertIn("invalid null bytes", stderr)

        code_list, stdout_list, stderr_list = run_cmd(["python3\x00", "-c", "print(1)"])
        self.assertEqual(code_list, 1)
        self.assertIn("invalid null bytes", stderr_list)

    def test_run_cmd_success(self) -> None:
        code, stdout, stderr = run_cmd([sys.executable, "-c", "print('hello')"])
        self.assertEqual(code, 0)
        self.assertEqual(stdout, "hello")

    @patch("anneal_runner.run_cmd")
    def test_anneal_success_first_try(self, mock_run: MagicMock) -> None:
        mock_run.return_value = (0, "All tests passed", "")
        result = execute_anneal_loop(test_cmd="pytest", max_iterations=3, auto_rollback=True)
        self.assertTrue(result["converged"])
        self.assertEqual(result["iterations"], 1)
        self.assertFalse(result["rollback_executed"])

    @patch("anneal_runner.run_cmd")
    def test_anneal_max_iterations_exceeded_triggers_rollback(self, mock_run: MagicMock) -> None:
        # Mock test runner failing continuously, then git rollback call
        mock_run.side_effect = [
            (1, "Test failed line 10", ""),
            (1, "Test failed line 10", ""),
            (1, "Test failed line 10", ""),
            (0, "Rollback success", ""),
        ]
        result = execute_anneal_loop(test_cmd="pytest", max_iterations=3, auto_rollback=True)
        self.assertFalse(result["converged"])
        self.assertEqual(result["iterations"], 3)
        self.assertTrue(result["rollback_executed"])


if __name__ == "__main__":
    unittest.main()
