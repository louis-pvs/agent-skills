---
name: self-annealer
description: Enforce bounded self-healing repair loops with strict convergence limits and automated git rollback safeguards. Use when fixing failing tests or lint errors without infinite loops.
requires:
  - tdd
enhances:
  - graphify
version: 1.0.0
---

# Self Annealer

Enforce bounded self-healing repair loops (RED -> GREEN -> REFACTOR) with strict iteration limits and automated git rollback safeguards to prevent infinite repair loops and structural debt ("slob").

## When to Use

- When attempting automated bug fixes or lint repairs.
- When running iterative refactoring loops against test suites.
- When repairing broken builds while guaranteeing rollback if repairs fail to converge.

## Procedural Workflow (Bounded Convergence Loop)

Follow these 4 steps:

### 1. Capture State Checkpoint

Ensure uncommitted work is clean or stashed before launching annealing loops:

```bash
git status --porcelain
```

### 2. Launch Bounded Annealing Loop

Run `anneal_runner.py` with your test command and iteration cap (default max 3 attempts):

```bash
python3 skills/self-annealer/scripts/anneal_runner.py --cmd "python3 skills/tdd/scripts/tdd_runner.py --verify-green" --max-iterations 3
```

### 3. Iterative Refinement

For each iteration up to `--max-iterations`:

1. Read error traceback / `ruff` lint output.
2. Formulate a minimal diagnostic hypothesis.
3. Apply targeted code fix.
4. Re-evaluate test command.

### 4. Convergence & Escalation

- **GREEN (Converged)**: All tests/linters pass within 3 iterations. Commit changes.
- **RED (Failed)**: If 3 iterations expire without reaching GREEN status, `anneal_runner.py` automatically triggers `git checkout -- .` to revert uncommitted changes and output a structured failure report for human escalation.

## Automation CLI (`anneal_runner.py`)

```bash
# Run self-annealing loop with default test runner (3 iterations max + auto rollback)
python3 skills/self-annealer/scripts/anneal_runner.py

# Custom command with JSON output
python3 skills/self-annealer/scripts/anneal_runner.py --cmd "pytest tests/" --max-iterations 3 --json
```

## Completion Criteria

- [ ] Repair loop converged to GREEN status within the iteration limit (`--max-iterations 3`).
- [ ] If non-convergent, automated git rollback executed cleanly to prevent slob/regression.
- [ ] All unit tests pass cleanly.

## References

- [overview.md](references/overview.md) — Self-annealing principles, temperature scaling, and rollback mechanics.
