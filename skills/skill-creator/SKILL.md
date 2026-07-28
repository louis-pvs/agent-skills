---
name: skill-creator
description: Create, scaffold, format, and audit new Agent Skills adhering to the agentskills.io open specification, progressive disclosure, and Python standard library scripting standards.
---

# Skill Creator

Guide and automate the creation of high-quality, standardized Agent Skills.

## Overview

An **Agent Skill** is a structured directory containing procedural knowledge, rules, guidelines, templates, and optional automation scripts. Skills allow AI coding assistants (Antigravity, Gemini CLI, Claude Code, Cursor, Codex) to perform specialized, complex workflows reliably.

## Procedural Workflow

When creating a new skill, follow these 5 steps:

### 1. Interview & Requirements Gathering

Determine the scope and architecture of the skill:

- **Skill Name**: Lowercase alphanumeric with hyphens (`^[a-z0-9-]+$`), max 64 chars.
- **Trigger Description**: What specific user intent or key terms should trigger this skill? (Max 1024 chars).
- **Skill Tier**:
  - **Simple (Markdown-only)**: Pure procedural guidance or style rules in `SKILL.md`.
  - **Complex (Multi-resource / Script-augmented)**: Requires `references/`, `scripts/`, `assets/`, `templates/`, or `examples/`.

### 2. Scaffold Skill Directory

Use the included automation script:

```bash
python3 skills/skill-creator/scripts/scaffold_skill.py \
  --name "my-new-skill" \
  --description "Description of when to trigger this skill." \
  --complex
```

Or manually create the structure:

```text
skills/my-new-skill/
├── SKILL.md
├── references/
├── scripts/
│   └── tests/
├── templates/
└── examples/
```

### 3. Enforce Progressive Disclosure

To optimize agent context windows:

1. **Discovery Tier**: Keep `SKILL.md` YAML frontmatter concise (~100 tokens).
2. **Activation Tier**: Keep `SKILL.md` body concise (< 500 lines). Focus on high-level workflow, decision trees, and guardrails.
3. **Execution Tier**: Offload detailed API schemas, extensive documentation, or reference manuals into `references/`.

### 4. Enforce Scripting & Testing Standards

If adding scripts to `scripts/`:

- **Language**: Python 3 using **Standard Library First** (no mandatory `pip install` dependencies).
- **Path Management**: Always use `pathlib.Path` for cross-platform OS compatibility (`/` vs `\`).
- **CLI Parsing**: Use `argparse` for clean argument interface.
- **Testing**: Include unit tests under `scripts/tests/test_<script_name>.py` runnable via:

  ```bash
  python3 -m unittest discover -s scripts/tests
  ```

### 5. Audit & Validate

Run `scaffold_skill.py --validate skills/<skill-name>` to verify:

- YAML frontmatter format compliance.
- Description character limit and triggering clarity.
- File references link check.
- Python script test pass status.

## References & Resources

- [agentskills-spec.md](references/agentskills-spec.md) — Full `agentskills.io` open standard specification.
- [multi-llm-patterns.md](references/multi-llm-patterns.md) — Multi-agent delegation & configuration patterns.
- [scripting-standards.md](references/scripting-standards.md) — Repository scripting and testing standards.
- [simple-skill.md](templates/simple-skill.md) — Template for markdown-only skills.
- [complex-skill-structure.md](templates/complex-skill-structure.md) — Template for complex skills.
- [python-script-template.py](templates/python-script-template.py) — Template for standard Python scripts.
- [python-test-template.py](templates/python-test-template.py) — Template for Python script unit tests.
