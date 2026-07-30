#!/usr/bin/env python3
"""Domain Modeling main automation script."""

import argparse
import sys
from pathlib import Path

# Dynamic sys.path Root Resolution
_repo_root = Path(__file__).resolve().parents[3]
if str(_repo_root) not in sys.path:
    sys.path.insert(0, str(_repo_root))


def run_health_check() -> bool:
    """Verifies existence and structural completeness of domain-modeling skill files."""
    skill_dir = Path(__file__).resolve().parents[1]
    required_files = [
        skill_dir / "SKILL.md",
        skill_dir / "README.md",
        skill_dir / "references" / "overview.md",
        skill_dir / "references" / "ddd-patterns.md",
        skill_dir / "references" / "state-machines.md",
    ]

    missing = [str(f.relative_to(skill_dir)) for f in required_files if not f.is_file()]
    if missing:
        print(f"Health check failed. Missing files: {missing}", file=sys.stderr)
        return False

    print("Domain Modeling skill health check passed cleanly.")
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description="Domain Modeling CLI tool")
    parser.add_argument("--check", action="store_true", help="Run health check")
    args = parser.parse_args()

    if args.check:
        success = run_health_check()
        return 0 if success else 1

    print("Domain Modeling script running.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
