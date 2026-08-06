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
JOB_DIR=$(cargo run -p agent-skills -- agent-council start "your question here")
cargo run -p agent-skills -- agent-council wait "$JOB_DIR"
cargo run -p agent-skills -- agent-council results "$JOB_DIR"
cargo run -p agent-skills -- agent-council clean "$JOB_DIR"
```

### Direct One-Shot Execution

For quick, synchronous query and result synthesis:

```bash
cargo run -p agent-skills -- agent-council start "your question here"
```

## Headless Execution & Prompt Guidelines

> [!TIP]
> **Pass Self-Contained Context in Prompts**: Member CLIs execute headlessly in background sub-shells without interactive user access. When formulating council questions, inline all critical context (such as code diffs, file snippets, or issue descriptions) directly into the prompt string so member agents do not fail on interactive permission prompts or remote repo auth limits.
>
> **Missing CLI Pre-checking**: Executables are pre-checked via `shutil.which`. If a configured CLI is absent, it is safely marked as `missing_cli` in status output without crashing the orchestrator.

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
