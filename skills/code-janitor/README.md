# Code Janitor

Automated code hygiene: detect and fix dead code, unused imports, code smells, oversized functions, deep nesting, missing docstrings, and stale TODOs. Use when the user requests code cleanup, dead code removal, code smell refactoring, or clean code enforcement.

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).
- **Architecture & References**: See [references/](references/) for clean code heuristics, smell catalog, and report template.

## Quick Start

### Scan a single file

```bash
python3 skills/code-janitor/scripts/janitor_runner.py --file path/to/file.py
```

### Scan an entire directory

```bash
python3 skills/code-janitor/scripts/janitor_runner.py --dir path/to/module/
```

### JSON output for programmatic consumption

```bash
python3 skills/code-janitor/scripts/janitor_runner.py --file path/to/file.py --json
```

## Running Tests

```bash
python3 -m unittest discover -s skills/code-janitor/scripts/tests
```
