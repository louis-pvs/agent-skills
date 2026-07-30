---
name: context-gatherer
description: Gather deep codebase context before making changes. Use when the user says 'gather context', 'what changes with this file', 'who calls this', 'find pattern', or wants to understand impact before a refactor.
disable-model-invocation: true
enhances:
  - graphify
version: 1.0.0
---

# Context Gatherer

Gather deep codebase context before making code changes. Orchestrates multiple context-extraction techniques to surface hidden dependencies, call sites, and structural patterns.

## When to Use

- Before a refactor: understand what files are coupled to the ones you're changing
- Impact analysis: find all callers/references of a symbol before modifying it
- Pattern discovery: find structural code patterns (all implementations of an interface, all handlers matching a signature)
- After a merge/rebase: refresh your understanding of recently-changed areas
- Entering unfamiliar code: build a mental map before making changes

## T-Shape Domain Scope & Boundary

- **Descriptive Domain (`context-gatherer`)**: Backward-looking & exploratory context mapping. Focuses on surfacing existing codebase state—git temporal coupling (`git_coupling.py`), symbol definitions/usages (`symbol_nav.py`), structural pattern matching (`ast_search.py`), and knowledge graph queries (`graphify`).
- **Predictive Domain (`what-if-analysis`)**: Forward-looking & simulation-driven. For pre-commit simulations, blast radius risk calculation, scenario tradeoff matrices, counterfactual RED test synthesis, and shift-left failure pre-emption, use `what-if-analysis`.

## Sub-Skill Directory & Routing Map

| Intent / Task | Script | Description |
| :--- | :--- | :--- |
| Files that change together / impact analysis | `scripts/git_coupling.py` | Parse git history to find temporally coupled files |
| Who calls this / find all references | `scripts/symbol_nav.py` | Find definitions and usages of symbols via regex search |
| Structural code pattern matching | `scripts/ast_search.py` | AST-based search for code structures (detect-and-degrade) |
| Architecture / relationship questions | → **graphify** | Use existing graphify skill if `graphify-out/` exists |

## Guidance for Agent

1. **Check graphify first**: If `graphify-out/graph.json` exists and the question is architectural (component relationships, dependency paths), use `/graphify query` instead.
2. **Select technique by intent**:
   - "What else changes when I modify X?" → `git_coupling.py`
   - "Who calls function Y?" / "Where is Z defined?" → `symbol_nav.py`
   - "Find all classes that implement X" / "Match this code pattern" → `ast_search.py`
3. **Combine techniques** when the question spans multiple dimensions (e.g., find all callers of X AND what files co-change with them).

## Usage

### Git Temporal Coupling

Find files that frequently change together with a target file:

```bash
python3 skills/context-gatherer/scripts/git_coupling.py --file path/to/file.py
python3 skills/context-gatherer/scripts/git_coupling.py --file path/to/file.py --min-commits 3 --limit 20
```

### Symbol Navigation

Find definitions and references of a symbol:

```bash
python3 skills/context-gatherer/scripts/symbol_nav.py --symbol "MyClass" --path src/
python3 skills/context-gatherer/scripts/symbol_nav.py --symbol "my_function" --path src/ --type definition
python3 skills/context-gatherer/scripts/symbol_nav.py --symbol "my_function" --path src/ --type reference
```

### AST Search

Find structural code patterns:

```bash
python3 skills/context-gatherer/scripts/ast_search.py --pattern "class * (BaseHandler)" --path src/
python3 skills/context-gatherer/scripts/ast_search.py --pattern "def test_*" --path tests/
```

## Completion Criteria

- [ ] Selected the appropriate technique based on user intent.
- [ ] Script executed cleanly with exit code 0.
- [ ] Results presented to user with file paths and relevant context.
- [ ] If multiple techniques combined, results synthesized into unified view.

## References

- [overview.md](references/overview.md) — Design rationale, graphify integration, and architecture decisions.
- [technique-heuristics.md](references/technique-heuristics.md) — When to prefer which technique.
