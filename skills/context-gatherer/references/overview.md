# Context Gatherer Architectural Overview

The **Context Gatherer** maps codebase topology, AST symbol references, temporal git coupling, and structural context across repositories.

---

## Analysis Pipeline

```mermaid
flowchart TD
    A[Codebase Exploration Request] --> B[AST Symbol Search]
    B --> C[Git Temporal Coupling Analysis]
    C --> D[Knowledge Graph Querying]
    D --> E[Synthesize Context Map]
```

---

## Core Capabilities

- **Symbol Navigation (`agent-skills context-gatherer symbol-nav`)**: Traverses AST definitions, import targets, and call sites.
- **AST Pattern Search (`agent-skills context-gatherer ast-search`)**: Locates class definitions, decorated handlers, and function signatures.
- **Git Coupling Analysis (`agent-skills context-gatherer git-coupling`)**: Identifies files that historically commit together.

---

## Key Invariants

> [!NOTE]
> `context-gatherer` is purely exploratory and read-only; it makes zero mutating code edits.
>
> [!IMPORTANT]
> Use `context-gatherer` to map existing codebase topology before running prospective refactoring simulations (`what-if-analysis`).
