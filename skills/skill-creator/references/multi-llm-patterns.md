# Multi-LLM & Script-Augmented Skill Design Patterns

This reference documents design patterns extracted from advanced skills like [`agent-council`](../skills/agent-council/SKILL.md) for building multi-LLM, multi-agent, or script-heavy skills.

## 1. Multi-Agent Delegation Pattern

When a skill requires consulting multiple models (e.g. Gemini, Claude, Copilot) or delegating tasks to sub-agents:

### Architecture

```text
my-multi-agent-skill/
├── SKILL.md                 # Primary instructions & execution entrypoint
├── config.yaml              # Member declarations & provider CLI commands
├── references/
│   ├── overview.md          # Architecture & delegation flow
│   └── safety.md            # Execution boundaries & safety rules
└── scripts/
    ├── main_orchestrator.py # Orchestrates parallel sub-processes
    └── tests/
        └── test_orchestrator.py
```

### Decoupled Configuration (`config.yaml`)

Keep provider models, CLI command templates, and timeouts in an explicit YAML configuration file rather than hardcoding them in scripts:

```yaml
council:
  members:
    - name: gemini
      command: "agy -p"
      emoji: "💎"
    - name: claude
      command: "claude -p"
      emoji: "🧠"
    - name: copilot
      command: "copilot -p"
      emoji: "✈️"

  settings:
    timeout: 120
```

---

## 2. Asynchronous Job & Worker Pattern

For tasks that take significant time (e.g., benchmark runs, multi-LLM synthesis):

1. **Job Spawning**: The skill script initializes a job directory (e.g., `.jobs/job-<id>/`) containing input params, status metadata (`status.json`), and raw log files.
2. **Non-Blocking Execution**: The job runs asynchronously in the background.
3. **Status Polling**: The host agent can inspect progress with lightweight status commands without blocking the main event loop.

---

## 3. Graceful Fallbacks & Degraded Operation

Skills that rely on external CLIs or sub-agents must gracefully degrade if a tool or provider is missing:

```python
import shutil

def check_cli_availability(cli_name: str) -> bool:
    """Returns True if the required CLI binary exists on system PATH."""
    return shutil.which(cli_name) is not None
```

- **Missing CLI Handling**: If a member CLI (e.g. `copilot`) is missing, mark its status as `missing_cli` in the job summary rather than crashing the entire workflow.
- **Permission Denied Fallback**: If nested CLI subprocess execution is blocked by sandbox permissions, fallback to single-agent local analysis with a clear diagnostic message.
