# Agent Council

Synthesize perspectives from multiple AI engines (Antigravity, Claude, Copilot) using parallel CLI execution and chairman consensus with transparent member status reporting.

---

## The Problem & Friction

Single-model answers carry inherent model bias, hallucination risks, and specialized blind spots. Relying on a single AI model for critical architecture or code auditing decisions can lead to narrow or flawed recommendations. Furthermore, when member tools fail silently or are missing, orchestrators can produce incomplete results without warning the user.

Agent Council fans out queries across multiple CLI tools in parallel, tracks the health and availability of all member agents, and synthesizes independent analyses into a unified consensus report while explicitly reporting any unavailable or non-responsive CLIs.

---

## Environment Diagnostics (`doctor`)

Check member CLI availability, binary paths, and suggested fixes:

```bash
agent-skills agent-council doctor
```

---

## Quickstart

```bash
# Asynchronous execution
JOB_DIR=$(agent-skills agent-council start "your question here")
agent-skills agent-council wait "$JOB_DIR"
agent-skills agent-council results "$JOB_DIR" # --verbose (default: true)
agent-skills agent-council clean "$JOB_DIR"

# One-shot synchronous query
agent-skills agent-council start "your question here"
```

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).
- **Architecture & Lifecycle**: See [references/overview.md](references/overview.md).
- **Member Configuration**: See [references/config.md](references/config.md).
- **Requirements & Setup**: See [references/requirements.md](references/requirements.md).
- **Interactive Decisions**: See [references/interactive-decisions.md](references/interactive-decisions.md).
