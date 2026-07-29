# KISS Principle Reference

Heuristics for **Keep It Simple, Stupid (KISS)**.

---

## 1. Core Principles of Simplicity

> **"Simplicity is prerequisite for reliability." — Edsger W. Dijkstra**

Code simplicity is not merely code brevity; it is the minimization of cognitive load required to understand, trace, and maintain a system.

---

## 2. Detection Heuristics & Anti-Patterns

### 1. High Cyclomatic Complexity

- **Signals**: Functions with nested loops, 4+ levels of indented `if/else` statements, or multiple complex logical boolean expressions (`if a and (b or not c) and d:`).
- **Threshold**: Functions with cyclomatic complexity > 10 should be simplified.

### 2. Excessive Indirection & Layers

- **Signals**: Call stacks where a simple data retrieval request passes through 6 different pass-through classes (`Controller` -> `Service` -> `Manager` -> `Handler` -> `Provider` -> `Repository` -> `DAO`) where each class simply delegates to the next without performing real business work.
- **Rule**: If a layer adds no transformation, validation, or abstraction value, delete the indirection layer.

### 3. Clever Code over Readable Code

- **Signals**: Over-use of complex inline lambda chains, dense ternary expressions, or obscure metaprogramming hacks (`__getattr__` trickery) when a clean 3-line loop is self-explanatory.
- **Rule**: Readability > Cleverness.

### 4. Over-Configurability

- **Signals**: Functions accepting a dictionary of 15 optional configuration overrides when only 2 parameters are ever changed in practice.
