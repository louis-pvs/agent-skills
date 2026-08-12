# Skill Creator

Guide and automate the creation of high-quality, standardized, predictable Agent Skills adhering to the agentskills.io specification and Progressive Disclosure design principles.

---

## Quickstart

```bash
# Scaffold a complex model-invoked skill (Rust CLI)
agent-skills skill-creator scaffold \
  --name "my-new-skill" \
  --description "Description of when to trigger this skill." \
  --type complex

# Validate an existing skill directory
agent-skills skill-creator validate --path skills/my-new-skill
```

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).
- **AgentSkills Specification**: See [references/agentskills-spec.md](references/agentskills-spec.md).
- **Multi-LLM Delegation Patterns**: See [references/multi-llm-patterns.md](references/multi-llm-patterns.md).
- **Scripting & Testing Standards**: See [references/scripting-standards.md](references/scripting-standards.md).
