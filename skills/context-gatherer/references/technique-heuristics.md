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
    B --> B_OUT["git_coupling.py"]
    C --> C_OUT["symbol_nav.py"]
    D --> D_OUT["ast_search.py"]
    E --> E_OUT["Combine multiple techniques & synthesize"]
```

## Technique Comparison

| Technique    | Strength                               | Weakness                                      | Complexity          |
| :----------- | :------------------------------------- | :-------------------------------------------- | :------------------ |
| Git Coupling | Reveals hidden logical dependencies    | Only works with sufficient git history        | Low (stdlib)        |
| Symbol Nav   | Fast, works on any text-based code     | Not AST-accurate, may have false positives    | Low (stdlib + grep) |
| AST Search   | Structurally accurate pattern matching | Language-specific, needs parsers for polyglot | Medium-High         |
| Graphify     | Persistent graph, community detection  | Static snapshot, doesn't capture recency      | Already built       |

## Combining Techniques

For maximum context, combine in this order:

1. **graphify** — architectural overview (if available)
2. **git_coupling** — find co-changing files for the target
3. **symbol_nav** — find callers/definitions in the coupled files
4. **ast_search** — verify structural patterns in the results
