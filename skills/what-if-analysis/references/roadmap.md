# What-If Analysis Multi-Phase Roadmap

Evolution path for the `what-if-analysis` skill across development phases.

## Phase 1: Blast-Radius & Dependency Impact Modeling (Current Release)

- **Regex Symbol Matching**: Word-boundary symbol reference search across project tree.
- **Test Suite Ripple Detection**: Identification of test files requiring pre-flight execution via strict path matching (`_is_test_file`).
- **Documentation Drift Tracking**: Detection of stale documentation references via `.md` and `docs/` path filtering (`_is_doc_file`).
- **Markdown Blast Radius Reports**: Automated GFM report generation with pipe-escaped code table snippets.

---

## Phase 2: Sensitivity & Parameter Scenario Modeling (Planned)

- **Performance Envelope Projections**: Modeling input parameter variation against latency baselines (extending `benchmarking`).
- **Architectural Tradeoff Matrices**: Automated comparison of architectural choices (e.g. REST vs gRPC, Subprocess vs Async Dispatch).
- **Parallel Subagent Branching**: Integration with `agent-council` to run isolated scenario probes in parallel.

---

## Phase 3: Counterfactual Test Generation & Pre-emptive Healing (Planned)

- **AST Call-Graph Traversal**: Deep AST parsing to distinguish definition vs invocation call sites.
- **Counterfactual Test Case Auto-Generation**: Auto-writing `RED` adversarial test cases for anticipated failure modes before refactoring (extending `tdd`).
- **Failure Mode Shift-Left**: Intercepting runtime bugs before `self-annealer` needs to repair them reactively.
