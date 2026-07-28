# Agent Skills

A curated, open-source collection of procedural skills and workflows for AI coding assistants (Antigravity, Gemini CLI, Claude Code, Codex, and Cursor).

Instead of relying on unguided AI generation, these skills provide structured procedures, architectural guidelines, and multi-agent synthesis tools to ensure clean code execution and reliable decision-making.

---

## 🚀 Available Skills

| Skill | Description | Path |
| :--- | :--- | :--- |
| **`agent-council`** | Collect and synthesize opinions from multiple AI agents (Gemini, Claude, Copilot) into a unified recommendation. | [`skills/agent-council`](skills/agent-council/SKILL.md) |
| **`skill-creator`** | Create, scaffold, format, and audit new Agent Skills adhering to the agentskills.io standard and Python stdlib standards. | [`skills/skill-creator`](skills/skill-creator/SKILL.md) |

---

## 📦 Global Setup & Installation

To make these skills available automatically across all your projects without modifying individual codebases:

### Option 1: Global Gemini / Antigravity Registration (Recommended)

Register the `skills/` directory in your global configuration file `~/.gemini/config/skills.json`:

```json
{
  "entries": [
    { "path": "/path/to/agent-skills/skills" }
  ]
}
```

Once added, your AI agent will automatically discover and trigger these skills regardless of which project workspace you are working in.

### Option 2: Symlink into Project Customization Root

If you prefer repository-scoped registration:

```bash
mkdir -p .agents/skills
ln -s /path/to/agent-skills/skills/agent-council .agents/skills/agent-council
ln -s /path/to/agent-skills/skills/skill-creator .agents/skills/skill-creator
```

---

## 🛠 Repository Structure

```text
agent-skills/
├── README.md
├── .gitignore
├── .agents/
│   └── AGENTS.md             # Project-level agent rules & privacy guidelines
├── docs/
│   └── adr/                  # Architectural Decision Records (ADRs)
└── skills/
    ├── agent-council/        # Multi-agent consensus synthesis skill
    └── skill-creator/        # Skill authoring, scaffolding, & validation skill
```

---

## ✍️ Creating New Skills

Each skill folder inside `skills/` must follow the Agent Skill standard:

1. **`SKILL.md`**: Must contain YAML frontmatter defining `name` and `description`:

   ```yaml
   ---
   name: my-new-skill
   description: Concise description of when the agent should trigger this skill.
   ---
   ```

2. **Body**: Keep instructions under 500 lines. Place supplementary material in a `references/` subdirectory.
3. **Scripts**: Use **Python 3 Standard Library First** for any executable scripts in `scripts/` with corresponding unit tests in `scripts/tests/`.

---

## 📄 License

MIT License. Free for use in personal and open-source projects.
