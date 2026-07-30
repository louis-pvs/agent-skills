# Skill Creator Reference Overview

Comprehensive architectural overview and reference documentation for `skill-creator`.

## Overview

`skill-creator` guides and automates the creation of high-quality, standardized, predictable Agent Skills adhering to the `agentskills.io` open specification.

## Core Capabilities

- **Scaffolding (`scaffold_skill.py`)**: Generates directory structure, YAML frontmatter, `SKILL.md`, `README.md`, and standard test suites.
- **Validation (`scaffold_skill.py --validate`)**: Audits skills against frontmatter syntax, context load limits (<= 500 lines), checkable completion criteria, path privacy rules, and stdlib script compliance.

## Reference Guides

- [agentskills-spec.md](agentskills-spec.md) — Open specification standard.
- [multi-llm-patterns.md](multi-llm-patterns.md) — Multi-agent delegation patterns.
- [scripting-standards.md](scripting-standards.md) — Python stdlib-first scripting standards.
