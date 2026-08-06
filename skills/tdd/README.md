# TDD (Test-Driven Development)

Enforce Test-Driven Development (TDD) workflows: write failing test (RED), write minimal code (GREEN), and refactor with automated agent verification gates.

---

## Quickstart

```bash
# Verify RED gate (failing test)
cargo run -p agent-skills -- tdd --cmd "pytest tests/test_feature.py" --verify-red

# Verify GREEN gate (passing test)
cargo run -p agent-skills -- tdd --cmd "pytest tests/test_feature.py" --verify-green
```

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).
- **Core Principles & Gates**: See [references/overview.md](references/overview.md).
- **Supported Test Runners Matrix**: See [references/runners.md](references/runners.md).
