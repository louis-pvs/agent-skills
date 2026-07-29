#!/usr/bin/env python3
"""What-If Analysis Phase 3 AST Call-Graph Parser.

Uses Python Standard Library AST NodeVisitor traversal to extract semantic function
and method call sites, distinguishing invocations from string literals and comments.
"""

import argparse
import ast
import json
import os
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional


class CallGraphVisitor(ast.NodeVisitor):
    """AST NodeVisitor that tracks function definitions and AST call sites."""

    def __init__(self, target_symbol: str):
        self.target_symbol = target_symbol
        self.call_sites: List[Dict[str, Any]] = []
        self.current_function: Optional[str] = None
        self.current_class: Optional[str] = None

    def visit_ClassDef(self, node: ast.ClassDef):
        prev_class = self.current_class
        self.current_class = node.name
        self.generic_visit(node)
        self.current_class = prev_class

    def visit_FunctionDef(self, node: ast.FunctionDef):
        prev_func = self.current_function
        self.current_function = node.name
        self.generic_visit(node)
        self.current_function = prev_func

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef):
        prev_func = self.current_function
        self.current_function = node.name
        self.generic_visit(node)
        self.current_function = prev_func

    def visit_Call(self, node: ast.Call):
        func_name = None
        if isinstance(node.func, ast.Name):
            func_name = node.func.id
        elif isinstance(node.func, ast.Attribute):
            func_name = node.func.attr

        if func_name == self.target_symbol:
            self.call_sites.append(
                {
                    "symbol": self.target_symbol,
                    "line": node.lineno,
                    "col": node.col_offset,
                    "caller_func": self.current_function or "<global>",
                    "caller_class": self.current_class or "<none>",
                }
            )
        self.generic_visit(node)


def extract_ast_call_sites(target_symbol: str, source_code: str) -> List[Dict[str, Any]]:
    """Extracts semantic AST call sites for target_symbol from Python source code."""
    try:
        tree = ast.parse(source_code)
        visitor = CallGraphVisitor(target_symbol)
        visitor.visit(tree)
        return visitor.call_sites
    except SyntaxError:
        return []


def resolve_safe_path(raw_path: str, base_dir: Optional[Path] = None) -> Path:
    """Sanitizes user-supplied input paths against Path Traversal vulnerabilities."""
    base = (base_dir if base_dir is not None else Path.cwd()).resolve()
    target = Path(raw_path).expanduser().resolve()
    if not str(target).startswith(str(base)) or os.path.commonpath([str(base), str(target)]) != str(base):
        sys.stderr.write(f"Security Error: Path traversal attempt detected. '{raw_path}' escapes base directory '{base}'.\n")
        sys.exit(1)
    return target


def parse_args():
    parser = argparse.ArgumentParser(description="What-If Analysis Phase 3 AST Call-Graph Parser")
    parser.add_argument("--symbol", type=str, required=True, help="Target symbol to parse AST call sites for")
    parser.add_argument("--file", type=str, required=True, help="Target Python source file to parse")
    parser.add_argument("--json", action="store_true", help="Output raw JSON data")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        file_path = resolve_safe_path(args.file)
    except SystemExit:
        return 1

    if not file_path.exists() or not file_path.is_file():
        sys.stderr.write(f"Error: File '{file_path}' does not exist.\n")
        return 1

    content = file_path.read_text(encoding="utf-8", errors="ignore")
    call_sites = extract_ast_call_sites(args.symbol, content)

    if args.json:
        print(json.dumps(call_sites, indent=2))
    else:
        print(f"AST Call Sites for '{args.symbol}' in {file_path.name}: {len(call_sites)}")
        for cs in call_sites:
            print(f"  L{cs['line']}:{cs['col']} in function '{cs['caller_func']}'")
    return 0


if __name__ == "__main__":
    sys.exit(main())
