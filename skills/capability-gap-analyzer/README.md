# Capability Gap Analyzer

Measure capability distance between registered agent skills and target project domains using deterministic sub-capability checklist scans, and scaffold missing skill proposals via `skill-creator`.

---

## Overview

The **Capability Gap Analyzer** evaluates your local workspace and global AI agent skill inventories (`~/.gemini/config/skills/`, `~/.claude/skills/`, `~/.copilot/skills/`) against domain taxonomy checklists (Architecture, Refactoring, Benchmarking, Frontend, Backend, DevOps, Security).

It computes real coverage fractions (`covered sub-capabilities / total sub-capabilities`) to highlight **Strong**, **Partial**, and **Zero-Zone** capability coverage, automatically filtering out-of-scope domain categories to prevent false positive gap warnings.

---

## Quick Usage

```bash
# Verify health of capability-gap-analyzer skill files
cargo run -p agent-skills -- capability-gap-analyzer check

# Auto-detect project domain mix and analyze skill coverage
cargo run -p agent-skills -- capability-gap-analyzer analyze --auto-detect

# Analyze an explicit target domain (e.g. frontend-web, devops-infra)
cargo run -p agent-skills -- capability-gap-analyzer analyze --domain frontend-web

# Output structured JSON metadata
cargo run -p agent-skills -- capability-gap-analyzer analyze --json
```

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md)
- **Architecture & Design**: See [references/overview.md](references/overview.md)
