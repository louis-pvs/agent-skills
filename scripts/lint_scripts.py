#!/usr/bin/env python3
"""Script Compliance Validator (ADR 0001 & ADR 0003).

Standalone, single-purpose CLI linter that audits Python automation scripts across
the repository for zero-dependency compliance (ADR 0001) and CLI scaffolding contracts (ADR 0003).
"""

import argparse
import ast
import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence

REPO_ROOT = Path(__file__).parent.parent.resolve()

# Allowlist of Python 3 standard library module names (mirrors
# skills/skill-creator/scripts/scaffold_skill.py's STDLIB_MODULES — copied
# rather than imported, since lint_scripts.py is repo-root tooling and must
# not depend on a portable skill script meant to be symlinked standalone).
STDLIB_MODULES = getattr(sys, "stdlib_module_names", None) or {
    "argparse",
    "ast",
    "asyncio",
    "base64",
    "collections",
    "concurrent",
    "configparser",
    "contextlib",
    "copy",
    "csv",
    "datetime",
    "decimal",
    "difflib",
    "doctest",
    "email",
    "enum",
    "functools",
    "glob",
    "hashlib",
    "hmac",
    "html",
    "http",
    "importlib",
    "inspect",
    "io",
    "json",
    "logging",
    "math",
    "multiprocessing",
    "os",
    "pathlib",
    "pickle",
    "platform",
    "pprint",
    "queue",
    "random",
    "re",
    "shlex",
    "shutil",
    "signal",
    "socket",
    "sqlite3",
    "ssl",
    "stat",
    "string",
    "struct",
    "subprocess",
    "sys",
    "tempfile",
    "textwrap",
    "threading",
    "time",
    "timeit",
    "tkinter",
    "token",
    "tokenize",
    "traceback",
    "types",
    "typing",
    "unittest",
    "urllib",
    "uuid",
    "warnings",
    "weakref",
    "xml",
    "zipfile",
    "zlib",
}


def discover_script_files(repo_dir: Path) -> List[Path]:
    """Finds all Python automation scripts subject to ADR 0001 and ADR 0003 audit."""
    target_patterns = [
        "scripts/*.py",
        "skills/*/scripts/*.py",
        "skills/*/templates/*.py",
    ]
    files = []
    for pattern in target_patterns:
        for path in repo_dir.glob(pattern):
            if "tests/" in str(path) or "__pycache__" in str(path):
                continue
            files.append(path)
    return sorted(files)


def is_inside_try_block(node: ast.AST, parent_map: dict) -> bool:
    """Helper to check if an AST node is enclosed in a try/except block."""
    curr = parent_map.get(node)
    while curr:
        if isinstance(curr, ast.Try):
            return True
        curr = parent_map.get(curr)
    return False


MUTATING_METHOD_NAMES = {
    "write_text",
    "write_bytes",
    "mkdir",
    "unlink",
    "rmdir",
    "makedirs",
    "symlink_to",
    "rmtree",
}

MUTATING_MODULE_CALLS = {
    ("os", "remove"),
    ("os", "unlink"),
    ("os", "rmdir"),
    ("os", "mkdir"),
    ("os", "makedirs"),
    ("os", "replace"),
    ("os", "rename"),
    ("os", "symlink"),
    ("os", "system"),
    ("shutil", "rmtree"),
    ("shutil", "move"),
    ("shutil", "copy"),
    ("shutil", "copy2"),
}


def is_mutating_script(tree: ast.AST) -> bool:
    """Inspects AST to determine if a script performs mutating operations.

    Checks for file writes, deletes, symlinks, or filesystem modification calls.
    """
    for node in ast.walk(tree):
        if isinstance(node, ast.Attribute):
            if node.attr in MUTATING_METHOD_NAMES:
                return True
            if isinstance(node.value, ast.Name) and (node.value.id, node.attr) in MUTATING_MODULE_CALLS:
                return True
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == "open":
            mode_str = ""
            if len(node.args) >= 2 and isinstance(node.args[1], ast.Constant) and isinstance(node.args[1].value, str):
                mode_str = node.args[1].value
            for kw in node.keywords:
                if kw.arg == "mode" and isinstance(kw.value, ast.Constant) and isinstance(kw.value.value, str):
                    mode_str = kw.value.value
            if any(m in mode_str for m in ("w", "a", "+", "x")):
                return True
    return False


def check_script_compliance(script_path: Path) -> List[str]:
    """Performs AST analysis on a script file and returns list of compliance issues."""
    issues = []
    try:
        content = script_path.read_text(encoding="utf-8")
        tree = ast.parse(content, filename=str(script_path))
    except Exception as err:
        return [f"AST parsing error: {err}"]

    # Build parent mapping for AST nodes
    parent_map = {}
    for parent in ast.walk(tree):
        for child in ast.iter_child_nodes(parent):
            parent_map[child] = parent

    # 1. Check for non-stdlib PyPI imports (ADR 0001)
    # Ignore optional fallback imports enclosed inside try...except blocks
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                mod = alias.name.split(".")[0]
                if mod and mod not in STDLIB_MODULES and not mod.startswith(".") and not is_inside_try_block(node, parent_map):
                    issues.append(f"ADR 0001 violation: Imports non-stdlib package '{mod}'.")
        elif isinstance(node, ast.ImportFrom):
            if node.module:
                mod = node.module.split(".")[0]
                if mod and mod not in STDLIB_MODULES and not mod.startswith(".") and not is_inside_try_block(node, parent_map):
                    issues.append(f"ADR 0001 violation: Imports from non-stdlib package '{mod}'.")

    # 2. Check for argparse usage (ADR 0003)
    has_argparse = "argparse" in content
    if not has_argparse and script_path.name != "python-test-template.py":
        issues.append("ADR 0003 violation: Missing 'argparse' CLI argument parser.")

    # 3. Check for main() function and __main__ guard (ADR 0003)
    has_main_def = "def main" in content
    has_main_guard = "__name__" in content and "__main__" in content
    if not has_main_def and script_path.name != "python-test-template.py":
        issues.append("ADR 0003 violation: Missing typed 'main()' function entrypoint.")
    if not has_main_guard:
        issues.append("ADR 0003 violation: Missing 'if __name__ == \"__main__\":' guard.")

    # 4. Check for --dry-run flag on mutating scripts (ADR 0003)
    # Mutating scripts are those that perform file writes, deletes, scaffolding, process spawning, or installs
    if script_path.name != "python-test-template.py" and is_mutating_script(tree):
        has_dry_run = "--dry-run" in content or "dry_run" in content
        if not has_dry_run:
            issues.append("ADR 0003 violation: Mutating script missing '--dry-run' flag support.")

    return issues


def audit_repository(repo_dir: Path, verbose: bool = False) -> Dict[str, Any]:
    """Audits all discovered script files across the repository."""
    scripts = discover_script_files(repo_dir)
    results = {}
    all_issues = []

    for s in scripts:
        rel_path = str(s.relative_to(repo_dir))
        issues = check_script_compliance(s)
        results[rel_path] = {
            "compliant": len(issues) == 0,
            "issues": issues,
        }
        if issues:
            all_issues.extend([f"{rel_path}: {issue}" for issue in issues])

        if verbose:
            status = "✅ PASS" if not issues else "❌ FAIL"
            sys.stderr.write(f"[DEBUG] {rel_path:<50} -> {status}\n")

    return {
        "compliant": len(all_issues) == 0,
        "total_audited": len(scripts),
        "issues_count": len(all_issues),
        "issues": all_issues,
        "details": results,
    }


def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    """Parses CLI arguments for lint_scripts.py."""
    parser = argparse.ArgumentParser(
        description="Script Compliance Validator (ADR 0001 & ADR 0003)",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview list of script files to be audited without executing AST checks.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output structured JSON audit report to stdout.",
    )
    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Print verbose diagnostic details to stderr.",
    )
    return parser.parse_args(argv)


def main(argv: Optional[Sequence[str]] = None) -> int:
    """CLI entrypoint for lint_scripts.py."""
    try:
        args = parse_args(argv)
    except SystemExit as exc:
        return exc.code if isinstance(exc.code, int) else 2

    scripts = discover_script_files(REPO_ROOT)

    if args.dry_run:
        rel_paths = [str(s.relative_to(REPO_ROOT)) for s in scripts]
        if args.json:
            print(json.dumps({"dry_run": True, "files_to_audit": rel_paths}, indent=2))
        else:
            print(f"[DRY-RUN] Would audit {len(scripts)} script files:")
            for p in rel_paths:
                print(f"  - {p}")
        return 0

    report = audit_repository(REPO_ROOT, verbose=args.verbose)

    if args.json:
        print(json.dumps(report, indent=2))
    else:
        if report["compliant"]:
            print(f"✅ All {report['total_audited']} repository scripts are fully compliant with ADR 0001 & ADR 0003!")
        else:
            print(
                f"❌ Script compliance check FAILED "
                f"({report['issues_count']} issues found across {report['total_audited']} files):"
            )
            for issue in report["issues"]:
                print(f"  - {issue}")

    return 0 if report["compliant"] else 1


if __name__ == "__main__":
    sys.exit(main())
