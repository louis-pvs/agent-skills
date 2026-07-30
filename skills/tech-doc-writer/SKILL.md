---
name: tech-doc-writer
description: "Generate, refactor, and audit human-facing technical documentation (README.md, ARCHITECTURE.md, CONTRIBUTING.md, docs/) using technical writing directives and GFM conventions."
disable-model-invocation: false
enhances:
  - domain-modeling
version: 1.0.0
---

# Tech Doc Writer

Generate and audit high-impact, human-centric technical documentation for GitHub projects while keeping agent token usage lean via Progressive Disclosure.

## Invocation Modes & Model Toggling

- **Agent/LLM Autonomous Activation**: Triggered automatically when creating, updating, or reviewing human-facing Markdown files (`README.md`, `ARCHITECTURE.md`, `CONTRIBUTING.md`, `docs/*.md`).
- **User Explicit Activation**: Triggered via `/tech-doc-writer` slash command.
- **Model Tier Selection**:
  - **Standard Tier (Fast / Budget)**: Use for formulaic documentation like `CHANGELOG.md`, `CONTRIBUTING.md`, or simple README scaffolding.
  - **High-Reasoning Tier**: Toggle to high-reasoning models when writing `ARCHITECTURE.md` trade-offs, security specifications, or system design analyses.

## Procedural Workflow

When tasked with generating or updating technical documentation, follow these 4 steps:

### 1. Identify Target Document & Scope

Determine the type of document required:

- `README.md` — Project entry point, friction humanization, quickstart, architecture summary.
- `ARCHITECTURE.md` — Deep system topology, data flow, component trade-offs.
- `CONTRIBUTING.md` — Developer setup, guidelines, workflow rules.
- `docs/*.md` — Specialized specs, design docs, API reference manuals.

### 2. Load On-Demand References

Read the required reference documents before drafting content:

- [writing-directives.md](references/writing-directives.md) — Voice, tone, and the 5 Technical Writing Directives.
- [gfm-conventions.md](references/gfm-conventions.md) — GitHub Flavored Markdown rules (Alerts, Mermaid, collapsibles).
- [doc-templates.md](references/doc-templates.md) — Skeletons for README, Architecture, and Contributing guides.
- **Domain Models & Diagrams**: For `ARCHITECTURE.md`, if domain concepts exist, refer to `domain-modeling` to source Bounded Context maps, Ubiquitous Language glossaries, and Mermaid state diagrams.

### 3. Draft Content Applying Technical Writing Directives

- **Humanize the Friction**: Document the real pain/frustration solved, not just feature bullet points.
- **Vocabulary Shift**: Reframe technical constructs into systemic roles; purge empty buzzwords.
- **Staccato Beat**: Strip unnecessary conjunctions; use punchy vertical logic.
- **Context Economy**: Treat human attention as finite; maximize information per sentence.
- **Surgical Subversion**: Expose second-order trade-offs and structural limitations.
- **Mermaid Diagrams**: Use `mermaid` blocks for visual flows (never ASCII art).

### 4. Audit & Validate Document

Run the document auditor to verify GFM compliance, single H1, alert syntax, and path privacy:

```bash
python3 skills/tech-doc-writer/scripts/doc_auditor.py path/to/document.md
```

## Completion Criteria

- [ ] Target document drafted or updated.
- [ ] On-demand references (`writing-directives.md`, `gfm-conventions.md`) applied.
- [ ] Visual diagrams use standard GFM `mermaid` syntax (no ASCII art).
- [ ] `doc_auditor.py` executed with exit code 0.
- [ ] Privacy check passed (no hardcoded absolute user home paths).

## References

- [writing-directives.md](references/writing-directives.md) — Deep tone, voice, and technical writing rules.
- [gfm-conventions.md](references/gfm-conventions.md) — GitHub Markdown conventions & alerts.
- [doc-templates.md](references/doc-templates.md) — Skeletons for standard project documentation.
