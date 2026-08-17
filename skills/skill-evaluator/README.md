# Skill Evaluator

Empirical Ground-Truth Verification Engine for Measuring Agent Skill Efficiency and Token ROI.

---

## Overview

The **Skill Evaluator** quantifies the empirical impact of agent skills. By running twin-session evaluations comparing baseline agent execution (standard tools only) against skill-enriched agent execution, it measures:

1. **Token Conservation**: Input, output, and peak context token reduction.
2. **Cognitive / Tool Reduction**: Conversation turn and tool invocation efficiency.
3. **Task Correctness**: Assertion passing rate and output accuracy.
4. **Economic ROI**: Monetary cost reduction and latency speedup.

---

## Quickstart CLI

```bash
# Check skill benchmark health
agent-skills skill-evaluator check --skill skill-evaluator

# Run evaluation with assertions
agent-skills skill-evaluator run \
  --skill graphify \
  --mock \
  --assert-min-token-savings 50.0

# Sync verified badges into SKILL.md
agent-skills skill-evaluator sync-badges --skill graphify
```

---

## Architecture & References

- [SKILL.md](SKILL.md) — Agent instruction protocol.
- [scorecard.md](references/scorecard.md) — The 4-Pillar Scorecard mathematical formulas.
- [scenarios.md](references/scenarios.md) — Declarative benchmark scenario authoring guide.
