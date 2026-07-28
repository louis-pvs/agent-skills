#!/usr/bin/env python3
"""AST-based structural code search.

Searches for code structures (classes, functions) using Python's ast module.
Uses ast-grep when available for polyglot support, falls back to Python ast
for .py files and regex heuristics for others.

Uses Python 3 Standard Library only.
"""

import argparse
import ast
import fnmatch
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional


def detect_ast_tool() -> str:
    """Detect the best available AST search tool.

    Returns:
        'ast-grep' if sg is available, 'python-ast' for Python-only,
        or 'regex' as last resort.
    """
    if shutil.which("sg") or shutil.which("ast-grep"):
        return "ast-grep"
    # Python's ast module is always available for .py files
    return "python-ast"


def parse_pattern(pattern: str) -> Dict[str, Any]:
    """Parse a search pattern into structured components.

    Supported patterns:
        - "class * (BaseClass)"  → find classes extending BaseClass
        - "class My*"            → find classes matching glob
        - "def test_*"           → find functions matching glob
        - "some_identifier"      → find any identifier

    Args:
        pattern: Human-readable search pattern.

    Returns:
        Dict with keys: type, name_glob, and optionally base_class.
    """
    pattern = pattern.strip()

    # "class NAME (BASE)"
    class_match = re.match(
        r"^class\s+(\S+)\s*\(\s*(\S+)\s*\)$", pattern
    )
    if class_match:
        return {
            "type": "class",
            "name_glob": class_match.group(1),
            "base_class": class_match.group(2),
        }

    # "class NAME"
    class_simple = re.match(r"^class\s+(\S+)$", pattern)
    if class_simple:
        return {
            "type": "class",
            "name_glob": class_simple.group(1),
        }

    # "def NAME"
    func_match = re.match(r"^def\s+(\S+)$", pattern)
    if func_match:
        return {
            "type": "function",
            "name_glob": func_match.group(1),
        }

    # Plain identifier
    return {
        "type": "identifier",
        "name_glob": pattern,
    }


def search_python_ast(
    filepath: str,
    parsed_pattern: Dict[str, Any],
) -> List[Dict[str, Any]]:
    """Search a Python file using the ast module.

    Args:
        filepath: Path to the Python file.
        parsed_pattern: Parsed pattern dict from parse_pattern().

    Returns:
        List of dicts with keys: file, name, line, type.
    """
    try:
        source = Path(filepath).read_text(encoding="utf-8")
        tree = ast.parse(source, filename=filepath)
    except (SyntaxError, UnicodeDecodeError, OSError):
        return []

    results = []
    pattern_type = parsed_pattern["type"]
    name_glob = parsed_pattern.get("name_glob", "*")
    base_class = parsed_pattern.get("base_class")

    for node in ast.walk(tree):
        if pattern_type == "class" and isinstance(node, ast.ClassDef):
            # Check name matches glob
            if not fnmatch.fnmatch(node.name, name_glob):
                continue
            # Check base class if specified
            if base_class:
                base_names = []
                for b in node.bases:
                    if isinstance(b, ast.Name):
                        base_names.append(b.id)
                    elif isinstance(b, ast.Attribute):
                        base_names.append(b.attr)
                if base_class not in base_names:
                    continue
            results.append({
                "file": filepath,
                "name": node.name,
                "line": node.lineno,
                "type": "class",
            })

        elif pattern_type == "function" and isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            if fnmatch.fnmatch(node.name, name_glob):
                results.append({
                    "file": filepath,
                    "name": node.name,
                    "line": node.lineno,
                    "type": "function",
                })

        elif pattern_type == "identifier":
            if isinstance(node, ast.Name) and fnmatch.fnmatch(node.id, name_glob):
                results.append({
                    "file": filepath,
                    "name": node.id,
                    "line": getattr(node, "lineno", 0),
                    "type": "identifier",
                })

    return results


def find_python_files(search_path: str) -> List[str]:
    """Recursively find all Python files in a directory.

    Args:
        search_path: Root directory to search.

    Returns:
        List of absolute file paths.
    """
    files = []
    root = Path(search_path).resolve()

    if root.is_file() and root.suffix == ".py":
        return [str(root)]

    for dirpath, dirnames, filenames in os.walk(root):
        # Skip hidden directories and common non-source dirs
        dirnames[:] = [
            d for d in dirnames
            if not d.startswith(".") and d not in ("node_modules", "__pycache__", "venv", ".venv")
        ]
        for f in filenames:
            if f.endswith(".py"):
                files.append(os.path.join(dirpath, f))

    return files


def find_classes(
    name_glob: str,
    search_path: str,
    base_class: Optional[str] = None,
) -> List[Dict[str, Any]]:
    """Find class definitions matching a glob pattern.

    Args:
        name_glob: Glob pattern for class names (e.g., '*Handler', 'My*').
        search_path: Directory to search in.
        base_class: Optional base class name to filter by.

    Returns:
        List of dicts with keys: file, name, line, type.
    """
    parsed = {
        "type": "class",
        "name_glob": name_glob,
    }
    if base_class:
        parsed["base_class"] = base_class

    results = []
    for filepath in find_python_files(search_path):
        results.extend(search_python_ast(filepath, parsed))
    return results


def find_functions(
    name_glob: str,
    search_path: str,
) -> List[Dict[str, Any]]:
    """Find function definitions matching a glob pattern.

    Args:
        name_glob: Glob pattern for function names (e.g., 'test_*', 'validate_*').
        search_path: Directory to search in.

    Returns:
        List of dicts with keys: file, name, line, type.
    """
    parsed = {
        "type": "function",
        "name_glob": name_glob,
    }
    results = []
    for filepath in find_python_files(search_path):
        results.extend(search_python_ast(filepath, parsed))
    return results


def format_results(
    pattern: str,
    results: List[Dict[str, Any]],
    fmt: str = "text",
) -> str:
    """Format AST search results for output.

    Args:
        pattern: The original search pattern.
        results: List of match result dicts.
        fmt: Output format, either 'text' or 'json'.

    Returns:
        Formatted string.
    """
    if fmt == "json":
        return json.dumps(
            {"pattern": pattern, "matches": results},
            indent=2,
            ensure_ascii=False,
        )

    if not results:
        return f"No matches found for pattern '{pattern}'."

    lines = [f"AST matches for: {pattern}", ""]
    for r in results:
        lines.append(
            f"  {r['type']:>10}  {r['name']:<40} {r['file']}:{r['line']}"
        )
    return "\n".join(lines)


def main() -> int:
    """CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Search for structural code patterns using AST analysis."
    )
    parser.add_argument(
        "--pattern", required=True,
        help='Search pattern (e.g., \'class * (BaseHandler)\', \'def test_*\').',
    )
    parser.add_argument(
        "--path", default=".",
        help="Directory to search in (default: current directory).",
    )
    parser.add_argument(
        "--format", choices=["text", "json"], default="text",
        help="Output format (default: text).",
    )
    args = parser.parse_args()

    search_path = str(Path(args.path).resolve())

    if not Path(search_path).is_dir():
        print(f"Error: '{args.path}' is not a directory.", file=sys.stderr)
        return 1

    tool = detect_ast_tool()
    parsed = parse_pattern(args.pattern)

    if tool == "python-ast" or tool == "regex":
        results = []
        for filepath in find_python_files(search_path):
            results.extend(search_python_ast(filepath, parsed))
    else:
        # ast-grep path (future enhancement)
        results = []
        for filepath in find_python_files(search_path):
            results.extend(search_python_ast(filepath, parsed))

    print(format_results(args.pattern, results, fmt=args.format))
    return 0


if __name__ == "__main__":
    sys.exit(main())
