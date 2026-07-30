---
name: capability-gap-analyzer
description: Measures capability distance between registered agent skills and target project domains, using a Two-Tier Hybrid Architecture (Deterministic Inventory Scan + Dynamic LLM Semantic Evaluation) and automated domain auto-detection for non-specific prompts.
enhances:
  - skill-creator
  - context-gatherer
version: 1.0.0
---

# Capability Gap Analyzer

Measure the distance between currently registered agent skills and the requirements of any target project domain. Detect missing skillsets, calculate taxonomy heatmaps, and scaffold draft `SKILL.md` specifications for unhandled domain areas.

## Overview

The **Capability Gap Analyzer** uses a **Two-Tier Hybrid Architecture**:

1. **Tier 1 (Deterministic Inventory Baseline)**: On-the-fly parsing of all `skills/*/SKILL.md` manifests to build a structured inventory JSON of registered skills, descriptions, and metadata.
2. **Tier 2 (Dynamic LLM Semantic Evaluation)**: AI agent semantic reasoning that compares current skill capabilities against any target domain (even custom or unlisted ones like WebAssembly or Quantum Computing) without relying on rigid hardcoded keyword lists.

---

## Procedural Workflow

When executing a capability gap analysis request, follow this 4-step workflow:

```mermaid
flowchart TD
    A["1. Resolve Target Domain Intent"] --> B["2. Scan Skill Manifest Inventory (Tier 1)"]
    B --> C["3. Perform LLM & Matrix Gap Scoring (Tier 2)"]
    C --> D["4. Output Gap Report & Scaffold Missing Skills"]
```

### 1. Resolve Target Domain Intent

- **Explicit Domain**: User specifies target domain (e.g. `frontend-web`, `data-engineering`, `devops-infra`).
- **Non-Specific Domain**: If the prompt is ambiguous or unspecified, execute workspace auto-detection:

  ```bash
  python3 skills/capability-gap-analyzer/scripts/main.py --auto-detect
  ```

  The domain detector inspects workspace markers (`package.json`, `pyproject.toml`, `Cargo.toml`, `Dockerfile`, `.sql`, etc.) to infer the active project stack.

### 2. Scan Skill Manifest Inventory (Tier 1)

- Run the deterministic inventory scanner:

  ```bash
  python3 skills/capability-gap-analyzer/scripts/gap_analyzer.py --json
  ```

- This parses all `skills/*/SKILL.md` frontmatter descriptions and metadata on the fly into a structured inventory dataset.

### 3. Perform LLM & Matrix Gap Scoring (Tier 2)

- Combine the Tier 1 inventory dataset with Tier 2 LLM semantic evaluation.
- Classify skills across standard lifecycle domains:
  - **Architecture & DDD**
  - **Static Analysis & Testing**
  - **Performance & Benchmarking**
  - **Frontend & UI/UX**
  - **Backend & Data Pipelines**
  - **DevOps & Infrastructure**
  - **Security & Compliance**
- Assign coverage levels: **Strong** (75-100%), **Partial** (25-74%), or **Zero-Zone** (0-24%).

### 4. Output Gap Report & Scaffold Missing Skills

- Render the GFM Taxonomy Heatmap matrix and Capability Gap Report.
- If missing skills are identified, offer to scaffold draft skill templates via [skill-creator](file:///home/phalou/github/louis-pvs/agent-skills/skills/skill-creator/SKILL.md):

  ```bash
  python3 skills/capability-gap-analyzer/scripts/main.py --scaffold-missing
  ```

---

## Usage

Unified CLI entrypoint:

```bash
# Analyze explicit domain
python3 skills/capability-gap-analyzer/scripts/main.py --domain frontend-web

# Auto-detect workspace domain mix for non-specific prompts
python3 skills/capability-gap-analyzer/scripts/main.py --auto-detect

# Output JSON structured inventory for agent consumption
python3 skills/capability-gap-analyzer/scripts/main.py --json
```

---

## References

- [overview.md](references/overview.md) — Two-tier evaluation architecture, taxonomy matrix definitions, and domain resolution heuristics.

---

## Completion Criteria

- [ ] Deterministic Tier 1 manifest scanner parses all `skills/*/SKILL.md` frontmatter without errors.
- [ ] Workspace domain auto-detector correctly identifies project manifest markers (`package.json`, `pyproject.toml`, `Dockerfile`, etc.).
- [ ] Taxonomy heatmap matrix accurately categorizes Strong, Partial, and Zero-Zone domain coverages.
- [ ] CLI exit codes pass cleanly with `--domain`, `--auto-detect`, and `--json`.
- [ ] All Python code passes `ruff check .` and `ruff format --check .` without warnings or errors.
