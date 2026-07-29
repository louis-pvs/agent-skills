#!/usr/bin/env python3
"""What-If Analysis Blast Radius Calculator & Impact Analyzer.

Analyzes symbol references, caller locations, impacted test suites,
and documentation drift across a codebase using word-boundary regex matching.
"""

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, List


def resolve_project_dir(raw_path: str) -> Path:
    """Resolves and validates a user-supplied directory path without stripping absolute path prefixes."""
    project_dir = Path(raw_path).resolve()
    if not project_dir.exists() or not project_dir.is_dir():
        sys.stderr.write(f"Error: Directory '{project_dir}' does not exist or is not a directory.\n")
        sys.exit(1)
    return project_dir


_IGNORED_DIRS = frozenset({"__pycache__", "node_modules", "venv", "graphify-out", ".jobs"})


def _is_test_file(file_path: str) -> bool:
    """Determines if a file is a test file based on path components and naming conventions."""
    p = Path(file_path)
    return any(part in ("test", "tests") for part in p.parts) or p.stem.startswith("test_") or p.stem.endswith("_test")


def _is_doc_file(file_path: str) -> bool:
    """Determines if a file is documentation based on extension or path components."""
    return file_path.endswith(".md") or "docs" in Path(file_path).parts


def find_symbol_callers(target_symbol: str, project_dir: Path) -> List[Dict[str, Any]]:
    """Searches project files for occurrences/references to the target symbol."""
    callers: List[Dict[str, Any]] = []
    symbol_regex = re.compile(rf"\b{re.escape(target_symbol)}\b")

    for file_path in project_dir.rglob("*"):
        if not file_path.is_file():
            continue
        # Skip git, venv, graphify-out, and binary/cache dirs
        if any(part.startswith(".") or part in _IGNORED_DIRS for part in file_path.parts):
            continue
        if file_path.suffix not in (".py", ".md", ".sh", ".yaml", ".yml", ".json"):
            continue

        try:
            content = file_path.read_text(encoding="utf-8", errors="ignore")
            for line_idx, line in enumerate(content.splitlines(), start=1):
                if symbol_regex.search(line):
                    callers.append(
                        {
                            "file": str(file_path.relative_to(project_dir)),
                            "line": line_idx,
                            "content": line.strip(),
                        }
                    )
        except Exception:
            continue

    return callers


def analyze_blast_radius(target_symbol: str, project_dir: Path) -> Dict[str, Any]:
    """Computes total blast radius metrics for a target symbol across code, tests, and docs."""
    references = find_symbol_callers(target_symbol, project_dir)

    code_callers = []
    test_files = []
    doc_files = []

    for ref in references:
        file_path = ref["file"]
        if _is_test_file(file_path):
            if file_path not in test_files:
                test_files.append(file_path)
        elif _is_doc_file(file_path):
            if file_path not in doc_files:
                doc_files.append(file_path)
        else:
            code_callers.append(ref)

    unique_code_files = list(dict.fromkeys(ref["file"] for ref in code_callers))
    total_impacted = len(unique_code_files) + len(test_files) + len(doc_files)

    if total_impacted > 10 or len(unique_code_files) > 5:
        risk_level = "HIGH"
    elif total_impacted > 3:
        risk_level = "MEDIUM"
    else:
        risk_level = "LOW"

    return {
        "symbol": target_symbol,
        "project_dir": str(project_dir.resolve()),
        "risk_level": risk_level,
        "total_impacted_files": total_impacted,
        "code_callers": code_callers,
        "impacted_code_files": unique_code_files,
        "impacted_test_files": test_files,
        "impacted_doc_files": doc_files,
    }


def generate_impact_report(analysis: Dict[str, Any]) -> str:
    """Formats analysis results into a clean GFM Markdown report."""
    symbol = analysis["symbol"]
    risk = analysis["risk_level"]

    report_lines = [
        f"# 🔮 What-If Analysis Blast Radius Report: `{symbol}`",
        "",
        f"**Target Symbol:** `{symbol}`  ",
        f"**Blast Radius Risk Level:** `{risk}`  ",
        f"**Total Impacted Files:** {analysis['total_impacted_files']}  ",
        "",
        "## 💥 Code Callers Blast Radius",
        "",
    ]

    if analysis["impacted_code_files"]:
        report_lines.append("| File | Line | Content Snippet |")
        report_lines.append("| :--- | :--- | :--- |")
        for caller in analysis["code_callers"]:
            snippet = caller["content"][:60].replace("|", r"\|")
            report_lines.append(f"| `{caller['file']}` | L{caller['line']} | `{snippet}` |")
        report_lines.append("")
    else:
        report_lines.append("_No direct code callers found._\n")

    report_lines.append("## 🧪 Impacted Test Suites (TDD Pre-Flight)")
    report_lines.append("")
    if analysis["impacted_test_files"]:
        for tf in analysis["impacted_test_files"]:
            report_lines.append(f"- [ ] `{tf}`")
        report_lines.append("")
    else:
        report_lines.append("_No directly associated test files detected. Recommend adding unit tests before refactoring._\n")

    report_lines.append("## 📚 Documentation Drift Check")
    report_lines.append("")
    if analysis["impacted_doc_files"]:
        for df in analysis["impacted_doc_files"]:
            report_lines.append(f"- [ ] `{df}`")
        report_lines.append("")
    else:
        report_lines.append("_No documentation references detected._\n")

    return "\n".join(report_lines)


def parse_args():
    parser = argparse.ArgumentParser(description="What-If Analysis Blast Radius Calculator")
    parser.add_argument("--symbol", type=str, required=True, help="Target symbol or function name to analyze")
    parser.add_argument("--dir", type=str, default=".", help="Project root directory (default: current dir)")
    parser.add_argument("--json", action="store_true", help="Output raw JSON data instead of GFM report")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    project_dir = resolve_project_dir(args.dir)

    analysis = analyze_blast_radius(args.symbol, project_dir)
    if args.json:
        print(json.dumps(analysis, indent=2))
    else:
        print(generate_impact_report(analysis))
    return 0


if __name__ == "__main__":
    sys.exit(main())
