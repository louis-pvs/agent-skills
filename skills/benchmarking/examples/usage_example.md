# Benchmarking Skill Usage Examples

This guide provides concrete examples of running performance benchmarks, comparing against baselines, and integrating benchmarking telemetry into automated repair loops.

---

## Example 1: Basic Command Benchmark

Measure wall-clock execution time and peak memory RSS over 5 iterations:

```bash
agent-skills benchmarking run \
  --cmd "cargo test --workspace" \
  --iterations 5
```

### Console Output

```text
📊 Benchmark Report: cargo test --workspace
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
agent-skills benchmarking run \
  --cmd "cargo test --package agent-skills --test cli_contract_tests" \
  --baseline-cmd "cargo test --workspace" \
  --iterations 5 \
  --json
```

---

## Example 3: Integration with `self-annealer` Repair Loop

Enforce automated rollback if performance degrades beyond threshold (e.g. > 100ms runtime):

```bash
agent-skills self-annealer run \
  --cmd "agent-skills benchmarking run --cmd 'cargo test' --assert-max-duration-ms 100" \
  --max-iterations 3
```
