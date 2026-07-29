# CUPID Design Principles Reference

Quality properties for joyful, readable, and maintainable software (Dan North's CUPID framework).

---

## The 5 CUPID Properties

### 1. **C**omposable

- **Core Ideal**: Plays well with others.
- **Heuristics**:
  - **Small Surface Area**: Minimal public API surface; hides internal state and helper functions.
  - **Intention-Revealing Signatures**: Functions accept explicit, strongly typed inputs and return predictable, self-contained values.
  - **Low Coupling**: Dependencies are injected or easily substituted.

### 2. **U**nix-Like

- **Core Ideal**: Does one thing well and composes via standard pipelines.
- **Heuristics**:
  - **Single Purpose**: Focused purpose that can be summarized in one short sentence.
  - **Standard Data Formats**: Accepts and emits standard datatypes (dicts, streams, lists, primitives) rather than proprietary internal wrapper classes.
  - **Text-Friendly Input/Output**: Easily testable via text feeds, CLI streams, or standard JSON schemas.

### 3. **P**redictable

- **Core Ideal**: Does what it looks like it does; no surprises under the hood.
- **Heuristics**:
  - **No Hidden Side Effects**: Calling a getter or query does not mutate global state, write to DB, or launch background side-effects unexpectedly.
  - **Deterministic Output**: Given the same inputs, produces identical outputs.
  - **Robust Error Handling**: Fails explicitly with clear, actionable exceptions rather than swallowing errors or returning silent `None` / `0` fallbacks.

### 4. **I**diomatic

- **Core Ideal**: Feels natural and familiar to developers working in the language ecosystem.
- **Heuristics**:
  - **Language Standards**: Follows PEP 8 for Python, standard idiomatic Go for Go, standard TypeScript design patterns for TS/JS.
  - **Ecosystem Libraries**: Leverages built-in standard library utilities or standard community tools rather than reinventing custom wheels.

### 5. **D**omain-Based

- **Core Ideal**: Code structure and naming mirror the business domain language (Ubiquitous Language).
- **Heuristics**:
  - **Domain Naming**: Variable and class names reflect actual domain terminology (`Invoice`, `Subscription`, `FulfillOrder`) rather than generic technical jargon (`DataHandlerProcessor`, `ManagerUtil`).
  - **Cohesive Package Layout**: Modules organized by domain capability rather than technical layer directories when appropriate.
