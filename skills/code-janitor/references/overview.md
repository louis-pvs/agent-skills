# Code Janitor Overview

The Code Janitor skill provides automated code hygiene maintenance for Python codebases.

## Architecture

The skill consists of three layers:

1. **Instruction Protocol** (`SKILL.md`): AI agent procedural workflow for scoping,
   scanning, refactoring, and reporting.
2. **Reference Documentation** (`references/`): On-demand heuristics, smell catalogs,
   and report templates loaded via progressive disclosure.
3. **Automation Script** (`scripts/janitor_runner.py`): Python standard library CLI tool
   that performs AST-based static analysis to detect code smells.

## Relationship to Other Skills

- **`architecture-auditor`**: Evaluates high-level design principles (SOLID, CUPID).
  Code Janitor focuses on file/function-level hygiene.
- **`self-annealer`**: Provides bounded repair loops. Code Janitor can be used within
  an anneal loop to fix detected smells incrementally.
- **`tdd`**: Test-Driven Development workflow. Code Janitor respects test guard rails
  (run tests before/after fixes) to prevent regressions.
