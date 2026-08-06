# Technique Selection Heuristics

## Decision Tree

```mermaid
flowchart TD
    Q["User asks about codebase"] --> A["Architecture / Component relationships"]
    Q --> B["Co-changing files / Impact analysis"]
    Q --> C["Who calls X / Where defined / References"]
    Q --> D["Class inheritance / Structural pattern matching"]
    Q --> E["Complex / Multi-dimensional question"]

    A --> A_OUT["graphify<br/>(if graphify-out/ exists)"]
    B --> B_OUT["agent-skills context-gatherer git-coupling"]
    C --> C_OUT["agent-skills context-gatherer symbol-nav"]
    D --> D_OUT["agent-skills context-gatherer ast-search"]
    E --> E_OUT["Combine multiple techniques & synthesize"]
```

## Technique Comparison

| Technique | Strength | Weakness | Complexity |
| :--- | :--- | :--- | :--- |
| Git Coupling | Reveals hidden logical dependencies | Only works with sufficient git history | Low (native Rust) |
| Symbol Nav | Fast, works on any text-based code | Regex/word-boundary based matching | Low (native Rust) |
| AST Search | Structurally accurate pattern matching | Pattern-based regex matcher | Medium (native Rust) |
| Graphify | Persistent graph, community detection | Static snapshot, doesn't capture recency | Already built |

## Combining Techniques

For maximum context, combine in this order:

1. **graphify** — architectural overview (if available)
2. **git-coupling** — find co-changing files for the target
3. **symbol-nav** — find callers/definitions in the coupled files
4. **ast-search** — verify structural patterns in the results
