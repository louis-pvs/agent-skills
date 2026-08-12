# Self Progress

Identify capability gaps, blind spots, and agent growth opportunities from a completed (or in-progress) conversation session, outputting a retrospective report to feed `skill-creator`.

---

## Overview

The `self-progress` skill acts as an in-flight and post-session growth retro for AI coding agents. While `/learn` captures mandatory user behavioral preferences and `capability-gap-analyzer` audits project-wide domain coverage, `self-progress` focuses on **session-level friction points**:

- Unhandled tool errors or execution failures
- Direct user corrections during a task
- Heavy web search / documentation lookups indicating missing agent knowledge
- High-frequency repetitive tool usage indicative of inefficient workarounds

Discovered gaps are classified into actionable next steps, with missing skill proposals routed straight into `skill-creator`.

---

## Quick Usage

```bash
# Verify health of self-progress skill files
agent-skills self-progress check

# Analyze a conversation transcript log file
agent-skills self-progress analyze --transcript /path/to/transcript.jsonl

# Output JSON signals for automated processing
agent-skills self-progress analyze --transcript /path/to/transcript.jsonl --json
```

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md)
- **Architecture & Design**: See [references/overview.md](references/overview.md)
