#!/usr/bin/env python3
"""Unit tests for what-if-analysis unified main CLI script."""

import sys
import unittest
from pathlib import Path
from unittest.mock import patch

SCRIPTS_DIR = Path(__file__).parent.parent.resolve()
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from main import main  # noqa: E402


class TestMainOrchestrator(unittest.TestCase):
    def test_main_no_args(self) -> None:
        with patch("sys.argv", ["main.py"]):
            ret = main()
            self.assertEqual(ret, 0)

    def test_main_impact_command(self) -> None:
        with patch("main.run_subscript", return_value=0) as mock_run:
            ret = main(["impact", "--symbol", "test_symbol"])
            self.assertEqual(ret, 0)
            mock_run.assert_called_once_with("impact_analyzer.py", ["--symbol", "test_symbol"])


if __name__ == "__main__":
    unittest.main()
