# Clean Code Heuristics

Reference guide for Clean Code principles applied during code janitorial scans.

## Core Principles

### 1. Boy Scout Rule

> Leave the code cleaner than you found it.

Every time code is touched (bug fix, feature addition, refactoring), improve at least one
nearby hygiene issue. This prevents gradual quality decay.

### 2. Meaningful Naming

- **Variables**: Reveal intent (`elapsed_time_in_days` not `d`).
- **Functions**: Use verb phrases (`calculate_total()` not `do_stuff()`).
- **Classes**: Use noun phrases (`InvoiceProcessor` not `Doer`).
- **Booleans**: Use predicates (`is_valid`, `has_permission`, `can_retry`).
- **Avoid**: Single-letter names (except in short lambdas/comprehensions), abbreviations,
  Hungarian notation, noise words (`data`, `info`, `manager`).

### 3. Small Functions

- **Target**: Functions should do one thing and do it well.
- **Line limit**: Aim for < 20 lines; flag functions > 30 lines as `⚠️ WARNING`.
- **Indentation**: Body should not exceed 2 levels of indentation ideally; flag > 4 levels
  as `⚠️ WARNING`.
- **Parameters**: Ideal is 0-2; 3 is acceptable; > 5 is `⚠️ WARNING` (consider a
  parameter object or builder pattern).

### 4. Single Responsibility

Each function, class, or module should have exactly one reason to change. Symptoms of
violation:

- Function name contains "and" (e.g., `parse_and_validate`).
- Class has methods spanning unrelated concerns.
- Module imports from 5+ unrelated domains.

### 5. DRY (Don't Repeat Yourself)

Eliminate **knowledge duplication** (not incidental code similarity):

- If logic must change in multiple places simultaneously, it violates DRY.
- Apply the **Rule of Three**: tolerate first duplication, refactor on third occurrence.
- Do not over-abstract prematurely (see YAGNI).

### 6. KISS (Keep It Simple, Stupid)

- Prefer straightforward solutions over clever abstractions.
- Avoid unnecessary indirection layers.
- Cyclomatic complexity > 10 per function is a strong signal of excess complexity.

### 7. YAGNI (You Ain't Gonna Need It)

- Do not build speculative abstractions for future requirements.
- Dead code, unused parameters, and commented-out blocks are YAGNI violations.
- Remove rather than comment out.

## Severity Calibration

| Level | Criteria | Examples |
| :--- | :--- | :--- |
| 🚨 CRITICAL | Actively harmful; hides bugs or breaks maintainability | Dead code after return, unreachable branches, shadowed variables |
| ⚠️ WARNING | Degrades readability or maintainability over time | Oversized functions, deep nesting, too many parameters |
| 💡 ADVISORY | Minor improvement opportunity | Missing docstring, stale TODO, minor naming issue |
