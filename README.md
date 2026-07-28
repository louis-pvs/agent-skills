# Agent Skills

A curated, open-source collection of procedural skills and workflows for AI coding assistants (Antigravity, Gemini CLI, Claude Code, Codex, and Cursor).

Instead of relying on unguided AI generation, these skills provide structured procedures, architectural guidelines, and multi-agent synthesis tools to ensure clean code execution and reliable decision-making.

---

## 🚀 Available Skills

| Skill                    | Description                                                                                                               | Path                                                              |
| :----------------------- | :------------------------------------------------------------------------------------------------------------------------ | :---------------------------------------------------------------- |
| **`agent-council`**      | Collect and synthesize opinions from multiple AI agents (Gemini, Claude, Copilot) into a unified recommendation.          | [`skills/agent-council`](skills/agent-council/SKILL.md)           |
| **`context-gatherer`**   | Gather deep codebase context: git temporal coupling, symbol navigation, and AST-based structural search.                  | [`skills/context-gatherer`](skills/context-gatherer/SKILL.md)     |
| **`skill-creator`**      | Create, scaffold, format, and audit new Agent Skills adhering to the agentskills.io standard and Python stdlib standards. | [`skills/skill-creator`](skills/skill-creator/SKILL.md)           |
| **`tdd`**                | Enforce Test-Driven Development (TDD) workflows: write failing test (RED), write minimal code (GREEN), and refactor.      | [`skills/tdd`](skills/tdd/SKILL.md)                               |

---

## 📦 Global Setup & Installation

To make all skills in this repository available automatically across all your projects in **Gemini / Antigravity**, **Claude Code**, and **GitHub Copilot**:

### Automatic Global Setup (Recommended)

Run the included zero-dependency Python installer:

```bash
python3 scripts/install_global.py
```

This automatically creates global symbolic links for all skills in `skills/` across:

- **Gemini / Antigravity**: `~/.gemini/config/skills/`
- **Claude Code**: `~/.claude/skills/`
- **GitHub Copilot**: `~/.copilot/skills/`

To show planned actions without modifying files:

```bash
python3 scripts/install_global.py --dry-run
```

To remove global symlinks:

```bash
python3 scripts/install_global.py --unlink
```

### Manual Configuration Options

<details>
<summary>Click to view manual config details</summary>

#### Gemini / Antigravity (`skills.json`)

Add the repository's `skills/` path to `~/.gemini/config/skills.json`:

```json
{
  "entries": [{ "path": "/path/to/agent-skills/skills" }]
}
```

#### Project-Scoped Symlinking

Symlink specific skills into a project's `.agents/skills` folder:

```bash
mkdir -p .agents/skills
ln -s /path/to/agent-skills/skills/agent-council .agents/skills/agent-council
```

</details>

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
    ├── context-gatherer/     # Codebase context gathering (git coupling, symbol nav, AST search)
    ├── skill-creator/        # Skill authoring, scaffolding, & validation skill
    └── tdd/                  # Test-Driven Development (RED-GREEN-REFACTOR execution & verification) skill
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
