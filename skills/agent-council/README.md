# Agent Council

Synthesize perspectives from multiple AI engines (Gemini, Claude, Copilot) using parallel CLI execution and chairman consensus.

---

## The Problem & Friction

Single-model answers carry inherent model bias, hallucination risks, and specialized blind spots. Relying on a single AI model for critical architecture or code auditing decisions can lead to narrow or flawed recommendations.

Agent Council fans out queries across multiple CLI tools in parallel, then synthesizes their independent analyses into a unified consensus report.

---

## Quickstart

```bash
# Asynchronous execution
JOB_DIR=$(cargo run -p agent-skills -- agent-council start "your question here")
cargo run -p agent-skills -- agent-council wait "$JOB_DIR"
cargo run -p agent-skills -- agent-council results "$JOB_DIR" # --verbose (default: true)
cargo run -p agent-skills -- agent-council clean "$JOB_DIR"

# One-shot synchronous query
cargo run -p agent-skills -- agent-council start "your question here"
```

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).
- **Architecture & Lifecycle**: See [references/overview.md](references/overview.md).
- **Member Configuration**: See [references/config.md](references/config.md).
