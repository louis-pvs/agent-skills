#!/usr/bin/env python3
"""What-If Analysis Unified CLI Orchestrator.

Provides a standardized single-entrypoint CLI for all what-if-analysis capabilities:
  - impact: Calculate blast radius & dependency caller ripple (Phase 1)
  - scenario: Evaluate scenario tradeoff matrices (Phase 2)
  - ast: Parse AST call-graph nodes (Phase 3)
  - counterfactual: Generate adversarial RED test cases (Phase 3)
  - preempt: Analyze pre-emptive failure risks (Phase 3)
"""

import argparse
import subprocess
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent.resolve()

# Single source of truth for sub-command dispatch. Flag definitions live in the
# sub-scripts themselves so they cannot drift between the two entrypoints.
_COMMANDS = {
    "impact": "impact_analyzer.py",
    "scenario": "scenario_runner.py",
    "ast": "ast_graph_parser.py",
    "counterfactual": "counterfactual_generator.py",
    "preempt": "failure_preemptor.py",
}


def run_subscript(script_name: str, passthrough_args: list) -> int:
    """Executes a sub-script via subprocess to preserve ADR 0001 zero-dependency isolation."""
    target_script = SCRIPT_DIR / script_name
    if not target_script.exists():
        sys.stderr.write(f"Error: Sub-script '{target_script}' not found.\n")
        return 1
    cmd = [sys.executable, str(target_script)] + passthrough_args
    res = subprocess.run(cmd)
    return res.returncode


def parse_args(argv=None):
    """Parses only the sub-command; all remaining flags are forwarded to the sub-script."""
    parser = argparse.ArgumentParser(
        description="What-If Analysis Unified CLI",
        epilog="Run '<command> --help' for the flags of a specific sub-command.",
    )
    parser.add_argument("command", choices=sorted(_COMMANDS), help="Sub-command to execute")
    parser.add_argument("args", nargs=argparse.REMAINDER, help="Flags forwarded to the sub-script")
    return parser.parse_args(argv)


def main(argv=None) -> int:
    argv = list(sys.argv[1:]) if argv is None else list(argv)
    if not argv:
        print(f"Usage: python3 skills/what-if-analysis/scripts/main.py [{'|'.join(sorted(_COMMANDS))}]")
        return 0

    args = parse_args(argv)
    return run_subscript(_COMMANDS[args.command], args.args)


if __name__ == "__main__":
    sys.exit(main())
