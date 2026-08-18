---
name: tdd
description: 'Enforce Test-Driven Development (TDD) workflows: write failing test (RED), write minimal code (GREEN), and refactor. Use when the user requests TDD, "write tests first", or when building tested features.'
---

# TDD (Test-Driven Development)

Enforce strict test-driven development cycles for code additions and bug fixes. Guarantees that specifications are verifiably asserted before implementation code is written.

## When to Use

- When the user explicitly requests TDD (`"implement X with TDD"`, `"write tests first"`, `"TDD workflow"`).
- When implementing critical business logic or core algorithms requiring high test confidence.
- When fixing regressions or bugs by creating reproduction tests first.

## Exemption & Bail-out Conditions

**Do NOT force TDD when:**

- Making pure documentation or markdown updates.
- Performing UI layout tweaks or exploratory spikes/prototypes.
- Operating in a repository with no existing or supportable test framework.

---

## Procedural Workflow (RED → GREEN → REFACTOR)

Follow these 4 steps sequentially:

### 1. Specify & RED (Verify Failure)

1. Identify or create the appropriate test file in the project.
2. Write a minimal unit test asserting the expected behavior/contract for the new feature or bug fix. (For domain logic, identify aggregate invariants, value object rules, and state transition guards to use directly as test specifications).
3. Run `agent-skills tdd` with `--verify-red` to verify that the test fails as expected:

   ```bash
   agent-skills tdd --verify-red
   ```

   > [!IMPORTANT]
   > Do NOT write production implementation code until the test execution has failed and the RED state is verified.

### 2. Implement & GREEN (Verify Pass)

1. Write the minimal production code necessary to make the failing test pass. Avoid over-engineering.
2. Run `agent-skills tdd` with `--verify-green` to confirm that all tests pass:

   ```bash
   agent-skills tdd --verify-green
   ```

3. If tests fail, adjust implementation code (maximum 3 iterations) before seeking user guidance.

### 3. Refactor

1. Clean up code structure, variable naming, and duplication while maintaining design patterns and clean ubiquitous naming across test descriptions and symbols.
2. Re-verify the GREEN state to ensure no regressions were introduced:

   ```bash
   agent-skills tdd --verify-green
   ```

### 4. Boundary & Edge Case Coverage

1. Add secondary test cases for boundary conditions, invalid inputs, and error states.
2. Re-run `--verify-green` to ensure complete suite stability.

---

## Automation CLI (`agent-skills tdd`)

The CLI handles test runner auto-detection (`pytest`, `unittest`, `npm test`, `jest`, `go test`, `cargo test`):

```bash
# Auto-detect project test runner
agent-skills tdd --detect

# Assert RED state (fails if tests pass)
agent-skills tdd --verify-red

# Assert GREEN state (fails if tests fail)
agent-skills tdd --verify-green

# Custom command override
agent-skills tdd --cmd "pytest tests/test_feature.py" --verify-green
```

---

## Completion Criteria

- [ ] Minimal failing test written and verified in RED state (`--verify-red` exit code 0).
- [ ] Minimal implementation code written and verified in GREEN state (`--verify-green` exit code 0).
- [ ] Code refactored with all unit tests passing cleanly.
- [ ] Boundary conditions and edge cases tested.

---

## References

- [overview.md](references/overview.md) — TDD principles and agent self-healing loops.
- [runners.md](references/runners.md) — Supported test runners and framework auto-detection matrix.
