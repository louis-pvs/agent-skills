#!/usr/bin/env python3
"""Git temporal coupling analysis.

Parses git history to find files that frequently change together (temporal coupling).
This reveals hidden logical dependencies that static analysis misses.

Uses Python 3 Standard Library only.
"""

import argparse
import json
import subprocess
import sys
from collections import Counter
from typing import Any, Dict, List, Optional, Set


def get_git_root(cwd: Optional[str] = None) -> Optional[str]:
    """Detect the git repository root directory.

    Args:
        cwd: Working directory to check. Defaults to current directory.

    Returns:
        Absolute path to git root, or None if not in a git repo.
    """
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True,
            text=True,
            cwd=cwd,
            timeout=10,
        )
        if result.returncode == 0:
            return result.stdout.strip()
    except (FileNotFoundError, subprocess.TimeoutExpired):
        pass
    return None


def run_git_log(
    target_file: Optional[str] = None,
    max_commits: int = 500,
    cwd: Optional[str] = None,
) -> str:
    """Run git log --name-only and return raw output.

    Args:
        target_file: If provided, only show commits that touched this file.
        max_commits: Maximum number of commits to retrieve.
        cwd: Working directory for git commands.

    Returns:
        Raw git log output string.
    """
    cmd = [
        "git",
        "log",
        "--name-only",
        "--pretty=format:%H",
        f"-n{max_commits}",
    ]
    if target_file:
        cmd.extend(["--", target_file])

    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        cwd=cwd,
        timeout=30,
    )
    if result.returncode != 0:
        raise RuntimeError(f"git log failed: {result.stderr.strip()}")
    return result.stdout


def parse_git_log(log_output: str) -> List[Set[str]]:
    """Parse git log --name-only output into a list of file sets per commit.

    The format is:
        <commit_hash>
        <blank_line>
        <file1>
        <file2>
        ...
        <blank_line>
        <next_commit_hash>
        ...

    Args:
        log_output: Raw output from git log --name-only --pretty=format:%H.

    Returns:
        List of sets, each set containing filenames changed in one commit.
    """
    if not log_output.strip():
        return []

    commits: List[Set[str]] = []
    current_files: Set[str] = set()

    for line in log_output.splitlines():
        stripped = line.strip()

        if not stripped:
            continue

        # A 40-char hex string is a commit hash
        if len(stripped) >= 7 and all(c in "0123456789abcdef" for c in stripped):
            # This is a commit hash — save any previous commit's files
            if current_files:
                commits.append(current_files)
                current_files = set()
            continue

        # Otherwise it's a filename
        if stripped:
            current_files.add(stripped)

    # Don't forget the last commit
    if current_files:
        commits.append(current_files)

    return commits


def find_coupled_files(
    target_file: str,
    commits: List[Set[str]],
    min_commits: int = 2,
    limit: int = 30,
) -> List[Dict[str, Any]]:
    """Find files that frequently co-change with the target file.

    Args:
        target_file: The file to find coupling for.
        commits: List of file sets per commit (from parse_git_log).
        min_commits: Minimum number of co-commits to include in results.
        limit: Maximum number of results to return.

    Returns:
        List of dicts with keys: file, co_commits, coupling_ratio.
        Sorted by co_commits descending.
    """
    # Normalize target path
    target = target_file.replace("\\", "/")

    # Count co-occurrences
    co_changes: Counter = Counter()
    target_commits = 0

    for file_set in commits:
        # Normalize paths in commit
        normalized = {f.replace("\\", "/") for f in file_set}
        if target in normalized:
            target_commits += 1
            for f in normalized:
                if f != target:
                    co_changes[f] += 1

    if target_commits == 0:
        return []

    # Build results
    results = []
    for filepath, count in co_changes.most_common():
        if count < min_commits:
            break
        results.append(
            {
                "file": filepath,
                "co_commits": count,
                "coupling_ratio": round(count / target_commits, 4),
            }
        )

    return results[:limit]


def format_results(
    target_file: str,
    results: List[Dict[str, Any]],
    fmt: str = "text",
) -> str:
    """Format coupling results for output.

    Args:
        target_file: The file that was analyzed.
        results: List of coupling result dicts.
        fmt: Output format, either 'text' or 'json'.

    Returns:
        Formatted string.
    """
    if fmt == "json":
        return json.dumps(
            {"target_file": target_file, "coupled_files": results},
            indent=2,
            ensure_ascii=False,
        )

    if not results:
        return f"No coupled files found for '{target_file}'."

    lines = [f"Temporal coupling for: {target_file}", ""]
    lines.append(f"{'File':<60} {'Co-commits':>10} {'Ratio':>8}")
    lines.append("-" * 80)
    for r in results:
        lines.append(f"{r['file']:<60} {r['co_commits']:>10} {r['coupling_ratio']:>8.2f}")
    return "\n".join(lines)


def main() -> int:
    """CLI entry point."""
    parser = argparse.ArgumentParser(description="Find files temporally coupled to a target file via git history.")
    parser.add_argument(
        "--file",
        required=True,
        help="Target file path to analyze coupling for.",
    )
    parser.add_argument(
        "--min-commits",
        type=int,
        default=2,
        help="Minimum co-commit count to include (default: 2).",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=30,
        help="Maximum number of coupled files to show (default: 30).",
    )
    parser.add_argument(
        "--max-history",
        type=int,
        default=500,
        help="Maximum number of git commits to analyze (default: 500).",
    )
    parser.add_argument(
        "--format",
        choices=["text", "json"],
        default="text",
        help="Output format (default: text).",
    )
    args = parser.parse_args()

    git_root = get_git_root()
    if git_root is None:
        print("Error: Not inside a git repository.", file=sys.stderr)
        return 1

    try:
        raw_log = run_git_log(
            target_file=args.file,
            max_commits=args.max_history,
            cwd=git_root,
        )
    except RuntimeError as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1

    commits = parse_git_log(raw_log)

    # If we filtered by target_file, we only got commits touching that file.
    # But we need all files in those commits. Re-run without file filter
    # to get the full file lists for those commits.
    if commits:
        # Get full git log (all files) for comprehensive coupling analysis
        try:
            full_log = run_git_log(
                max_commits=args.max_history,
                cwd=git_root,
            )
            commits = parse_git_log(full_log)
        except RuntimeError:
            pass  # Fall back to filtered results

    results = find_coupled_files(
        args.file,
        commits,
        min_commits=args.min_commits,
        limit=args.limit,
    )

    print(format_results(args.file, results, fmt=args.format))
    return 0


if __name__ == "__main__":
    sys.exit(main())
