# Context Gatherer

Gather deep codebase context before making refactoring changes using git temporal coupling, symbol navigation, and AST-based pattern matching.

---

## Quickstart

```bash
# Git Temporal Coupling (find co-changing files)
agent-skills context-gatherer git-coupling --file path/to/file.py

# Symbol Navigation (find caller references & definitions)
agent-skills context-gatherer symbol-nav --symbol "MyClass" --path src/

# AST Structural Search
agent-skills context-gatherer ast-search --pattern "def test_.*" --path tests/
```

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).
- **Technique Triage & Decision Tree**: See [references/overview.md](references/overview.md).
- **Heuristics Guide**: See [references/technique-heuristics.md](references/technique-heuristics.md).
