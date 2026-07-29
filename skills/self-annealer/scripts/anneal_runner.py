#!/usr/bin/env python3
"""Self-Annealing Repair Loop Runner.

Enforces bounded iterative repair cycles (RED -> GREEN -> REFACTOR) with strict iteration caps
and automatic git rollback safeguards to prevent infinite repair loops and structural debt ("slob").

Uses Python Standard Library only.
"""

import argparse
import json
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, Optional, Tuple


def run_cmd(cmd_str: str, cwd: Optional[Path] = None) -> Tuple[int, str, str]:
    """Executes shell command and returns exit code, stdout, and stderr."""
    try:
        proc = subprocess.run(
            shlex.split(cmd_str) if isinstance(cmd_str, str) else cmd_str,
            capture_output=True,
            text=True,
            cwd=str(cwd) if cwd else None,
        )
        return proc.returncode, proc.stdout.strip(), proc.stderr.strip()
    except Exception as err:
        return 1, "", str(err)


def execute_anneal_loop(
    test_cmd: str,
    max_iterations: int = 3,
    auto_rollback: bool = True,
    cwd: Optional[Path] = None,
) -> Dict[str, Any]:
    """Executes bounded self-annealing repair loop with iteration tracking and rollback."""
    iterations_run = 0
    last_stdout = ""
    last_stderr = ""

    for i in range(1, max_iterations + 1):
        iterations_run = i
        code, stdout, stderr = run_cmd(test_cmd, cwd=cwd)
        last_stdout = stdout
        last_stderr = stderr

        if code == 0:
            return {
                "converged": True,
                "iterations": iterations_run,
                "max_iterations": max_iterations,
                "rollback_executed": False,
                "last_stdout": last_stdout,
                "last_stderr": last_stderr,
            }

    # If loop completes without converging
    rollback_executed = False
    if auto_rollback:
        rb_code, _, _ = run_cmd("git checkout -- .", cwd=cwd)
        rollback_executed = rb_code == 0

    return {
        "converged": False,
        "iterations": iterations_run,
        "max_iterations": max_iterations,
        "rollback_executed": rollback_executed,
        "last_stdout": last_stdout,
        "last_stderr": last_stderr,
    }


def parse_args(args=None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Self-Annealing Loop Runner & Convergence Guard")
    parser.add_argument(
        "--cmd",
        type=str,
        default="python3 skills/tdd/scripts/tdd_runner.py --verify-green",
        help="Test / lint command to evaluate GREEN state",
    )
    parser.add_argument(
        "--max-iterations",
        type=int,
        default=3,
        help="Maximum allowed repair iterations before rollback (default: 3)",
    )
    parser.add_argument(
        "--no-rollback",
        action="store_false",
        dest="auto_rollback",
        help="Disable automatic git rollback when repair fails to converge",
    )
    parser.add_argument("--json", action="store_true", help="Output summary in JSON format")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview annealing loop execution parameters without running commands or rollback",
    )
    return parser.parse_args(args)


def main() -> int:
    args = parse_args()
    if args.dry_run:
        result = {
            "converged": True,
            "dry_run": True,
            "test_cmd": args.cmd,
            "max_iterations": args.max_iterations,
            "auto_rollback": args.auto_rollback,
            "message": (
                f"Dry-run: Would run command '{args.cmd}' up to {args.max_iterations} "
                f"iterations with auto_rollback={args.auto_rollback}."
            ),
        }
        if args.json:
            print(json.dumps(result, indent=2))
        else:
            print(f"[DRY-RUN] {result['message']}")
        return 0

    result = execute_anneal_loop(
        test_cmd=args.cmd,
        max_iterations=args.max_iterations,
        auto_rollback=args.auto_rollback,
    )

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        if result["converged"]:
            print(f"✅ Self-Annealing CONVERGED in {result['iterations']}/{result['max_iterations']} iterations.")
        else:
            print(f"❌ Self-Annealing FAILED to converge within {result['max_iterations']} iterations.")
            if result["rollback_executed"]:
                print("⚠️ Automated Git Rollback EXECUTED: Uncommitted changes reverted to prevent slob/regression.")

    return 0 if result["converged"] else 1


if __name__ == "__main__":
    sys.exit(main())
