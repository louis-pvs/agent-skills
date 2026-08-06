# Tech Doc Writer Overview

Comprehensive architectural overview and reference guide for the `tech-doc-writer` skill.

## Purpose & Scope

`tech-doc-writer` provides technical documentation directives and GFM quality rules for generating, refactoring, and auditing human-centric documentation (`README.md`, `ARCHITECTURE.md`, `CONTRIBUTING.md`, `docs/*.md`).

## Core Components

- **Agent Protocol (`SKILL.md`)**: Progressive Disclosure workflow for AI coding assistants.
- **On-Demand Directives (`references/writing-directives.md`)**: The 5 Technical Writing Directives for human-centric tone and information density.
- **GFM Conventions (`references/gfm-conventions.md`)**: Standardized formatting rules for GitHub Flavored Markdown alerts, Mermaid diagrams, and collapsibles.
- **Document Skeletons (`references/doc-templates.md`)**: Structural skeletons for standard project documentation.
- **Document Auditor (`agent-skills tech-doc-writer audit`)**: Compiled Rust CLI tool enforcing single H1 rules, alert callout validation, Mermaid block checks, and path privacy rules.

## CLI Usage

```bash
# Audit single or multiple Markdown files
cargo run -p agent-skills -- tech-doc-writer audit --path README.md ARCHITECTURE.md
```
