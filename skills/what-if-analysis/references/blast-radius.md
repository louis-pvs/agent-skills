# Blast Radius Calculation & Heuristics

Detailed reference for calculating blast radius and risk categorization in `what-if-analysis`.

## Risk Level Classification Heuristics

| Risk Level | Impacted Files Threshold | Code Callers Threshold | Action Required |
| :--- | :--- | :--- | :--- |
| **LOW** | 1 – 3 files | 1 – 2 caller files | Standard TDD workflow (`RED` -> `GREEN`). |
| **MEDIUM** | 4 – 10 files | 3 – 5 caller files | Create dedicated feature branch; execute full unit test suite. |
| **HIGH** | > 10 files | > 5 caller files | Multi-perspective review (`agent-council`); pre-refactor architectural audit. |

## Symbol Caller Matching Rules

1. **AST Parsing**: Target symbols are matched using word boundary regex (`\b<symbol>\b`) against `.py`, `.md`, `.sh`, `.yaml`, `.json` files.
2. **Directory Filtering**: Local build caches (`__pycache__`), virtualenvs (`venv`), git objects (`.git`), test runner jobs (`.jobs`), and local graph outputs (`graphify-out`) are automatically ignored.
3. **Test Suite Mapping**: Any file containing `test` in its name or directory path that references the target symbol is flagged for pre-flight verification.
