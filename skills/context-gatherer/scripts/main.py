#!/usr/bin/env python3
"""Main CLI entrypoint router for context-gatherer.

Routes context requests to specialized extraction tools:
- git_coupling.py: git temporal co-change coupling
- symbol_nav.py: symbol definitions and usage references
- ast_search.py: AST pattern matching
"""

import argparse
import sys
from pathlib import Path

_script_dir = Path(__file__).resolve().parent
if str(_script_dir) not in sys.path:
    sys.path.insert(0, str(_script_dir))

from ast_search import main as ast_main  # noqa: E402
from git_coupling import main as git_main  # noqa: E402
from symbol_nav import main as symbol_main  # noqa: E402


def parse_args():
    parser = argparse.ArgumentParser(description="Context Gatherer Router CLI")
    parser.add_argument("mode", choices=["git", "symbol", "ast"], help="Extraction mode (git, symbol, ast)")
    return parser.parse_known_args()


def main() -> int:
    args, unknown = parse_args()
    sys.argv = [sys.argv[0]] + unknown

    if args.mode == "git":
        return git_main()
    elif args.mode == "symbol":
        return symbol_main()
    elif args.mode == "ast":
        return ast_main()

    return 0


if __name__ == "__main__":
    sys.exit(main())
