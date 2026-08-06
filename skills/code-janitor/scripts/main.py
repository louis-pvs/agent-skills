#!/usr/bin/env python3
"""Code Janitor main entry point.

Delegates to janitor_runner.py for AST-based code smell scanning.
Adheres to ADR 0003 (CLI script standard).
"""

import argparse
import sys
from pathlib import Path
from typing import Optional, Sequence

_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

from janitor_runner import main as janitor_main  # noqa: E402


def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    """Parse command-line arguments for the Code Janitor CLI."""
    parser = argparse.ArgumentParser(
        description="Code Janitor — AST-based code smell scanner",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--file",
        type=str,
        help="Path to a single Python file to scan.",
    )
    parser.add_argument(
        "--dir",
        type=str,
        help="Path to a directory to scan recursively.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output findings as JSON.",
    )
    parser.add_argument(
        "--max-function-lines",
        type=int,
        default=30,
        help="Maximum allowed lines per function.",
    )
    parser.add_argument(
        "--max-parameters",
        type=int,
        default=5,
        help="Maximum allowed parameters per function.",
    )
    parser.add_argument(
        "--max-nesting-depth",
        type=int,
        default=4,
        help="Maximum allowed nesting depth.",
    )
    parser.add_argument(
        "--max-complexity",
        type=int,
        default=10,
        help="Maximum allowed cyclomatic complexity per function.",
    )
    return parser.parse_args(argv)


def main(argv: Optional[Sequence[str]] = None) -> int:
    """CLI entrypoint delegating to janitor_runner."""
    return janitor_main(argv)


if __name__ == "__main__":
    sys.exit(main())
