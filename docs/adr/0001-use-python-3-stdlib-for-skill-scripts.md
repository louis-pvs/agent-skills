# ADR 0001: Use Python 3 Standard Library for Skill Scripts

## Status

Accepted

## Context

Agent Skills require executable scripts for automation, scaffolding, validation, benchmarking, and AST parsing. These scripts must run reliably across heterogeneous agent host environments (Linux, macOS, WSL, Windows) without assuming external dependency installations.

## Decision Drivers

- **Zero-Dependency Portability**: Scripts must execute on any environment with standard Python 3.8+ installed.
- **Maintenance Overhead**: External dependencies (`requests`, `pydantic`, `pytest`, `click`) add version drift risks, lockfile complexity, and installation failures in restricted agent environments.
- **Performance**: Standard library modules import rapidly without PyPI package resolution overhead.

## Considered Options

1. **Option 1: Python 3 (Standard Library First)** _(Selected)_
2. **Option 2: Node.js (CommonJS / ES Modules)**
3. **Option 3: Bash / Shell Scripts**

---

## Decision Outcome

### Selected Option 1: Python 3 (Standard Library First)

### Rationale

- **Zero Prerequisite Installation**: Python 3.8+ is universally available across Linux, macOS, and AI agent environments. Using standard library modules (`argparse`, `pathlib`, `json`, `subprocess`, `urllib`, `dataclasses`, `asyncio`, `unittest`) ensures scripts run out-of-the-box without `pip install`.
- **Built-in Testing**: `unittest` (`python3 -m unittest discover`) provides an immediate, zero-dependency test runner.
- **Cross-Platform Paths**: `pathlib.Path` handles OS path differences seamlessly.

---

## Code Quality & Formatting Contract

All Python scripts within this repository MUST strictly adhere to:

- **Linter**: `ruff check .`
- **Formatter**: `ruff format .`
- **Verification Rule**: No Python script change is considered complete until both `ruff check .` and `ruff format --check .` pass cleanly without errors or warnings.

---

## Exceptions

If a task requires specialized third-party capabilities (e.g. `graphify`), fallback handling MUST be implemented so scripts gracefully notify the user or degrade functionality when third-party libraries are absent, rather than hard-crashing.
