# Benchmarking Skill Usage Examples

This guide provides concrete examples of running performance benchmarks, comparing against baselines, and integrating benchmarking telemetry into automated repair loops.

---

## Example 1: Basic Command Benchmark

Measure wall-clock execution time and peak memory RSS over 5 iterations:

```bash
python3 skills/benchmarking/scripts/benchmark_runner.py \
  --cmd "python3 -m unittest discover -s scripts/tests" \
  --iterations 5
```

### Console Output

```text
📊 Benchmark Report: python3 -m unittest discover -s scripts/tests
Status: PASS
Avg Wall Time: 42.15 ms
Avg Peak Memory: 1.85 MB
  - [PASS] timing: 42.15 ms (Wall-clock runtime 42.15 ms)
  - [PASS] memory: 1.85 MB (Peak RSS allocation 1.85 MB)
  - [PASS] pass_ratio: 1.0 ratio (Process completed cleanly (exit code 0))
```

---

## Example 2: Baseline Differential Comparison

Compare performance of a refactored implementation against a baseline command:

```bash
python3 skills/benchmarking/scripts/benchmark_runner.py \
  --cmd "python3 scripts/optimized_main.py" \
  --baseline-cmd "python3 scripts/legacy_main.py" \
  --iterations 5 \
  --json
```

### Structured JSON Report

```json
{
  "command": "python3 scripts/optimized_main.py",
  "iterations": 5,
  "summary": {
    "status": "pass",
    "avg_wall_time_ms": 12.4,
    "avg_peak_memory_mb": 2.1,
    "exit_code": 0
  },
  "metrics": [
    {
      "name": "timing",
      "status": "pass",
      "value": 12.4,
      "unit": "ms",
      "threshold": null,
      "detail": "Wall-clock runtime 12.40 ms",
      "raw": { "wall_time_ms": 12.4 }
    }
  ],
  "baseline": {
    "command": "python3 scripts/legacy_main.py",
    "avg_wall_time_ms": 28.6,
    "avg_peak_memory_mb": 4.5,
    "wall_time_delta_pct": -56.64
  }
}
```

---

## Example 3: Integration with `self-annealer` Repair Loop

Enforce automated rollback if performance degrades beyond threshold (e.g. > 100ms runtime):

```bash
python3 skills/self-annealer/scripts/anneal_runner.py \
  --cmd "python3 skills/benchmarking/scripts/benchmark_runner.py --cmd 'python3 scripts/main.py' --assert-max-duration-ms 100" \
  --max-iterations 3
```
