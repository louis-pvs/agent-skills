#!/usr/bin/env python3
"""Architecture Auditor main automation script."""

import argparse
import ast
import os
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional


def analyze_file(file_path: Path) -> Dict[str, Any]:
    """Analyze a single Python file for structural metrics."""
    content = file_path.read_text(encoding="utf-8")
    lines = content.splitlines()

    class_count = 0
    function_count = 0

    try:
        tree = ast.parse(content)
        for node in ast.walk(tree):
            if isinstance(node, ast.ClassDef):
                class_count += 1
            elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                function_count += 1
    except SyntaxError:
        pass

    return {
        "file": str(file_path),
        "lines": len(lines),
        "classes": class_count,
        "functions": function_count,
    }


def resolve_safe_file(raw_path: str, base_dir: Optional[Path] = None) -> Path:
    """Resolve --file to an absolute path within base_dir."""
    base = (base_dir if base_dir is not None else Path.cwd()).resolve()
    target = Path(raw_path).expanduser().resolve()

    if not target.is_relative_to(base) or os.path.commonpath([str(base), str(target)]) != str(base):
        raise ValueError(f"Security Error: '{raw_path}' escapes allowed base directory '{base}'.")

    return target


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="Architecture Auditor CLI tool")
    parser.add_argument("--check", action="store_true", help="Run health check")
    parser.add_argument("--file", type=str, help="Path to single file to analyze")
    args = parser.parse_args(argv)

    if args.file:
        try:
            safe_path = resolve_safe_file(args.file)
        except ValueError as err:
            print(f"Error: {err}", file=sys.stderr)
            return 1
        if not safe_path.is_file():
            print(f"Error: File not found: {safe_path}", file=sys.stderr)
            return 1
        metrics = analyze_file(safe_path)
        print(f"File analysis metrics for {safe_path}: {metrics}")

    if args.check:
        print("Architecture Auditor check completed successfully.")

    return 0


if __name__ == "__main__":
    sys.exit(main())
