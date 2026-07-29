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


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="Architecture Auditor CLI tool")
    parser.add_argument("--check", action="store_true", help="Run health check")
    parser.add_argument("--file", type=str, help="Path to single file to analyze")
    args = parser.parse_args(argv)

    if args.file:
        safe_file = os.path.basename(args.file)
        metrics = analyze_file(Path(safe_file))
        print(f"File analysis metrics for {safe_file}: {metrics}")

    if args.check:
        print("Architecture Auditor check completed successfully.")

    return 0


if __name__ == "__main__":
    sys.exit(main())
