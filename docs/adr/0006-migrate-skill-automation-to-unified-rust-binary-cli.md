# ADR 0006: Migrate Skill Automation to Unified Rust Binary CLI (`agent-skills`)

## Status

Accepted

## Context

Initially, skill automation scripts were implemented in Python 3 using standard library modules (ADR 0001). While Python stdlib scripts provided zero-dependency bootstrapping, as the project evolved into an ecosystem of 15+ skills, several pain points emerged:

- Fragmented CLI interfaces across multiple standalone script entry points.
- Runtime speed overhead when parsing ASTs, YAML frontmatter, and directory structures.
- Potential runtime exceptions not caught at build time.
- Security enforcement (path traversal sanitization ADR 0004, 4-tier configuration ADR 0005) needing duplicate Python helper logic.

A unified, compiled Rust binary (`agent-skills`) provides type safety, compile-time assurances, sub-millisecond execution, and single-binary portability across development environments.

## Decision Drivers

- **Unified CLI Interface**: Consolidate skill automation under a single binary `agent-skills <skill-name> <subcommand>`.
- **Sub-millisecond Performance**: Instant execution for skill validation, scaffolding, and workflow automation.
- **Compile-Time Safety**: Elimination of runtime type errors and unhandled exceptions via Rust static typing.
- **Centralized Security Core**: `agent_skills_core` crate providing canonical implementations of path sanitization (ADR 0004) and 4-tier configuration resolution (ADR 0005).

## Considered Options

1. **Option 1: Unified Rust Workspace CLI (`crates/core` + `crates/cli`)** _(Selected)_
2. **Option 2: Multiple Independent Rust Binaries per Skill**
3. **Option 3: Retain Pure Python stdlib Scripts**

---

## Decision Outcome

### Selected Option 1: Unified Rust Workspace CLI

We create a Rust workspace containing:

- **`crates/core` (`agent_skills_core`)**: Shared core logic, security utilities (`path_safety`), and configuration loader (`config_safety`).
- **`crates/cli` (`agent-skills`)**: Command-line binary interface dispatching subcommands to individual skill modules (starting with `skill-creator`).

### 4-Tier Skill Configuration Hierarchy (Rust Contract)

Configuration resolution follows the ADR 0005 precedence rules implemented in `agent_skills_core::config_safety::load_skill_config()`:

1. **Tier 1: CLI Flags** — Highest Precedence.
2. **Tier 2: Skill-Scoped Config** (`skills/<skill_name>/config.yaml`).
3. **Tier 3: Repository Global Config** (`skills.config.yaml` at repo root).
4. **Tier 4: Embedded Rust Defaults** — Fallback values embedded in binary.

### Path Sanitization & Security (Rust Contract)

All file system operations MUST pass through `agent_skills_core::path_safety::sanitize_path()` and use `agent_skills_core::path_safety::get_repo_root()` for root detection, eliminating path traversal risks (CWE-22).

---

## Migration Path

1. **Phase 1**: Implement `agent_skills_core` and migrate `skill-creator` to `agent-skills skill-creator`.
2. **Phase 2**: Progressively migrate remaining skills (`architecture-decision-records`, `code-janitor`, `tech-doc-writer`, etc.).
3. **Compatibility**: Python stdlib scripts remain as backward-compatible fallback aliases.
