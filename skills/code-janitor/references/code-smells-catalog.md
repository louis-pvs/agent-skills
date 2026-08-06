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
