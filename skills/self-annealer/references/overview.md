# Self-Annealer Overview & Repair Invariants

Self-Annealer enforces bounded, closed-loop repair cycles for failing tests, lint errors, and execution failures. It prevents infinite debugging loops by establishing strict convergence limits and git rollback safety gates.

---

## The Problem & Friction

When automated tests or linters fail, LLMs frequently enter infinite retry loops, making superficial symptom patches (swallowing exceptions, deleting failing tests, or adding arbitrary delays) that degrade code quality without resolving the root cause.

Self-Annealer eliminates this friction by binding the agent to an iterative repair loop with an explicit maximum attempt limit and automated git safeguards.

---

## Convergence Repair Loop

```mermaid
flowchart TD
    Start[Detect Error / Failure] --> CheckLimit{Attempts < Max?}
    CheckLimit -- Yes --> RunDiagnosis[Inspect Error & Log Traceback]
    RunDiagnosis --> FormHypothesis[Form Single Target Hypothesis]
    FormHypothesis --> ApplyPatch[Apply Surgical Code Patch]
    ApplyPatch --> RunVerification[Run Test / Linter Verification]
    RunVerification -- Success --> SuccessEnd[Annealing Succeeded (Exit 0)]
    RunVerification -- Failed --> IncrementAttempt[Increment Attempt Count]
    IncrementAttempt --> CheckLimit
    CheckLimit -- No (Exceeded) --> GitRollback[Git Rollback to Baseline]
    GitRollback --> FailEnd[Annealing Failed (Exit 1)]
```

---

## Core Invariants

> [!IMPORTANT]
> A repair attempt MUST NOT alter or comment out existing test assertions unless explicitly requested by the user.

> [!WARNING]
> If the maximum attempt limit (default: 3 iterations) is reached without convergence, the skill automatically executes `git checkout -- .` to restore the last clean baseline state.

---

## Architectural Trade-Offs

| Decision | Benefit | Trade-off / Cost |
| :--- | :--- | :--- |
| Hard limit on retry count | Prevents token drain and infinite loops | Stops prematurely on complex multi-bug errors |
| Automatic git rollback | Guarantees codebase remains in working state | Discards partial fixes if convergence fails |
| Mandatory log inspection | Ensures evidence-based root cause analysis | Requires log retrieval overhead per step |
