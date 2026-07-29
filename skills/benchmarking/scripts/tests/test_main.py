#!/usr/bin/env python3
"""Unit tests for benchmarking main.py script."""

import sys
import unittest
from pathlib import Path

TESTS_DIR = Path(__file__).parent.resolve()
SCRIPTS_DIR = TESTS_DIR.parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from main import main  # noqa: E402


class TestBenchmarkingMain(unittest.TestCase):
    def test_main_cli_execution(self):
        exit_code = main(["--cmd", "echo 'main test'", "--iterations", "1", "--json"])
        self.assertEqual(exit_code, 0)


if __name__ == "__main__":
    unittest.main()
