# Code Smells Catalog

Categorized catalog of common code smells with detection heuristics and refactoring recipes.

## Dispensables

Smells that represent unnecessary code that should be removed.

### Unused Imports

- **Detection**: AST import nodes not referenced in function bodies, assignments, or type annotations.
- **Severity**: 🚨 CRITICAL (unused imports increase load time, confuse readers, and may mask dependency issues).
- **Fix**: Remove the import statement.

### Dead Code

- **Detection**: Statements after unconditional `return`, `raise`, `break`, or `continue`.
- **Severity**: 🚨 CRITICAL (misleads readers into thinking the code executes).
- **Fix**: Delete the unreachable statements.

### Commented-Out Code

- **Detection**: Multi-line comments containing syntactically valid code patterns.
- **Severity**: ⚠️ WARNING (use version control instead of comment preservation).
- **Fix**: Delete the commented block. Git history preserves the original.

## Bloaters

Smells indicating code that has grown too large to work with effectively.

### Oversized Functions (Long Method)

- **Detection**: Function body exceeds configured line threshold (default: 30 lines).
- **Severity**: ⚠️ WARNING.
- **Fix**: Extract Method — identify cohesive blocks and extract into named helper functions.

### Too Many Parameters (Long Parameter List)

- **Detection**: Function signature exceeds configured parameter threshold (default: 5).
- **Severity**: ⚠️ WARNING.
- **Fix**: Introduce Parameter Object — group related parameters into a dataclass or TypedDict.

### Large Class

- **Detection**: Class body exceeds 300 lines or has > 20 methods.
- **Severity**: ⚠️ WARNING.
- **Fix**: Extract Class — identify independent responsibilities and split.

## Complexity Abusers

Smells indicating unnecessarily complex control flow.

### Deep Nesting

- **Detection**: Indentation depth exceeds configured threshold (default: 4 levels).
- **Severity**: ⚠️ WARNING.
- **Fix**: Apply Guard Clause pattern — invert conditions and return early. Extract nested blocks into helper functions.

### High Cyclomatic Complexity

- **Detection**: Branch count (`if`, `elif`, `for`, `while`, `and`, `or`, `except`) exceeds threshold (default: 10 per function).
- **Severity**: ⚠️ WARNING.
- **Fix**: Replace conditionals with polymorphism, strategy pattern, or dispatch tables.

## Documentation Gaps

Smells indicating missing documentation or type safety.

### Missing Docstrings

- **Detection**: Functions or classes without a docstring (`__doc__` is None).
- **Severity**: 💡 ADVISORY.
- **Fix**: Add a concise docstring describing purpose, parameters, and return value.

### Missing Type Annotations

- **Detection**: Function definitions without return type annotation.
- **Severity**: 💡 ADVISORY.
- **Fix**: Add return type hint (e.g., `-> None`, `-> str`, `-> List[int]`).

## Maintenance Debt

Smells indicating deferred work that accumulates over time.

### Stale TODO/FIXME Markers

- **Detection**: Comments containing `TODO`, `FIXME`, `HACK`, `XXX`, or `NOQA`.
- **Severity**: 💡 ADVISORY.
- **Fix**: Resolve the TODO (implement, remove, or convert to a tracked issue).

## Modernization & Expressive Idioms

Smells indicating legacy syntax patterns, verbose guard checks, or procedural boilerplate that can be refactored into modern, declarative language idioms.

### Verbose Guard Checks

- **Detection**: Explicit type or existence guard checks prior to function invocation, e.g., `if (typeof fn === 'function')` or `if (obj !== null && obj !== undefined)`.
- **Severity**: 💡 ADVISORY.
- **Fix**: Use modern language operators such as optional chaining (`fn?.(arg)` or `obj?.prop`) or pattern matching / guard clauses.

### Imperative Boilerplate to Declarative Expressions

- **Detection**: Multi-line imperative setup (e.g. manual iteration loops for filtering/transforming state) that can be expressed cleanly as functional transformations.
- **Severity**: 💡 ADVISORY.
- **Fix**: Refactor to declarative collection transformations (`filter`, `map`, direct conditional returns).

### Redundant Ternary & Nullish Checks

- **Detection**: Redundant boolean ternaries (e.g., `cond ? true : false`) or manual null checks refactorable via nullish coalescing.
- **Severity**: 💡 ADVISORY.
- **Fix**: Use direct boolean expressions or nullish coalescing operators (`??`).

### Nullish Coalescing & Logical Assignment

- **Detection**: Explicit null/undefined checks prior to assignment, e.g. `val !== null && val !== undefined ? val : defaultVal` or `if (!x.cfg) x.cfg = defaultCfg;`.
- **Severity**: 💡 ADVISORY.
- **Fix**: Refactor to nullish coalescing (`val ?? defaultVal`) or logical nullish assignment (`x.cfg ??= defaultCfg`).

### Modern Array & Grouping Operations (ES2023 / ES2024)

- **Detection**: Manual `reduce` for grouping (`items.reduce((acc, x) => ...)`), mutating `.sort()`, or `reverse().find()`.
- **Severity**: 💡 ADVISORY.
- **Fix**: Refactor to native immutability and array features:
  - `Object.groupBy(items, x => x.category)` (ES2024)
  - `items.toSorted((a, b) => a - b)` (ES2023)
  - `items.findLast(x => x.active)` (ES2023)

### Python Structural Pattern Matching & Pipe Unions (Python 3.10+)

- **Detection**: Verbose `if/elif/isinstance` nested dict/event checks or legacy `typing.Union` / `Optional` imports.
- **Severity**: 💡 ADVISORY.
- **Fix**: Refactor to structural pattern matching (`match/case`) and pipe union type annotations (`str | int | None`).

### Rust `let-else` Flat Control Flow (Rust 1.65+)

- **Detection**: Nested `if let` blocks with immediate `else { return; }` divergence causing rightward drift.
- **Severity**: 💡 ADVISORY.
- **Fix**: Apply `let-else` statements (`let Some(val) = maybe_val else { return; };`) to maintain flat Happy-Path execution.

## Extending Rules for New Languages

Modernization rules are dynamically loaded and scoped by file extension defined in `skills/code-janitor/config.yaml`. To support new languages (e.g. Go, Python, C#, Rust):

1. Open `skills/code-janitor/config.yaml` (or your project's `skills.config.yaml`).
2. Add target file extensions under `modernization.rules.<rule_name>.extensions`.
3. Save the configuration. The scanner will automatically apply those rules to the configured file extensions.
