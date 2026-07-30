# ADR 0005: Unified Skill Configuration & Merge Hierarchy

## Status

Accepted

## Context

As the Agent Skill ecosystem expands, configuration files have proliferated in
ad-hoc formats across the repository root (`skills.config.yaml`), skill
subdirectories (`council.config.yaml`), and agent platform standards
(`skills.json`). Without a standardized configuration hierarchy and merge strategy,
skills risk configuration drift, duplicated parsers, and resolution ambiguity.

## Decision Drivers

- **Predictable Resolution Hierarchy**: Clear precedence rules when CLI flags,
  skill configs, repo configs, and stdlib defaults collide.
- **Zero-Dependency Standardization**: Canonical, stdlib-only parser and loader
  handling configuration loading without PyPI dependencies (ADR 0001).
- **Naming Uniformity**: Standardized config file naming (`config.yaml`) within
  skill subdirectories.
- **Backward Compatibility**: Graceful fallback support for legacy config
  filenames and missing config files.

## Considered Options

1. **Option 1: 4-Tier Hierarchy with Canonical Stdlib Loader (`_config_safety.py`)** _(Selected)_
2. **Option 2: Ad-hoc Skill-Specific YAML Parsers**
3. **Option 3: Pure JSON Configuration Files**

---

## Decision Outcome

### Selected Option 1: 4-Tier Hierarchy with Canonical Stdlib Loader

### Configuration Precedence Order

When resolving runtime configuration settings, skills MUST evaluate configuration
in strict priority order:

1. **Tier 1: CLI Flags** (e.g. `--config`, `--domain`) — Highest Precedence.
2. **Tier 2: Skill-Scoped Config** (`skills/<skill_name>/config.yaml`).
3. **Tier 3: Repository Global Config** (`skills.config.yaml` at repo root).
4. **Tier 4: Embedded Stdlib Defaults** — Fallback when no configs exist.

### Standard File Naming Conventions

- **Repository-Level Config**: `skills.config.yaml` at repository root.
- **Skill-Level Config**: `skills/<skill_name>/config.yaml` inside skill directory.

---

## Canonical Implementation Contract

All skill scripts requiring configuration loading SHOULD import from
`scripts._config_safety`:

- `load_skill_config(skill_name, base_dir)`: Loads merged settings across Tier
  2, Tier 3, and Tier 4.
- `parse_simple_yaml(content)`: Canonical zero-dependency stdlib YAML parser.
