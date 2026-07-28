---
name: agent-council
description: Collect and synthesize opinions from multiple AI agents. Use when users say "summon the council", "ask other AIs", or want multiple AI perspectives on a question.
---

# Agent Council

Collect multiple AI opinions and synthesize one answer using parallel CLI execution.

## Usage

Run a job and collect results (via Python 3 orchestrator or bash wrapper):

```bash
JOB_DIR=$(python3 ./scripts/council.py start "your question here")
python3 ./scripts/council.py wait "$JOB_DIR"
python3 ./scripts/council.py results "$JOB_DIR"
python3 ./scripts/council.py clean "$JOB_DIR"
```

Or using the thin bash wrapper:

```bash
JOB_DIR=$(./scripts/council.sh start "your question here")
./scripts/council.sh wait "$JOB_DIR"
./scripts/council.sh results "$JOB_DIR"
./scripts/council.sh clean "$JOB_DIR"
```

One-shot execution:

```bash
python3 ./scripts/council.py "your question here"
```

## References

- [overview.md](references/overview.md) — Workflow and multi-agent synthesis background.
- [examples.md](references/examples.md) — Usage examples.
- [config.md](references/config.md) — Member configuration in `council.config.yaml`.
- [requirements.md](references/requirements.md) — Python 3 requirements and CLI checks.
- [host-ui.md](references/host-ui.md) — Host UI checklist guidance.
- [safety.md](references/safety.md) — Safety guidelines.
