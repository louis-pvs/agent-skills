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
)


class TestAnnealRunner(unittest.TestCase):
    def test_parse_args_defaults(self) -> None:
        args = parse_args([])
        self.assertEqual(args.max_iterations, 3)
        self.assertTrue(args.auto_rollback)

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
