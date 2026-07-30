---
name: agent-council
description: Collect and synthesize opinions from multiple AI agents. Use when users say "summon the council", "ask other AIs", or want multiple AI perspectives on a question.
---

# Agent Council

Collect multiple AI opinions and synthesize one answer using parallel CLI execution.

## Usage

### Asynchronous Lifecycle Execution (Recommended for AI Agents)

For background tracking and non-blocking execution across member CLIs:

```bash
JOB_DIR=$(python3 skills/agent-council/scripts/council.py start "your question here")
python3 skills/agent-council/scripts/council.py wait "$JOB_DIR"
python3 skills/agent-council/scripts/council.py results "$JOB_DIR"
python3 skills/agent-council/scripts/council.py clean "$JOB_DIR"
```

### Direct One-Shot Execution

For quick, synchronous query and result synthesis:

```bash
python3 skills/agent-council/scripts/council.py "your question here"
```

## References

- [overview.md](references/overview.md) — Workflow and multi-agent synthesis background.
- [examples.md](references/examples.md) — Usage examples.
- [config.md](references/config.md) — Member configuration in `council.config.yaml`.
- [requirements.md](references/requirements.md) — Python 3 requirements and CLI checks.
- [host-ui.md](references/host-ui.md) — Host UI checklist guidance.
- [safety.md](references/safety.md) — Safety guidelines.

---

## Completion Criteria

- [ ] Parallel CLI sub-agent processes spawned with repo root CWD context.
- [ ] Member responses collected and synthesized without unhandled process exceptions.
- [ ] Job logs and temporary process files cleaned up after execution.
