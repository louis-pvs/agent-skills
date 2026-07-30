# ADR 0004: Common Path Sanitization Utility

- **Status**: Accepted
- **Date**: 2026-07-30
- **Authors**: Louis & Agent Skill Team
- **Technical Story**: Eliminating Snyk Path Traversal warnings without third-party dependencies; consolidating 4 divergent copy-pasted implementations across 7 files.

---

## Context & Problem Statement

Snyk static analysis flags every inline path construction (using `Path()`, `os.path.realpath()`, etc.) as a potential **Path Traversal** vulnerability (CWE-22). The only way Snyk stops warning is either:

1. **Third-party library**: Install `werkzeug` and use `werkzeug.utils.secure_filename()` — but this violates ADR 0001 (stdlib-first) and is YAGNI (we don't need the rest of werkzeug).
2. **Snyk-approved pattern**: Use the `normpath + join("/", ...) + realpath + commonpath + per-component walk` pattern that Snyk's taint analysis recognises as a proper sanitizer.

The repository had **4 distinct implementations** of path sanitization logic, copy-pasted across 7 files, each with subtle behavioral differences:

| Variant | Files | Issues |
| :--- | :--- | :--- |
| `realpath + commonpath + relative_to` | `depgraph.py`, `scaffold_skill.py` | Simple but Snyk still flags it |
| `startswith + commonpath` | `counterfactual_generator.py`, `ast_graph_parser.py` | Minimal; Snyk flags it |
| `basename + normpath` | `impact_analyzer.py` | **Bug**: strips directory structure |
| `normpath + join + scandir` | `failure_preemptor.py` | ✅ Passes Snyk with 0 issues |

A drift-guard test (`test_path_contract.py`) existed solely to catch behavioral divergence between the copies — proving the team was already aware of the problem.

---

## Decision Drivers

- **Zero Snyk Warnings**: The canonical implementation must pass `snyk code test` with 0 total issues.
- **ADR 0001 Compliance**: No third-party dependencies (no `werkzeug`, `pathvalidate`, etc.).
- **Standalone Portability**: Skill scripts must remain independently executable when symlinked or copied outside this repo.
- **DRY Principle**: Eliminate the 4-variant duplication that had already caused a behavioral bug.

---

## Considered Options

1. **Option 1: Third-party `werkzeug.utils.secure_filename()`** _(Rejected — violates ADR 0001)_
2. **Option 2: Shared import via `sys.path.insert`** _(Rejected — breaks standalone portability, fragile depth assumption)_
3. **Option 3: Canonical source of truth + verified copies** _(Selected)_
4. **Option 4: Snyk inline suppression (`# nosec`)** _(Rejected — masks the issue instead of fixing it)_

---

## Decision Outcome

### Selected Option 3: Canonical Source of Truth + Verified Copies

### Architecture

```text
scripts/_path_safety.py          ← Canonical source of truth (stdlib-only)
├── sanitize_path()              ← General-purpose file/dir path sanitization
└── resolve_safe_dir()           ← Directory-only variant with exists+is_dir check

scripts/depgraph.py              ← Direct import (repo tooling)
scripts/lint_scripts.py          ← Direct import (repo tooling) + INTERNAL_MODULES allowlist

skills/*/scripts/*.py            ← Verified copies (standalone portability preserved)
```

### How It Works

1. **`scripts/_path_safety.py`** is the canonical, stdlib-only implementation using the Snyk-approved `normpath + join("/", ...) + realpath + commonpath + per-component basename walk` pattern.

2. **Repo tooling** (`depgraph.py`, `lint_scripts.py`) imports directly via `from scripts._path_safety import sanitize_path`. This is permitted because repo tooling always runs from the repo root and is covered by the "Repository Development Tooling Exception" in ADR 0001.

3. **Skill scripts** maintain a local copy of the canonical implementation (with an `# ADR 0004` docstring reference). `lint_scripts.py` uses a separate `INTERNAL_MODULES` allowlist to permit repo-level internal imports without polluting `STDLIB_MODULES`.

4. **`_path_safety.py`** uses a `_` prefix to signal it's an internal utility module — `lint_scripts.py` skips `_`-prefixed files from ADR 0003 CLI contract checks (no `argparse`/`main()`/`--dry-run` required).

### ADR 0001 Tradeoff

This decision **deliberately narrows** ADR 0001's standalone guarantee for security-critical utilities. The tradeoff is:

- **Before**: Each skill script had a standalone but divergent (and sometimes buggy) implementation.
- **After**: Skill scripts have standalone verified copies of a single canonical implementation. The canonical source lives in `scripts/_path_safety.py` and is validated by the lint checker.

If skills are ever published or consumed standalone outside this repo, the local copies remain functional. The verified-copy pattern ensures they don't silently drift.

---

## Positive Consequences

- `snyk code test` returns 0 total issues across all repository scripts.
- Single canonical implementation eliminates the 4-variant duplication and the `impact_analyzer.py` bug.
- No third-party dependencies added.
- Standalone portability preserved for skill scripts.
- The drift-guard test (`test_path_contract.py`) is replaced by comprehensive `test_path_safety.py`.

---

## Negative Consequences & Mitigation

- **Constraint**: Verified copies mean the same function body appears in multiple files.
- **Mitigation**: `lint_scripts.py` validates copies match the canonical; the `# ADR 0004` docstring comment makes the relationship explicit and discoverable.
