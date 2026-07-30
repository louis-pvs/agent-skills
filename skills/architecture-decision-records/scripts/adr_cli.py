#!/usr/bin/env python3
"""CLI utility wrapper for Architecture Decision Records (ADR)."""

import argparse
import sys
from pathlib import Path
from typing import Optional, Sequence

_script_dir = Path(__file__).resolve().parent
if str(_script_dir) not in sys.path:
    sys.path.insert(0, str(_script_dir))

from main import main as target_main  # noqa: E402


def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Architecture Decision Records CLI Wrapper")
    parser.add_argument("--dry-run", action="store_true", help="Preview ADR operations without modifying files")
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    """Main entrypoint wrapper for adr_cli."""
    _ = parse_args(argv)
    return target_main()


if __name__ == "__main__":
    sys.exit(main())
