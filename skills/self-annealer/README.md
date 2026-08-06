# Self-Annealer

Enforce bounded self-healing repair loops for failing tests and lint errors with strict attempt limits and automated git rollback safeguards.

---

## The Problem & Friction

When automated tests fail, AI coding assistants often enter infinite retry loops, applying superficial symptom patches (swallowing exceptions, deleting failing tests, or adding arbitrary delays) that degrade codebase quality.

Self-Annealer prevents token drain and codebase corruption by enforcing a strict maximum retry limit (default: 3 iterations) and executing an automatic `git checkout` rollback if repair fails to converge.

---

## Quickstart

```bash
# Verify health of self-annealer skill files
cargo run -p agent-skills -- self-annealer check

# Run bounded self-healing repair loop
cargo run -p agent-skills -- self-annealer run --cmd "cargo test" --max-iterations 3
```

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).
- **Repair Invariants & Architecture**: See [references/overview.md](references/overview.md).
