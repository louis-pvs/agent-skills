# Capability Gap Analyzer

Measure capability distance between registered agent skills and target project domains using deterministic sub-capability checklist scans, and scaffold missing skill proposals via `skill-creator`.

---

## Overview

The **Capability Gap Analyzer** evaluates your local workspace and global AI agent skill inventories (`~/.gemini/config/skills/`, `~/.claude/skills/`, `~/.copilot/skills/`) against domain taxonomy checklists (Architecture, Refactoring, Benchmarking, Frontend, Backend, DevOps, Security).

It computes real coverage fractions (`covered sub-capabilities / total sub-capabilities`) to highlight **Strong**, **Partial**, and **Zero-Zone** capability coverage, automatically filtering out-of-scope domain categories to prevent false positive gap warnings.

---

## Quick Usage

```bash
# Auto-detect project domain mix and analyze skill coverage
python3 skills/capability-gap-analyzer/scripts/main.py --auto-detect

# Analyze an explicit target domain (e.g. frontend-web, devops-infra)
python3 skills/capability-gap-analyzer/scripts/main.py --domain frontend-web

# Output structured JSON metadata and suggest missing skill scaffolds
python3 skills/capability-gap-analyzer/scripts/main.py --json --scaffold-missing

# Run unit tests
python3 -m unittest discover -s skills/capability-gap-analyzer/scripts/tests
```

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md)
- **Architecture & Design**: See [references/overview.md](references/overview.md)
