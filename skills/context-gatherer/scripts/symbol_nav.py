#!/usr/bin/env python3
"""Symbol navigation — find definitions and references of symbols.

Uses ripgrep (rg) when available, falls back to grep.
Detects common definition patterns for Python, JavaScript/TypeScript, Go, and Rust.

Uses Python 3 Standard Library only (plus external grep/rg via subprocess).
"""

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

# Definition patterns per language extension
DEFINITION_PATTERNS = {
    ".py": [
        r"^\s*def\s+{symbol}\s*\(",
        r"^\s*class\s+{symbol}\s*[:\(]",
        r"^\s*{symbol}\s*=",
    ],
    ".js": [
        r"^\s*function\s+{symbol}\s*\(",
        r"^\s*(?:const|let|var)\s+{symbol}\s*=",
        r"^\s*class\s+{symbol}\s*[\{{\s]",
        r"^\s*export\s+(?:default\s+)?(?:function|class|const|let|var)\s+{symbol}",
    ],
    ".ts": [
        r"^\s*function\s+{symbol}\s*[\(<]",
        r"^\s*(?:const|let|var)\s+{symbol}\s*[=:]",
        r"^\s*(?:export\s+)?class\s+{symbol}\s*[\{{\s<]",
        r"^\s*(?:export\s+)?interface\s+{symbol}\s*[\{{\s<]",
        r"^\s*(?:export\s+)?type\s+{symbol}\s*=",
    ],
    ".go": [
        r"^\s*func\s+(?:\([^)]*\)\s+)?{symbol}\s*\(",
        r"^\s*type\s+{symbol}\s+",
        r"^\s*var\s+{symbol}\s+",
    ],
    ".rs": [
        r"^\s*(?:pub\s+)?fn\s+{symbol}\s*[\(<]",
        r"^\s*(?:pub\s+)?struct\s+{symbol}\s*[\{{\s<]",
        r"^\s*(?:pub\s+)?enum\s+{symbol}\s*[\{{\s<]",
        r"^\s*(?:pub\s+)?trait\s+{symbol}\s*[\{{\s<]",
    ],
}

# File extensions to search by default
DEFAULT_EXTENSIONS = [
    ".py", ".js", ".ts", ".jsx", ".tsx", ".go", ".rs",
    ".java", ".rb", ".cpp", ".c", ".h", ".hpp", ".cs",
]


def detect_search_tool() -> str:
    """Detect the best available search tool.

    Returns:
        'rg' if ripgrep is available, otherwise 'grep'.
    """
    if shutil.which("rg"):
        return "rg"
    return "grep"


def search_files(
    pattern: str,
    search_path: str,
    tool: Optional[str] = None,
) -> List[Dict[str, Any]]:
    """Search for a pattern across files in a directory.

    Args:
        pattern: Text pattern to search for (literal string, not regex).
        search_path: Directory path to search in.
        tool: Search tool to use ('rg' or 'grep'). Auto-detected if None.

    Returns:
        List of dicts with keys: file, line_number, line.
    """
    if tool is None:
        tool = detect_search_tool()

    search_path = str(Path(search_path).resolve())

    if tool == "rg":
        cmd = [
            "rg", "--no-heading", "--line-number", "--with-filename",
            "--color=never", "--fixed-strings", pattern, search_path,
        ]
    else:
        cmd = [
            "grep", "-rnI", "--include=*.py", "--include=*.js",
            "--include=*.ts", "--include=*.go", "--include=*.rs",
            "--include=*.java", "--include=*.rb",
            pattern, search_path,
        ]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=30,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return []

    if not result.stdout.strip():
        return []

    results = []
    for line in result.stdout.splitlines():
        # Format: file:line_number:content
        parts = line.split(":", 2)
        if len(parts) >= 3:
            try:
                results.append({
                    "file": parts[0],
                    "line_number": int(parts[1]),
                    "line": parts[2].strip(),
                })
            except ValueError:
                continue

    return results


def find_definitions(
    symbol: str,
    search_path: str,
) -> List[Dict[str, Any]]:
    """Find definition sites for a symbol.

    Uses language-specific definition patterns (def, class, function, type, etc.)

    Args:
        symbol: Symbol name to find definitions for.
        search_path: Directory to search in.

    Returns:
        List of dicts with keys: file, line_number, line.
    """
    tool = detect_search_tool()
    search_path_resolved = str(Path(search_path).resolve())
    results = []

    # Collect all definition patterns across supported languages
    all_patterns = []
    for patterns in DEFINITION_PATTERNS.values():
        for pat in patterns:
            escaped_symbol = re.escape(symbol)
            all_patterns.append(pat.format(symbol=escaped_symbol))

    # Build a combined regex
    combined = "|".join(f"({p})" for p in all_patterns)

    if tool == "rg":
        cmd = [
            "rg", "--no-heading", "--line-number", "--with-filename",
            "--color=never", "-e", combined, search_path_resolved,
        ]
    else:
        cmd = [
            "grep", "-rnEI", combined, search_path_resolved,
        ]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=30,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return []

    for line in (result.stdout or "").splitlines():
        parts = line.split(":", 2)
        if len(parts) >= 3:
            try:
                results.append({
                    "file": parts[0],
                    "line_number": int(parts[1]),
                    "line": parts[2].strip(),
                })
            except ValueError:
                continue

    return results


def find_references(
    symbol: str,
    search_path: str,
) -> List[Dict[str, Any]]:
    """Find reference sites for a symbol (usages, excluding definitions).

    Args:
        symbol: Symbol name to find references for.
        search_path: Directory to search in.

    Returns:
        List of dicts with keys: file, line_number, line.
    """
    # Get all matches
    all_matches = search_files(symbol, search_path)

    # Get definitions to exclude
    definitions = find_definitions(symbol, search_path)
    def_keys = {
        (d["file"], d["line_number"]) for d in definitions
    }

    # References = all matches minus definitions
    return [
        m for m in all_matches
        if (m["file"], m["line_number"]) not in def_keys
    ]


def format_results(
    symbol: str,
    results: List[Dict[str, Any]],
    fmt: str = "text",
) -> str:
    """Format search results for output.

    Args:
        symbol: The symbol that was searched.
        results: List of match result dicts.
        fmt: Output format, either 'text' or 'json'.

    Returns:
        Formatted string.
    """
    if fmt == "json":
        return json.dumps(
            {"symbol": symbol, "matches": results},
            indent=2,
            ensure_ascii=False,
        )

    if not results:
        return f"No matches found for '{symbol}'."

    lines = [f"Matches for: {symbol}", ""]
    for r in results:
        lines.append(f"  {r['file']}:{r['line_number']}  {r['line']}")

    return "\n".join(lines)


def main() -> int:
    """CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Find definitions and references of a symbol in source code."
    )
    parser.add_argument(
        "--symbol", required=True,
        help="Symbol name to search for.",
    )
    parser.add_argument(
        "--path", default=".",
        help="Directory to search in (default: current directory).",
    )
    parser.add_argument(
        "--type", choices=["all", "definition", "reference"], default="all",
        help="Type of match to find (default: all).",
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

    if args.type == "definition":
        results = find_definitions(args.symbol, search_path)
    elif args.type == "reference":
        results = find_references(args.symbol, search_path)
    else:
        results = search_files(args.symbol, search_path)

    print(format_results(args.symbol, results, fmt=args.format))
    return 0


if __name__ == "__main__":
    sys.exit(main())
