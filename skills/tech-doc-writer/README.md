# Tech Doc Writer

A specialized Agent Skill for generating, refactoring, and auditing human-centric technical documentation (`README.md`, `ARCHITECTURE.md`, `CONTRIBUTING.md`, `docs/`) while preserving AI agent token budgets.

---

## The Problem & Friction

Technical documentation is often either too brief or diluted with marketing hyperbole. Furthermore, providing extensive writing guidelines directly inside an AI agent's prompt taxes token limits on every single message.

`tech-doc-writer` solves this by separating **agent execution protocols (`SKILL.md`)** from **on-demand writing directives (`references/writing-directives.md`)** and **automated Markdown linting (`agent-skills tech-doc-writer audit`)**.

---

## Quickstart & Audit Usage

Run the document auditor against your Markdown files:

```bash
agent-skills tech-doc-writer audit --path README.md ARCHITECTURE.md
```

### Audited Rules

- **Single H1 Title**: Ensures exactly one `# Title` per file to preserve GitHub Table of Contents structure.
- **GFM Callouts**: Validates `> [!NOTE]`, `> [!TIP]`, `> [!IMPORTANT]`, `> [!WARNING]`, and `> [!CAUTION]`.
- **Mermaid Diagrams**: Enforces `mermaid` blocks over ASCII text art.
- **Privacy Path Checks**: Ensures no hardcoded absolute user home directories.

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).
- **Writing Directives & Tone**: See [references/writing-directives.md](references/writing-directives.md).
- **GitHub Markdown Conventions**: See [references/gfm-conventions.md](references/gfm-conventions.md).
- **Document Skeletons**: See [references/doc-templates.md](references/doc-templates.md).
