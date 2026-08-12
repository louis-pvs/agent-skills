---
name: benchmarking
description: 'Empirical performance benchmarking, baseline differential analysis, expandable metric evaluations, blind-spot probing, and data-driven verification. Use when verifying execution speed, memory footprint, assertion ratios, or comparing performance before and after code changes.'
requires:
  - tdd
enhances:
  - self-annealer
  - architecture-auditor
version: 1.0.0
---

# Benchmarking

Empirical ground-truth verification engine that replaces unverified AI assertions and code "slop" with repeatable performance measurements, expandable metric evaluations, and data-driven feedback loops.

## Philosophy: Empirical Proof vs. Hallucinated Slop

> **"How do we know what we said is true is true, and how do we know what we don't know?"**

- **Ground Truth over Vibes**: An assertion about code speed or memory efficiency is merely a hypothesis until empirically measured. Benchmarking runs code under controlled conditions to collect quantitative proof.
- **Uncovering Blind Spots**: Edge-probing and differential baseline testing expose unhandled performance bottlenecks, memory leaks, and regression risks that static inspection misses.
- **Expandable Metrics**: Rather than hardcoding fixed metrics, benchmarking uses an expandable plugin architecture (`MetricEvaluator`) allowing teams to plug in custom evaluators (lint scores, code complexity, coverage deltas, security metrics).

---

## Procedural Workflow

Follow these 4 steps when evaluating performance or verifying code changes:

### 1. Identify Target Command & Baseline

Determine the command to benchmark (e.g. test runner, script, CLI entrypoint) and optional baseline command:

```bash
agent-skills benchmarking run \
  --cmd "cargo test" \
  --iterations 5
```

### 2. Run Metric Evaluations & Assertions

Execute benchmark runs with target assertion thresholds:

```bash
# Verify max runtime threshold (< 500 ms) and 100% pass ratio
agent-skills benchmarking run \
  --cmd "cargo test" \
  --assert-max-duration-ms 500 \
  --assert-min-pass-ratio 1.0 \
  --json
```

### 3. Load Custom Metric Plugins (Expandable Metrics)

Plug in custom metric evaluators from project directories:

```bash
agent-skills benchmarking run \
  --cmd "cargo test --workspace" \
  --metrics "timing,memory,pass_ratio"
```

### 4. Feed Telemetry to Continuous Improvement Loops

If metrics fail or regress, feed structured JSON reports directly to `self-annealer` or rollback uncommitted changes:

```bash
agent-skills self-annealer run \
  --cmd "agent-skills benchmarking run --cmd 'cargo test' --assert-max-duration-ms 200" \
  --max-iterations 3
```

---

## Metric Plugin System Architecture

Every metric evaluator implements the standard interface:

```python
from skills.benchmarking.scripts.evaluators.base import BenchmarkContext, MetricResult, MetricStatus


class CustomMetricEvaluator:
    name = "custom_metric"
    requires = ()

    def configure(self, config: dict) -> None:
        self.limit = config.get("limit", 100)

    def evaluate(self, context: BenchmarkContext) -> MetricResult:
        val = 42
        status = MetricStatus.PASS if val <= self.limit else MetricStatus.FAIL
        return MetricResult(
            name=self.name,
            status=status,
            value=val,
            unit="score",
            threshold=self.limit,
            detail=f"Score {val} (limit {self.limit})",
        )


METRIC = CustomMetricEvaluator()
```

---

## Completion Criteria

- [ ] Target command benchmarked cleanly with statistical iterations.
- [ ] Metric evaluators executed and results aggregated into JSON/console output.
- [ ] Assertions validated (`--assert-max-duration-ms`, `--assert-min-pass-ratio`).
- [ ] Baseline differential comparison verified (if `--baseline-cmd` specified).
- [ ] Exit code 0 returned on pass, 1 on threshold failure.

---

## References

- [philosophy.md](references/philosophy.md) — Ground-truth epistemology, statistical variance, confidence intervals, and eliminating AI slop.
- [metrics.md](references/metrics.md) — Built-in evaluators (`timing`, `memory`, `pass_ratio`) and custom metric plugin development guide.
