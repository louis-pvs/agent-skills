# Configure Members

Edit `council.config.yaml` to configure the chairman and council members:

```yaml
council:
  chairman:
    role: "antigravity"
  members:
    - name: antigravity
      command: "agy -p"
      emoji: "💎"
      color: "GREEN"
    - name: copilot
      command: "copilot -p"
      emoji: "✈️"
      color: "BLUE"
    - name: claude
      command: "claude -p"
      emoji: "🧠"
      color: "CYAN"
```

## Adding Custom Members

Add custom members by appending entries to `members`:

- Use a stable `name` (lowercase, short identifier).
- Set `command` to a runnable CLI invocation (e.g., `claude -p`, `agy -p`, `copilot -p`).
- Provide `emoji` and `color` for readability.
- If a configured CLI is not found in `PATH` or fails to respond, it is marked as `missing_cli` or `no_response` and explicitly reported in the final council report.
