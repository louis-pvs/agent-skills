---
name: tdd
description: 'Enforce Test-Driven Development (TDD) workflows: write failing test (RED), write minimal code (GREEN), and refactor. Use when the user requests TDD, "write tests first", or when building tested features.'
enhances:
  - domain-modeling
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
2. Write a minimal unit test asserting the expected behavior/contract for the new feature or bug fix. (For domain logic, check `domain-modeling` for aggregate invariants, value object rules, and state transition guards to use directly as test specifications).
3. Run `tdd_runner.py` with `--verify-red` to verify that the test fails as expected:

   ```bash
   python3 skills/tdd/scripts/tdd_runner.py --verify-red
   ```

   > [!IMPORTANT]
   > Do NOT write production implementation code until the test execution has failed and the RED state is verified.

### 2. Implement & GREEN (Verify Pass)

1. Write the minimal production code necessary to make the failing test pass. Avoid over-engineering.
2. Run `tdd_runner.py` with `--verify-green` to confirm that all tests pass:

   ```bash
   python3 skills/tdd/scripts/tdd_runner.py --verify-green
   ```

3. If tests fail, adjust implementation code (maximum 3 iterations) before seeking user guidance.

### 3. Refactor

1. Clean up code structure, variable naming, and duplication while maintaining design patterns (applying `domain-modeling` Ubiquitous Language to test descriptions and symbols).
2. Re-verify the GREEN state to ensure no regressions were introduced:

   ```bash
   python3 skills/tdd/scripts/tdd_runner.py --verify-green
   ```

### 4. Boundary & Edge Case Coverage

1. Add secondary test cases for boundary conditions, invalid inputs, and error states.
2. Re-run `--verify-green` to ensure complete suite stability.

---

## Automation CLI (`tdd_runner.py`)

The helper script handles test runner auto-detection (`pytest`, `unittest`, `npm test`, `jest`, `go test`, `cargo test`):

```bash
# Auto-detect project test runner
python3 skills/tdd/scripts/tdd_runner.py --detect

# Assert RED state (fails if tests pass)
python3 skills/tdd/scripts/tdd_runner.py --verify-red

# Assert GREEN state (fails if tests fail)
python3 skills/tdd/scripts/tdd_runner.py --verify-green

# Custom command override
python3 skills/tdd/scripts/tdd_runner.py --cmd "pytest tests/test_feature.py" --verify-green
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
