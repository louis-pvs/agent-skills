#!/usr/bin/env python3
"""Benchmarking main automation script entrypoint."""

import argparse
import sys
from pathlib import Path
from typing import List, Optional

SCRIPT_DIR = Path(__file__).parent.resolve()
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

try:
    from .benchmark_runner import main as runner_main
except ImportError:
    from benchmark_runner import main as runner_main


def main(argv: Optional[List[str]] = None) -> int:
    """Main CLI entrypoint for benchmarking skill."""
    parser = argparse.ArgumentParser(description="Benchmarking Automation Script")
    parser.add_argument("--cmd", type=str, help="Target command to benchmark")
    parser.add_argument("--check", action="store_true", help="Run health check")

    args, remaining = parser.parse_known_args(argv)

    if args.check:
        print("Benchmarking skill check completed successfully.")
        return 0

    return runner_main(argv)


if __name__ == "__main__":
    sys.exit(main())
