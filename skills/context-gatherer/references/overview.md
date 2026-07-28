# Context Gatherer — Overview

## Design Rationale

Context Gatherer provides **task-scoped, on-demand** codebase context extraction. Unlike graphify (which builds a persistent, static knowledge graph), this skill runs live queries against the current state of git history, source code, and AST structures.

## Complementary to Graphify

| Question Type | Best Tool | Why |
| :--- | :--- | :--- |
| Component architecture, dependency paths | graphify | Static graph with community detection already built |
| What else changes when I modify this file? | git-coupling | Live git history, captures recent commits |
| Who calls this function right now? | symbol-nav | Live code search, reflects current state |
| Find all handler implementations | ast-search | Structural matching on current code |

### Integration Rule

If `graphify-out/graph.json` exists, the agent should check it first for architectural questions before dispatching context-gatherer sub-skills. Context-gatherer handles what a static graph cannot: recency, exact current usages, and temporal patterns.

## External Dependencies Strategy

**Detect-and-degrade, not hard-depend.**

- `git` — Required (reasonable assumption for any codebase)
- `rg` (ripgrep) — Optional; used by `symbol_nav.py` when available, falls back to `grep`
- `ast-grep` / `tree-sitter` — Optional; used by `ast_search.py` when available, falls back to Python `ast` module for `.py` files and regex heuristics for others
- No `pip install` required — Python 3 stdlib first

## When to Re-Gather Context

- After a large merge or rebase (coupling relationships changed)
- When entering an unfamiliar area of the codebase mid-session
- After external PRs landed that touched your working area
- When the agent's initial context window gets stale during a long refactor
