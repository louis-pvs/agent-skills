# Context Gatherer — Usage Examples

## Example 1: Impact Analysis Before a Refactor

You're about to refactor `src/auth/session.py`. Find what else will need to change:

```bash
# Find temporally coupled files
agent-skills context-gatherer git-coupling --file src/auth/session.py --min-commits 2

# Output:
# Temporal coupling for: src/auth/session.py
#
# File                                                      Co-commits    Ratio
# -------------------------------------------------------------------------------
# src/auth/middleware.py                                            12     0.80
# tests/test_session.py                                            10     0.67
# src/api/endpoints.py                                              5     0.33
# docs/auth.md                                                      3     0.20
```

This tells you: every time `session.py` changes, `middleware.py` changes too (80% of the time). Don't forget to update the tests and docs.

## Example 2: Find All Callers of a Function

Before renaming `validate_token`, find every place that calls it:

```bash
# Find where validate_token is defined
agent-skills context-gatherer symbol-nav --symbol "validate_token" --path src/ --type definition

# Find all usages (excluding the definition)
agent-skills context-gatherer symbol-nav --symbol "validate_token" --path src/ --type reference
```

## Example 3: Find All Handler Implementations

Find every class that extends `BaseHandler`:

```bash
agent-skills context-gatherer ast-search --pattern "class * (BaseHandler)" --path src/
```

Find all test functions:

```bash
agent-skills context-gatherer ast-search --pattern "def test_*" --path tests/
```

## Example 4: Combined Deep Context

For maximum understanding before a major change, combine techniques:

```bash
# 1. Check graphify first (if available)
# /graphify query "How does the auth module connect to the API layer?"

# 2. Find co-changing files
agent-skills context-gatherer git-coupling --file src/auth/session.py --format json

# 3. Find all references to the symbol you're changing
agent-skills context-gatherer symbol-nav --symbol "SessionManager" --path src/

# 4. Find related implementations
agent-skills context-gatherer ast-search --pattern "class * (SessionManager)" --path src/
```

## Example 5: JSON Output for Programmatic Use

All commands support `--json` / `--format json` for piping into other tools:

```bash
agent-skills context-gatherer git-coupling --file src/app.py --format json
```
