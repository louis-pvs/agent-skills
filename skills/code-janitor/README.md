# Code Janitor

Automated code hygiene: detect and fix dead code, unused imports, code smells, oversized functions, deep nesting, missing docstrings, and stale TODOs. Use when the user requests code cleanup, dead code removal, code smell refactoring, or clean code enforcement.

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).
- **Architecture & References**: See [references/](references/) for clean code heuristics, smell catalog, and report template.

## Quick Start

### Verify health of code-janitor skill files

```bash
agent-skills code-janitor check
```

### Scan a single file

```bash
agent-skills code-janitor scan --file path/to/file.py
```

### Scan an entire directory

```bash
agent-skills code-janitor scan --dir path/to/module/
```

### JSON output for programmatic consumption

```bash
agent-skills code-janitor scan --file path/to/file.py --json
```
