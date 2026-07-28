---
name: agent-council
description: Collect and synthesize opinions from multiple AI agents. Use when users say "summon the council", "ask other AIs", or want multiple AI perspectives on a question.
---

# Agent Council

Collect multiple AI opinions and synthesize one answer.

## Usage

Run a job and collect results:

```bash
JOB_DIR=$(./scripts/council.sh start "your question here")
./scripts/council.sh wait "$JOB_DIR"
./scripts/council.sh results "$JOB_DIR"
./scripts/council.sh clean "$JOB_DIR"
```

One-shot:

```bash
./scripts/council.sh "your question here"
```

## References

- [overview.md](references/overview.md) — workflow and background.
- [examples.md](references/examples.md) — usage examples.
- [config.md](references/config.md) — member configuration.
- [requirements.md](references/requirements.md) — dependencies and CLI checks.
- [host-ui.md](references/host-ui.md) — host UI checklist guidance.
- [safety.md](references/safety.md) — safety notes.
