#!/usr/bin/env python3
"""What-If Analysis Phase 3 Failure Pre-emptor.

Evaluates pre-emptive risk factors across AST call sites and test suites to intercept
runtime failures before self-annealer is needed.
"""

import argparse
import ast
import json
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


def preempt_failure_modes(symbol: str, project_dir: Path) -> Dict[str, Any]:
    """Identifies pre-emptive failure risks for a target symbol across project files."""
    risks: List[Dict[str, Any]] = []

    for file_path in project_dir.rglob("*.py"):
        if any(part.startswith(".") or part in ("venv", "node_modules", "graphify-out") for part in file_path.parts):
            continue
        try:
            content = file_path.read_text(encoding="utf-8", errors="ignore")
            tree = ast.parse(content)
            for node in ast.walk(tree):
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == symbol:
                    param_names = [arg.arg for arg in node.args.args]
                    body_code = ast.unparse(node) if hasattr(ast, "unparse") else content
                    has_zero_guard = "== 0" in body_code or "!= 0" in body_code or "is None" in body_code

                    for child in ast.walk(node):
                        if isinstance(child, ast.BinOp) and isinstance(child.op, ast.Div):
                            line_no = getattr(child, "lineno", node.lineno)
                            if isinstance(child.right, ast.Constant) and child.right.value == 0:
                                risks.append(
                                    {
                                        "file": str(file_path.relative_to(project_dir)),
                                        "line": line_no,
                                        "risk_type": "ZeroDivisionRisk",
                                        "severity": "HIGH",
                                        "recommendation": "Add explicit non-zero boundary check before calculation.",
                                    }
                                )
                            elif isinstance(child.right, ast.Name) and child.right.id in param_names:
                                if not has_zero_guard:
                                    risks.append(
                                        {
                                            "file": str(file_path.relative_to(project_dir)),
                                            "line": line_no,
                                            "risk_type": "UnguardedDivisionRisk",
                                            "severity": "MEDIUM",
                                            "recommendation": f"Add zero guard check for parameter '{child.right.id}'.",
                                        }
                                    )
        except Exception:
            continue

    return {
        "symbol": symbol,
        "project_dir": str(project_dir.resolve()),
        "preemptive_risks": risks,
        "total_risks": len(risks),
    }


def parse_args():
    parser = argparse.ArgumentParser(description="What-If Analysis Failure Mode Pre-emptor")
    parser.add_argument("--symbol", type=str, required=True, help="Target symbol to analyze for preemptive risks")
    parser.add_argument("--dir", type=str, default=".", help="Project root directory (default: current dir)")
    parser.add_argument("--json", action="store_true", help="Output raw JSON data")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    project_dir = resolve_project_dir(args.dir)

    analysis = preempt_failure_modes(args.symbol, project_dir)
    if args.json:
        print(json.dumps(analysis, indent=2))
    else:
        print(f"Failure Pre-emption Report for '{args.symbol}': {analysis['total_risks']} risks found.")
        for r in analysis["preemptive_risks"]:
            line_str = f":L{r['line']}" if "line" in r else ""
            print(f"  [{r['severity']}] {r['file']}{line_str}: {r['risk_type']} -> {r['recommendation']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
