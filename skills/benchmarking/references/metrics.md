# Metric Evaluators Reference Guide

The Benchmarking Skill provides native empirical performance metrics evaluated directly via `agent-skills benchmarking run`.

---

## 1. Core Native Evaluators

| Evaluator Metric | Measured Attribute          | Unit                | Assertion CLI Flag                | Description                                                          |
| :--------------- | :-------------------------- | :------------------ | :-------------------------------- | :------------------------------------------------------------------- |
| `duration_ms`    | Wall-clock execution time   | `ms`                | `--assert-max-duration-ms <ms>`   | Average wall-clock execution duration across statistical iterations. |
| `pass_ratio`     | Process exit status         | `ratio (0.0 - 1.0)` | `--assert-min-pass-ratio <ratio>` | Proportion of runs returning exit code 0.                            |
| `delta_ms`       | Differential baseline delta | `ms`                | N/A                               | Execution duration difference relative to `--baseline-cmd`.          |

---

## 2. Statistical Iteration & Variance

To eliminate transient noise and variance:

- Default iterations: 5 (`--iterations <n>`)
- Reports `min_duration_ms`, `max_duration_ms`, and `avg_duration_ms`.
- Pass ratio calculates `pass_count / iterations`.

```bash
# Example: Assert strict < 250ms avg duration and 100% pass ratio across 10 runs
agent-skills benchmarking run \
  --cmd "cargo test --workspace" \
  --iterations 10 \
  --assert-max-duration-ms 250 \
  --assert-min-pass-ratio 1.0 \
  --json
```
