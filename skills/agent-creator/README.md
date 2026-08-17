# Agent Creator

The **Agent Creator** skill guides and automates the creation of high-quality, specialized Antigravity Custom Agents (`.agents/agents/<name>.md` or `~/.gemini/config/agents/<name>.md`).

---

## Features

- **Standardized Schema**: Enforces official Antigravity YAML frontmatter and Markdown system prompt architecture.
- **Least-Privilege Tool Scoping**: Prevents tool bloat and ensures safe, role-scoped execution boundaries.
- **Interactive Decision Protocol**: Automatically equips generated agents with tidy interactive UI selection modals via `ask_question`.
- **Validation Suite**: Native Rust CLI subcommand (`agent-skills agent-creator validate`) to verify frontmatter, tool scoping, and prompt completeness.

---

## Directory Layout

```text
skills/agent-creator/
├── SKILL.md                 # Agent instruction protocol
├── README.md                # Human landing page
├── references/
│   ├── agent-schema.md      # Frontmatter schema specification
│   ├── least-privilege-tools.md # Role-based tool access matrix
│   └── interactive-protocol.md  # Interactive decision protocol
└── templates/
    ├── custom-agent-template.md # Base agent template
    └── qa-tdd-engineer.md   # QA & TDD agent template
```

---

## Quick Usage

### Scaffold a Custom Agent

```bash
agent-skills agent-creator scaffold \
  --name "qa-tdd-engineer" \
  --description "Specialized in Test-Driven Development and test suites." \
  --model "gemini-3.7-flash-thinking" \
  --tools "view_file,write_to_file,replace_file_content,run_command,ask_question" \
  --skills "tdd,self-annealer"
```

### Validate an Agent File

```bash
agent-skills agent-creator validate --path .agents/agents/qa-tdd-engineer.md
```
