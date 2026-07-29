# Architectural Principle Tensions & Trade-Off Matrix

Framework for resolving trade-offs when architectural principles conflict with one another.

---

## Conflict Matrix & Resolution Strategies

### 1. DRY vs YAGNI (Rule of Three vs Premature Abstraction)

- **Tension**: DRY urges extracting common code immediately. YAGNI warns against premature abstractions for hypothetical future reuse.
- **Resolution**:
  - Apply the **Rule of Three**: Allow up to 2 instances of code similarity without abstracting.
  - On the 3rd repetition, extract a shared helper if the logic shares the same domain boundary.
  - If code looks identical but belongs to different domain contexts, keep them separate (Accidental Duplication).

### 2. SOLID ISP vs KISS (Interface Proliferation vs Simplicity)

- **Tension**: ISP encourages creating tiny 1-method interfaces. KISS warns against explosive growth of tiny interface files and cognitive bloat.
- **Resolution**:
  - Prefer concrete classes by default when there is only one consumer and implementation.
  - Introduce Protocols / Interfaces only when multiple implementations exist OR when boundaries cross major system layers (e.g. database vs domain).

### 3. SOLID OCP vs YAGNI (Pluggable Strategy vs Simple Conditional)

- **Tension**: OCP recommends abstract strategies and factories to allow future extensions without modifying existing code. YAGNI warns against building plugin frameworks for code that rarely changes.
- **Resolution**:
  - Use simple `if/elif` or `match/case` blocks when types are fixed and predictable (e.g., 3 known export formats: CSV, JSON, PDF).
  - Refactor to polymorphic OCP classes only when new types are added frequently or provided by third-party plugins.

### 4. SOLID SRP vs CUPID Unix-Like (Micro-Classes vs Cohesive Modules)

- **Tension**: Over-zealous SRP can fragment a single workflow into 15 tiny single-method classes, hurting readability.
- **Resolution**:
  - Group tightly related functions into cohesive, single-module domain pipelines.
  - Ensure high cohesion within a file rather than splitting every 10 lines into separate class files.
