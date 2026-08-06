# Benchmarking

Empirical performance benchmarking, baseline differential analysis, and expandable metric evaluations to prevent quality regressions and performance hallucinations.

---

## Quickstart

```bash
# Verify health of benchmarking skill files
cargo run -p agent-skills -- benchmarking check

# Run empirical performance benchmark
cargo run -p agent-skills -- benchmarking run --cmd "cargo test" --iterations 5
```

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).
- **Execution Pipeline & Invariants**: See [references/overview.md](references/overview.md).
- **Metric Evaluators Guide**: See [references/metrics.md](references/metrics.md).
- **Benchmarking Philosophy**: See [references/philosophy.md](references/philosophy.md).
