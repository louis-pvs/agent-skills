# Domain Modeling

Design, formalize, and model business domain entities, ubiquitous language, aggregate boundaries, value objects, domain events, and state machine lifecycles using Domain-Driven Design (DDD) principles.

---

## Quickstart

```bash
# Verify health of domain-modeling skill files
cargo run -p agent-skills -- domain-modeling check

# Scaffold a pure domain entity stub
cargo run -p agent-skills -- domain-modeling scaffold-entity Order
```

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).
- **Architecture & References**: See [references/overview.md](references/overview.md).
