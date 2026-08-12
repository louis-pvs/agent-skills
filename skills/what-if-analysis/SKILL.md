---
name: what-if-analysis
description: Prospective simulation, blast-radius calculation, AST call-graph parsing, counterfactual test generation, and pre-emptive failure risk analysis before refactoring or committing code changes.
enhances:
  - agent-council
  - tdd
  - self-annealer
  - context-gatherer
version: 1.0.0
---

# What-If Analysis

Perform prospective blast-radius calculation, symbol-reference dependency mapping, AST call-graph node traversal, sensitivity scenario modeling, counterfactual RED test case generation, and pre-emptive failure risk analysis before making code changes.

## T-Shape Domain Scope & Boundary

- **Predictive Domain (`what-if-analysis`)**: Forward-looking & simulation-driven. Focuses on predicting consequences before making code changes—blast-radius risk calculation (`impact_analyzer.py`), multi-option scenario tradeoff matrices (`scenario_runner.py`), AST call-graph node traversal (`ast_graph_parser.py`), adversarial RED counterfactual test generation (`counterfactual_generator.py`), and shift-left failure pre-emption (`failure_preemptor.py`).
- **Descriptive Domain (`context-gatherer`)**: Backward-looking & exploratory. For mapping existing codebase structure, git history coupling, and architectural graph queries, use `context-gatherer`.

## Usage

All capabilities are reachable through the Rust CLI:

```bash
agent-skills what-if-analysis impact --symbol <symbol_name>
```

### 1. Direct CLI Blast Radius Execution (Phase 1)

To calculate the blast radius of a target function, class, or symbol:

```bash
agent-skills what-if-analysis impact --symbol <symbol_name>
```

Output raw JSON metadata:

```bash
agent-skills what-if-analysis impact --symbol <symbol_name> --json
```

### 2. Scenario Tradeoff Matrix & Sensitivity Probes (Phase 2)

To model and compare trade-offs across competing architectural options or parameter configurations:

```bash
agent-skills what-if-analysis scenario --symbol <symbol_name>
```

### 3. AST Call-Graph Parsing & Counterfactual Test Generation (Phase 3)

Parse AST call sites, generate counterfactual RED test cases, and pre-empt runtime failure modes:

```bash
# AST Call-Graph Node Traversal
agent-skills what-if-analysis ast --symbol <symbol_name> --file <file_path>

# Generate Adversarial RED Counterfactual Unit Test
agent-skills what-if-analysis counterfactual --symbol <symbol_name>

# Pre-emptive Failure Mode Risk Interception
agent-skills what-if-analysis preempt --symbol <symbol_name>
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
3. **AST Call-Graph & Pre-emption Report** (`ast_graph_parser.py` & `failure_preemptor.py`): Semantic call sites, boundary checking, and shift-left failure recommendations.

---

## Completion Criteria

- [ ] Blast-radius calculation executed without errors or missing caller files.
- [ ] Test files (`_is_test_file`) and doc drift (`_is_doc_file`) accurately classified.
- [ ] Scenario comparison matrix rendered with explicit fallback notice when council is absent.
- [ ] AST call-graph parser, counterfactual test generator, and failure pre-emptor scripts pass all unit tests cleanly.
- [ ] Code formatting passes `ruff check .` and `ruff format --check .` without errors.

---

## References & Resources

- [overview.md](references/overview.md) — Comprehensive architectural overview and component design.
- [blast-radius.md](references/blast-radius.md) — Heuristics, risk level formulas, and symbol reference matching rules.
- [overview.md](references/overview.md) — Multi-phase evolution (Phase 1: Blast Radius, Phase 2: Sensitivity Modeling, Phase 3: Counterfactuals & Pre-emptive Healing [Shipped]).
- [usage_example.md](examples/usage_example.md) — Step-by-step workflow examples.
- [counterfactual_template.py](templates/counterfactual_template.py) — Starter template for adversarial RED unit tests.
