---
name: skill-creator
description: Create, scaffold, format, and audit new Agent Skills adhering to the agentskills.io open specification, progressive disclosure, and predictable design principles.
---

# Skill Creator

Guide and automate the creation of high-quality, standardized, predictable Agent Skills.

## Overview

An **Agent Skill** is a structured directory containing procedural knowledge, rules, guidelines, templates, and optional automation scripts. Skills allow AI coding assistants (Antigravity, Gemini CLI, Claude Code, Cursor, Codex) to perform specialized, complex workflows reliably.

## Procedural Workflow

When creating a new skill, follow these 5 steps:

### 1. Interview & Requirements Gathering

Determine the scope, invocation mode, and architecture of the skill:

- **Interactive Decision-Making**: When interviewing the user, clarifying scope, or resolving architectural choices, the agent **MUST** call the `ask_question` tool following the [Interactive Decision-Making Protocol](references/interactive-decisions.md):
  - Place the recommended choice first with a `(Recommended)` prefix.
  - Phrase options from the user's direct intent/perspective.
  - Do not add manual "Other" options (provided by UI).
  - Ask one question at a time down the decision tree.
- **Skill Name**: Lowercase alphanumeric with hyphens (`^[a-z0-9-]+$`), max 64 chars.
- **Invocation Mode**:
  - **Model-Invoked**: Autonomous trigger. Description sits in prompt context window (**pays Context Load**). Write rich trigger phrasing ("Use when the user wants...").
  - **User-Invoked**: Explicit user command only (e.g. `/my-skill`). Set `disable-model-invocation: true` in YAML frontmatter (**pays zero Context Load**, pays Cognitive Load).
- **Skill Type**:
  - **Simple (`--type simple`)**: Pure procedural guidance or style rules in `SKILL.md`.
  - **Complex (`--type complex`)**: Multi-resource / script-augmented (includes `references/`, `templates/`, `examples/`).
  - **Router (`--type router`)**: Lightweight user-invoked menu that triages requests to specialized sub-skills without payload execution logic.

### 2. Scaffold Skill Directory

Use the native Rust CLI:

```bash
# Model-invoked complex skill (Native Rust CLI)
agent-skills skill-creator scaffold \
  --name "my-new-skill" \
  --description "Description of when to trigger this skill." \
  --type complex

# User-invoked router skill (zero context load)
agent-skills skill-creator scaffold \
  --name "my-router-skill" \
  --description "Short human summary." \
  --type router \
  --user-invoked
```

Directory Structure:

```text
skills/my-new-skill/
├── SKILL.md                 # AI agent instruction protocol (lean)
├── README.md                # Human developer landing page (GFM)
├── references/              # On-demand reference documentation
├── scripts/
│   └── tests/
├── templates/
└── examples/
```

### 3. Enforce Progressive Disclosure & Load Management

Optimize context and cognitive load:

1. **Discovery Tier**: Keep `SKILL.md` YAML frontmatter concise (~100 tokens).
2. **Activation Tier**: Keep `SKILL.md` body concise (< 500 lines). Focus on high-level workflow, decision trees, and guardrails (**Context Load Audit**).
   - Ensure branching decisions explicitly instruct the agent to use `ask_question`.
3. **Execution Tier**: Offload detailed API schemas, extensive documentation, or reference manuals into `references/` (**Cognitive Load Audit**).
4. **Human Landing Tier**: Use **`tech-doc-writer`** directives (`agent-skills tech-doc-writer audit`) to maintain human-facing `README.md` entry points without inflating `SKILL.md` token load.

### 4. Checkable Completion Criteria & CLI Standards

- **Checkable Completion Criteria**: Every step or skill must end with explicit, checkable completion criteria (e.g. `- [ ] Tests pass cleanly (exit 0)`) to prevent premature completion.
- **Automation Standards**:
  - **Native Rust CLI**: Implement subcommands in `crates/cli/src/commands/` for zero-dependency execution.
  - **Testing**: Include unit tests and CLI contract tests runnable via:

    ```bash
    cargo test --workspace
    ```

### 5. Audit & Validate

Run `agent-skills skill-creator validate --path skills/<skill-name>` to verify:

- **YAML Frontmatter Syntax**: Ensure valid YAML formatting. If `description` or any string field contains colons (`:`), quotes, or special characters, enclose the value in single quotes (`'...'`) or double quotes (`"..."`).
- **Invocation Mode Compliance**: Validate name, description, and invocation mode flag (`disable-model-invocation`).
- **Interactive Decision Compliance**: Ensure branching logic mandates `ask_question` adhering to [interactive-decisions.md](references/interactive-decisions.md).
- Checkable **Completion Criteria** section presence (`## Completion Criteria`).
- **Context Load** (lines <= 500) and **Cognitive Load** branch checks.
- Test pass status via `cargo test`.

## Completion Criteria

- [ ] Frontmatter YAML syntax validated (no unquoted colons `:` in description or metadata).
- [ ] Frontmatter metadata verified with valid name and description.
- [ ] Invocation mode explicitly chosen (`disable-model-invocation` set for command-only skills).
- [ ] Interactive decisions formatted with `ask_question` tool protocol.
- [ ] Checkable completion criteria included in `SKILL.md`.
- [ ] Workspace tests passing cleanly (`cargo test --workspace`).

## References & Resources

- [interactive-decisions.md](references/interactive-decisions.md) — Standard specification for interactive decision-making with `ask_question`.
- [agentskills-spec.md](references/agentskills-spec.md) — Full `agentskills.io` open standard specification.
- [multi-llm-patterns.md](references/multi-llm-patterns.md) — Multi-agent delegation & configuration patterns.
- [scripting-standards.md](references/scripting-standards.md) — Repository scripting and testing standards.
- [simple-skill.md](templates/simple-skill.md) — Template for markdown-only skills.
- [complex-skill-structure.md](templates/complex-skill-structure.md) — Template for complex skills.
- [router-skill.md](templates/router-skill.md) — Template for router skills.
