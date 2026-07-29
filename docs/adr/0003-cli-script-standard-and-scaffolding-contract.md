# ADR 0003: CLI Script Standard & Scaffolding Contract

- **Status**: Accepted
- **Date**: 2026-07-29
- **Authors**: Louis & Agent Skill Team
- **Technical Story**: Establishing repository-wide CLI scripting contracts, standardized command-line flags (`--help`, `--dry-run`), output stream boundaries, exit code conventions, and executable scaffolding to prevent script drift across Agent Skills.

---

## Context & Problem Statement

As the number of Agent Skills in this repository grows, executable scripts (`skills/*/scripts/` and repository maintenance tools) risk drifting in behavior, flag conventions, error handling, and output formats.

Specifically:

1. **Inconsistent CLI Flags**: Some scripts used positional arguments, while others supported `-h`/`--help` or custom option names.
2. **Unsafe Mutating Side-Effects**: Scripts modifying the filesystem or external systems lacked a uniform `--dry-run` flag for previewing changes.
3. **Stream Pollution**: Error messages and diagnostic logs were frequently mixed into standard output (`stdout`), breaking downstream JSON parsers and piping workflows.
4. **Exit Code Ambiguity**: Differing exit code conventions made automated test runners and agent CLI orchestrators unreliable when detecting runtime vs argument failures.

We need a standardized script architecture contract and executable template to eliminate drift and ensure uniform developer/agent experience.

---

## Decision Drivers

- **Zero External Dependencies**: Must strictly adhere to ADR 0001 (Python 3 stdlib only).
- **Executable Template over Prose Rules**: Ship a canonical template script (`scripts/_template.py`) that serves as a copy-paste baseline.
- **Out-of-the-Box Help**: `argparse` stdlib must be used so `--help` / `-h` is supported automatically.
- **Safe Execution**: Mutating operations must provide a `--dry-run` flag to preview actions without side-effects.
- **Stream Discipline**: `stdout` for clean data outputs / `--json`, `stderr` for logs and errors.
- **Deterministic Exit Codes**: `0` for success, `1` for execution failure, `2` for argument parsing errors.

---

## Considered Options

1. **Option 1: Python 3 stdlib `argparse` Contract + `scripts/_template.py` Scaffold** _(Selected)_
2. **Option 2: Prose Guidelines Only (No template script)**
3. **Option 3: External CLI Framework (Click/Typer)** _(Rejected due to ADR 0001 zero-external-dependency constraint)_

---

## Decision Outcome

**Selected Option 1: Python 3 stdlib `argparse` Contract + `scripts/_template.py` Scaffold**

### Rationale

- **`scripts/_template.py` Baseline**: Provides an executable baseline supporting `--help`, `--dry-run`, `--json`, `--verbose`, stream separation, and typed `main()` functions.
- **Argparse Stdlib**: Guarantees zero external prerequisites while providing automatic `-h`/`--help` flag formatting.
- **Mandatory `--dry-run`**: Any script with file, network, or resource mutations MUST implement `--dry-run` to enable safe dry runs by AI agents and developers.
- **Standardized Exit Codes**:
  - `0`: Clean execution / success.
  - `1`: Runtime execution failure.
  - `2`: Invalid CLI arguments or usage error.
- **Stream Separation**:
  - `stdout`: Structured results / primary user output / `--json` payload.
  - `stderr`: Diagnostic logging and error tracebacks.

---

## Positive Consequences

- Scripts across all skills follow identical CLI flags, logging habits, and stream rules.
- AI agents and developers can execute any script with `--dry-run` safely before running mutating operations.
- Unit testing (`unittest`) can import `main()` directly without subprocess overhead.
- CI and test runners receive deterministic exit codes.

---

## Negative Consequences & Mitigation

- **Slight Boilerplate**: Small single-purpose scripts require `argparse` blocks and a `main()` function.
- **Mitigation**: Developers and agents copy `scripts/_template.py` directly, reducing setup time to under 30 seconds.
