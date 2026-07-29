#!/usr/bin/env python3
"""What-If Analysis Phase 3 Counterfactual Test Generator.

Generates adversarial RED unit test cases for anticipated failure modes
before refactoring code (extending TDD).
"""

import argparse
import os
import sys
from pathlib import Path
from typing import List, Optional


def generate_counterfactual_test(symbol: str, params: List[str], module_name: str = "module") -> str:
    """Generates counterfactual adversarial RED test code for a target function signature."""
    null_args = ", ".join(["None"] * len(params)) if params else ""
    empty_args = ", ".join(['""'] * len(params)) if params else ""

    class_name = "TestCounterfactual" + "".join(word.capitalize() for word in symbol.split("_"))

    test_lines = [
        "#!/usr/bin/env python3",
        f'"""Counterfactual adversarial tests for target symbol `{symbol}`."""',
        "",
        "import unittest",
        f"from {module_name} import {symbol}",
        "",
        "",
        f"class {class_name}(unittest.TestCase):",
        f"    def test_counterfactual_{symbol}_null_check(self) -> None:",
        '        """Assert behavior when target symbol is invoked with null arguments."""',
        "        with self.assertRaises((TypeError, ValueError, AttributeError)):",
        f"            {symbol}({null_args})",
        "",
        f"    def test_counterfactual_{symbol}_empty_boundary(self) -> None:",
        '        """Assert behavior when target symbol is invoked with empty boundary arguments."""',
        "        with self.assertRaises((TypeError, ValueError, AttributeError)):",
        f"            {symbol}({empty_args})",
        "",
        "",
        'if __name__ == "__main__":',
        "    unittest.main()",
        "",
    ]
    return "\n".join(test_lines)


def resolve_safe_path(raw_path: str, base_dir: Optional[Path] = None) -> Path:
    """Sanitizes user-supplied input paths against Path Traversal vulnerabilities."""
    base = (base_dir if base_dir is not None else Path.cwd()).resolve()
    target = Path(raw_path).expanduser().resolve()
    if not str(target).startswith(str(base)) or os.path.commonpath([str(base), str(target)]) != str(base):
        sys.stderr.write(f"Security Error: Path traversal attempt detected. '{raw_path}' escapes base directory '{base}'.\n")
        sys.exit(1)
    return target


def parse_args():
    parser = argparse.ArgumentParser(description="What-If Analysis Counterfactual Test Generator")
    parser.add_argument("--symbol", type=str, required=True, help="Target symbol to generate counterfactual tests for")
    parser.add_argument("--params", type=str, default="", help="Comma-separated parameter names")
    parser.add_argument("--module", type=str, default="module", help="Target module name to import from")
    parser.add_argument("--out", type=str, help="Output file path for generated test")
    parser.add_argument("--dry-run", action="store_true", help="Preview generated test without writing to disk")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    param_list = [p.strip() for p in args.params.split(",") if p.strip()]

    test_code = generate_counterfactual_test(args.symbol, param_list, args.module)
    if args.dry_run:
        print(f"[DRY-RUN] Would write counterfactual test for '{args.symbol}' to: {args.out or 'stdout'}")
        return 0

    if args.out:
        try:
            out_path = resolve_safe_path(args.out)
        except SystemExit:
            return 1
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(test_code, encoding="utf-8")
        print(f"Generated counterfactual test: {out_path}")
    else:
        print(test_code)
    return 0


if __name__ == "__main__":
    sys.exit(main())
