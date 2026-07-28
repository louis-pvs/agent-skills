# Configure members

Edit `council.config.yaml` to set chairman and members:

```yaml
council:
  chairman:
    role: "gemini"
  members:
    - name: gemini
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

Add custom members by appending entries to `members`:

- Use a stable `name` (lowercase, short).
- Set `command` to a runnable CLI invocation.
- Provide `emoji` and `color` for readability (optional but recommended).
- Note that missing CLIs will report `missing_cli` at runtime; remove unavailable members or install the CLI before running.
