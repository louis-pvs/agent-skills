# Architecture Auditor

Audit codebases against modern software design principles (SOLID, DRY, YAGNI, KISS, CUPID) and resolve architectural principle tensions.

---

## Quickstart

```bash
# Verify health of architecture-auditor skill files
cargo run -p agent-skills -- architecture-auditor check

# Analyze structural metrics for a single file
cargo run -p agent-skills -- architecture-auditor analyze --file path/to/file.rs
```

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).
- **Audit Workflow & Triage**: See [references/overview.md](references/overview.md).
- **SOLID Principles**: See [references/solid.md](references/solid.md).
- **DRY & YAGNI**: See [references/dry-yagni.md](references/dry-yagni.md).
- **KISS Principle**: See [references/kiss.md](references/kiss.md).
- **CUPID Properties**: See [references/cupid.md](references/cupid.md).
- **Principle Tensions**: See [references/principle-tensions.md](references/principle-tensions.md).
- **Audit Report Template**: See [references/audit-report.md](references/audit-report.md).
