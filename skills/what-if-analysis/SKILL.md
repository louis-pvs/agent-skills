---
name: what-if-analysis
description: Prospective simulation, blast-radius calculation, dependency impact modeling, and test suite ripple analysis before refactoring or committing code changes.
---

# What-If Analysis

Perform prospective blast-radius calculation, symbol-reference dependency mapping, test suite impact estimation, and doc drift detection before making code changes.

## Usage

### Direct CLI Blast Radius Execution

To calculate the blast radius of a target function, class, or symbol:

```bash
python3 skills/what-if-analysis/scripts/impact_analyzer.py --symbol <symbol_name>
```

Output raw JSON metadata:

```bash
python3 skills/what-if-analysis/scripts/impact_analyzer.py --symbol <symbol_name> --json
```

---

## Output Structure

The skill generates a structured Markdown report containing:

1. **Target Symbol & Risk Level**: Categorized as `LOW`, `MEDIUM`, or `HIGH` risk based on total impacted file count.
2. **Code Callers Table**: Precise file paths, line numbers, and code snippets referencing the symbol.
3. **Impacted Test Suites (TDD Pre-Flight)**: List of test files containing references to the target symbol.
4. **Documentation Drift Check**: List of documentation files (`.md`) referencing the symbol.

---

## Agent Council Integration

When blast-radius analysis reports a `HIGH` risk rating (> 10 impacted files or > 5 core caller files), option to invoke `agent-council` for multi-perspective architectural review:

```bash
python3 skills/agent-council/scripts/council.py "What are the trade-offs of refactoring <symbol> given high blast-radius impact?"
```

---

## References & Future Roadmap

- [blast-radius.md](references/blast-radius.md) — Heuristics, risk level formulas, and symbol reference matching rules.
- [roadmap.md](references/roadmap.md) — Multi-phase evolution (Phase 2: Sensitivity Modeling, Phase 3: Counterfactual Test Generation).
