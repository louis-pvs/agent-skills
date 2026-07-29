#!/usr/bin/env python3
"""Canonical Script Template for Agent Skills Automation.

This template defines the standard CLI contract for Python scripts in this repository.
It adheres strictly to ADR 0001 (Python 3 stdlib only) and ADR 0003 (CLI script standard).

Standard features:
- argparse with --help (-h) automatically supported
- --dry-run for safe preview of mutating side-effects
- --json for machine-readable output on stdout
- --verbose (-v) for detailed logging on stderr
- Structured exit codes: 0 (success), 1 (runtime error), 2 (argument error)
"""

import argparse
import json
import sys
from typing import Optional, Sequence


def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    """Parses command-line arguments using Python stdlib argparse."""
    parser = argparse.ArgumentParser(
        description="Canonical Script Template for Agent Skills Automation.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )

    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview actions and operations without making persistent changes.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Format output as a structured JSON object on stdout.",
    )
    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Print verbose diagnostic details to stderr.",
    )

    return parser.parse_args(argv)


def run_work(dry_run: bool = False, verbose: bool = False) -> dict:
    """Core execution logic.

    Args:
        dry_run: If True, simulate operations without side-effects.
        verbose: If True, emit diagnostic details to stderr.

    Returns:
        Dict payload representing outcome data.
    """
    if verbose:
        sys.stderr.write("[DEBUG] Starting script execution loop...\n")

    if dry_run:
        if verbose:
            sys.stderr.write("[DEBUG] Dry-run mode enabled. Skipping file I/O.\n")
        return {"status": "success", "dry_run": True, "message": "Dry-run execution completed."}

    # Perform actual production work here
    return {"status": "success", "dry_run": False, "message": "Execution completed successfully."}


def main(argv: Optional[Sequence[str]] = None) -> int:
    """Main entrypoint for script execution.

    Args:
        argv: Optional list of CLI arguments (defaults to sys.argv[1:]).

    Returns:
        Integer exit code: 0 for success, 1 for runtime failure, 2 for argument errors.
    """
    try:
        args = parse_args(argv)
    except SystemExit as exc:
        # argparse raises SystemExit on --help or argument errors
        return exc.code if isinstance(exc.code, int) else 2

    try:
        result = run_work(dry_run=args.dry_run, verbose=args.verbose)

        if args.json:
            print(json.dumps(result, indent=2))
        elif args.dry_run:
            print(f"[DRY-RUN] {result['message']}")
        else:
            print(f"[SUCCESS] {result['message']}")

        return 0
    except Exception as err:
        sys.stderr.write(f"[ERROR] Script execution failed: {err}\n")
        return 1


if __name__ == "__main__":
    sys.exit(main())
