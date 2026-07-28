# ADR 0001: Use Python 3 Standard Library First for Skill Scripts

- **Status**: Accepted
- **Date**: 2026-07-28
- **Authors**: Louis & Agent Skill Team
- **Technical Story**: Establishing repository-wide scripting and testability standards across all Agent Skills.

---

## Context & Problem Statement

Agent Skills often require executable automation scripts (for scaffolding, parsing multi-LLM outputs, running background jobs, or data processing).

Previously, skills used a mix of Bash (`.sh`) and Node.js (`.cjs`/`.js`). However, during real-world execution across diverse agent platforms (Antigravity, Gemini CLI, Claude Code, Cursor, Codex):
1. **Prerequisite & Dependency Failures**: Node.js scripts frequently failed due to missing npm dependencies (e.g., `Missing runtime dependency: yaml`), requiring manual `npm/pnpm install` steps.
2. **Cross-Platform Incompatibility**: Shell scripts (`.sh`) fail natively on Windows environments without WSL or Git Bash.
3. **Subprocess Sandbox Restrictions**: Spawning nested shell subprocesses within AI agent CLI tool environments often triggered binary path and permission errors.
4. **Lack of Standard Unit Testing**: Bash scripts and ad-hoc Node scripts lacked standard unit test runners out-of-the-box.

We need a unified, portable, zero-prerequisite scripting language standard for all skills in this repository.

---

## Decision Drivers

- **Zero External Prerequisites**: Scripts must run out-of-the-box on Linux, macOS, Windows, and containerized agent environments without requiring `pip install`, `npm install`, or external package managers.
- **Built-in Testability**: Scripts must support zero-dependency unit testing.
- **Cross-Platform OS Compatibility**: Paths, file I/O, and process execution must work cross-platform (`/` vs `\`).
- **LLM Readability & Execution**: High generation accuracy and execution reliability by AI coding agents.

---

## Considered Options

1. **Option 1: Python 3 (Standard Library First)** *(Selected)*
2. **Option 2: Node.js (CommonJS / ES Modules)**
3. **Option 3: Bash / Shell Scripts**

---

## Decision Outcome

**Selected Option 1: Python 3 (Standard Library First)**

### Rationale:
- **Zero Prerequisite Installation**: Python 3.8+ is universally available across Linux, macOS, and AI agent environments. Using standard library modules (`argparse`, `pathlib`, `json`, `subprocess`, `urllib`, `dataclasses`, `asyncio`, `unittest`) ensures scripts run out-of-the-box without `pip install`.
- **Built-in Testing**: `unittest` (`python3 -m unittest discover`) provides an immediate, zero-dependency test runner.
- **Cross-Platform Paths**: `pathlib.Path` handles OS path differences seamlessly.
- **Clean LLM Maintenance**: A single `.py` file encapsulates logic cleanly, avoiding fragmented `.sh` + `.js` script wrappers.

### Restrictions & Exceptions:
- **Shell Script Wrappers**: Permitted **only** as thin entrypoints ($< 20$ lines) with strict bash flags (`set -euo pipefail`).

---

## Positive Consequences

- All skill automation scripts run out-of-the-box without dependency installation errors.
- Unit testing can be enforced automatically via `python3 -m unittest`.
- Skills become completely portable across Antigravity, Gemini CLI, Claude Code, Cursor, and Codex on any OS.

---

## Negative Consequences & Mitigation

- **Constraint**: Cannot freely pull heavy third-party PyPI packages (`requests`, `pandas`, `yaml`).
- **Mitigation**: Standard library `urllib.request`, `json`, and built-in Python parsing are sufficient for skill automation needs. If third-party packages are strictly necessary, they must be optional features with explicit fallback handlers.
