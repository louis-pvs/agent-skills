#!/usr/bin/env python3
"""Tests for Domain Modeling main script."""

import sys
import unittest
from pathlib import Path

# Add scripts directory to sys.path
_scripts_dir = Path(__file__).resolve().parents[1]
if str(_scripts_dir) not in sys.path:
    sys.path.insert(0, str(_scripts_dir))

from main import main, run_health_check  # noqa: E402


class TestMain(unittest.TestCase):
    def test_run_health_check(self) -> None:
        self.assertTrue(run_health_check())

    def test_main_check_flag(self) -> None:
        sys.argv = ["main.py", "--check"]
        self.assertEqual(main(), 0)


if __name__ == "__main__":
    unittest.main()
