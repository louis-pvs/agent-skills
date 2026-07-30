#!/usr/bin/env python3
"""Standard Python Script Template for Agent Skills.

Uses Python Standard Library only to ensure zero-dependency execution across platforms.
Adheres strictly to ADR 0001 (Python 3 stdlib only) and ADR 0003 (CLI script standard).
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Dict, Optional, Sequence

# Dynamic root resolution to support clean internal imports when invoked directly
_REPO_ROOT = Path(__file__).resolve().parents[3] if len(Path(__file__).resolve().parents) > 3 else Path.cwd()
if str(_REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT))

try:
    from scripts._path_safety import sanitize_path
except ImportError:
    pass


def process_data(input_path: Optional[Path] = None, dry_run: bool = False, verbose: bool = False) -> Dict[str, Any]:
    """Processes input file and returns structured metadata result dictionary."""
    if verbose:
        sys.stderr.write("[DEBUG] Starting script execution loop...\n")

    if dry_run:
        if verbose:
            sys.stderr.write("[DEBUG] Dry-run mode enabled. Skipping persistent changes.\n")
        return {
            "status": "success",
            "dry_run": True,
            "message": "Dry-run execution completed.",
        }

    if input_path and not input_path.exists():
        raise FileNotFoundError(f"Input path does not exist: {input_path}")

    return {
        "status": "success",
        "dry_run": False,
        "file_name": input_path.name if input_path else None,
        "size_bytes": input_path.stat().st_size if input_path and input_path.is_file() else 0,
    }


def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    """Parses command-line arguments using Python stdlib argparse."""
    parser = argparse.ArgumentParser(
        description="Skill Python Automation Tool",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--input",
        type=str,
        help="Path to target input file or directory",
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


def main(argv: Optional[Sequence[str]] = None) -> int:
    """CLI entrypoint."""
    try:
        args = parse_args(argv)
    except SystemExit as exc:
        return exc.code if isinstance(exc.code, int) else 2

    if args.input:
        try:
            input_path = sanitize_path(args.input)
        except (ValueError, NameError):
            input_path = Path(args.input).resolve()
    else:
        input_path = None

    try:
        result = process_data(input_path=input_path, dry_run=args.dry_run, verbose=args.verbose)
        if args.json:
            print(json.dumps(result, indent=2))
        elif args.dry_run:
            print(f"[DRY-RUN] {result.get('message', 'Dry run completed.')}")
        else:
            print(f"[SUCCESS] Processed: {result}")
        return 0
    except Exception as err:
        sys.stderr.write(f"[ERROR] Script execution failed: {err}\n")
        return 1


if __name__ == "__main__":
    sys.exit(main())
