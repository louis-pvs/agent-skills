# Benchmarking

Empirical performance benchmarking, baseline differential analysis, and expandable metric evaluations to prevent quality regressions and performance hallucinations.

---

## Quickstart

```bash
# Run baseline benchmark
python3 skills/benchmarking/scripts/benchmark_runner.py --cmd "python3 main.py" --save-baseline

# Run differential analysis post-patch
python3 skills/benchmarking/scripts/benchmark_runner.py --cmd "python3 main.py" --compare-baseline
```

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).
- **Execution Pipeline & Invariants**: See [references/overview.md](references/overview.md).
- **Metric Evaluators Guide**: See [references/metrics.md](references/metrics.md).
- **Benchmarking Philosophy**: See [references/philosophy.md](references/philosophy.md).
