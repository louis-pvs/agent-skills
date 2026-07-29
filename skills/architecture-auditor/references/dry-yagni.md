# DRY vs YAGNI Reference

Comprehensive heuristics for balancing **Don't Repeat Yourself (DRY)** with **You Aren't Gonna Need It (YAGNI)**.

---

## 1. Don't Repeat Yourself (DRY)

> **"Every piece of knowledge must have a single, unambiguous, authoritative representation within a system."**

### Key Distinction: Knowledge Duplication vs Code Repetition

- **True DRY Violation (Knowledge Duplication)**: The exact same business logic, calculation formula, validation rule, or domain constraint copied across multiple places. Changing the rule requires remembering to edit 5 separate files.
- **Accidental Duplication (Coincidental Similarity)**: Two code blocks look syntactically similar today, but represent different domain concepts (e.g. formatting a customer billing address vs formatting a shipping warehouse address). Unifying them into a single function creates coupling across unrelated domains.

### Detection Heuristics & Anti-Patterns

- **Magic Constant Repetition**: Hardcoded URLs, timeout numbers, SQL query strings, or regex patterns repeated across modules.
- **Copy-Paste Business Rules**: Multi-step algorithm logic duplicated across endpoints.
- **Rule of Three**: Code or logic appearing verbatim 3 or more times is a strong candidate for extraction.

---

## 2. You Aren't Gonna Need It (YAGNI)

> **"Always implement things when you actually need them, never when you just foresee that you might need them."**

### Detection Heuristics & Anti-Patterns

- **Speculative Generality**:
  - Abstract classes with only a single concrete subclass.
  - Complex configuration parameters, flags, or hooks that are never passed or customized in production.
  - "Just in case" database columns, unused API parameters, or dead fallback branches.
- **Over-Architected Extensibility**: Building a plugin architecture or micro-kernel framework for a simple CRUD utility that will only ever support one implementation.
- **Premature Abstraction**: Wrapping simple standard library calls in custom interface wrappers before a second provider or implementation actually exists.

---

## 3. Balancing DRY and YAGNI

```
                  ┌────────────────────────────────────────┐
                  │ Does this logic repeat 3+ times OR    │
                  │ represent a single authoritative rule? │
                  └───────────────────┬────────────────────┘
                                      │
                         ┌────────────┴────────────┐
                         ▼                         ▼
                       [YES]                     [NO]
                         │                         │
            ┌────────────┴────────────┐     ┌──────┴────────────────────┐
            │ Do both sites share the │     │ Keep duplicated/separate; │
            │ exact same domain boundary?│   │ Do NOT extract premature │
            └────────────┬────────────┘     │ abstraction (YAGNI).      │
                         │                  └───────────────────────────┘
            ┌────────────┴────────────┐
            ▼                         ▼
          [YES]                     [NO]
            │                         │
   ┌────────┴─────────┐     ┌─────────┴────────┐
   │ Extract Shared   │     │ Keep Separate   │
   │ Function/Module  │     │ (Accidental DRY)│
   └──────────────────┘     └──────────────────┘
```

### Heuristic Rule of Thumb

- **0–2 Repetitions**: Duplication is cheaper than the wrong abstraction. Apply YAGNI.
- **3+ Repetitions**: Extract shared function or module. Apply DRY.
