---
name: what-if-analysis
description: Prospective simulation, blast-radius calculation, dependency impact modeling, sensitivity scenario evaluation, and test suite ripple analysis before refactoring or committing code changes.
---

# What-If Analysis

Perform prospective blast-radius calculation, symbol-reference dependency mapping, sensitivity scenario modeling, test suite impact estimation, and doc drift detection before making code changes.

## Usage

### 1. Direct CLI Blast Radius Execution (Phase 1)

To calculate the blast radius of a target function, class, or symbol:

```bash
python3 skills/what-if-analysis/scripts/impact_analyzer.py --symbol <symbol_name>
```

Output raw JSON metadata:

```bash
python3 skills/what-if-analysis/scripts/impact_analyzer.py --symbol <symbol_name> --json
```

### 2. Scenario Tradeoff Matrix & Sensitivity Probes (Phase 2)

To model and compare trade-offs across competing architectural options or parameter configurations:

```bash
python3 skills/what-if-analysis/scripts/scenario_runner.py --symbol <symbol_name> --scenarios "Option A: Async Dispatch, Option B: Subprocess Fork"
```

---

## Consumer System Fallback (Soft Dependency)

`what-if-analysis` automatically detects whether `agent-council` and external AI CLIs (`claude`, `gemini`, `copilot`) are available:

- **Full Environment**: Runs parallel multi-model probes via `agent-council`.
- **Consumer System (Fallback)**: Gracefully falls back to single-agent analytical heuristics (`Antigravity`) without hard-failing or requiring external CLIs.

---

## Output Structure

1. **Blast Radius Report** (`impact_analyzer.py`): Target Symbol, Risk Level (`LOW`/`MEDIUM`/`HIGH`), Code Callers Table, Impacted Test Suites (TDD Pre-Flight), and Documentation Drift Check.
2. **Scenario Tradeoff Matrix** (`scenario_runner.py`): Scenario Comparison Matrix across Latency Impact, Blast Radius Risk, Maintenance Cost, Confidence Level, and Execution Mode / Fallback Notice.

---

## References & Multi-Phase Roadmap

- [blast-radius.md](references/blast-radius.md) — Heuristics, risk level formulas, and symbol reference matching rules.
- [roadmap.md](references/roadmap.md) — Multi-phase evolution (Phase 1: Blast Radius, Phase 2: Sensitivity & Scenario Modeling [Shipped], Phase 3: Counterfactual Test Generation).
