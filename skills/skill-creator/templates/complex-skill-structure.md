# Complex Skill Template Structure

This template defines the standard layout for multi-resource or script-augmented Agent Skills.

```text
skills/{{SKILL_NAME}}/
├── SKILL.md                 # Lean agent instruction protocol
├── README.md                # Human-facing landing page
├── config.yaml              # Optional configuration file
├── references/
│   └── overview.md          # Architecture & reference overview
├── templates/
│   └── sample_output.txt    # Optional resource templates
└── examples/
    └── usage_example.md     # Workflow usage examples
```

## `SKILL.md` Boilerplate for Complex Skill

````markdown
---
name: { { SKILL_NAME } }
description: { { SKILL_DESCRIPTION } }
---

# {{SKILL_TITLE}}

{{SKILL_OVERVIEW}}

## Workflow

1. **Initialization & Triage**: Read configuration or validate parameters. When multiple execution modes or ambiguous options exist, prompt the user via `ask_question` with `(Recommended)` first-choice options.
2. **Execution**: Run automation script

   ```bash
   agent-skills {{SKILL_NAME}} check
   ```

3. **Synthesis & Verification**: Process output, handle exceptions, and report results to user.

## Completion Criteria

- [ ] Interactive choices (if any) resolved using the `ask_question` tool protocol.
- [ ] All automation executions finish with exit code 0.
- [ ] Unit tests pass cleanly (`cargo test`).
- [ ] Output files and artifacts comply with required schema.

## References

- [interactive-decisions.md](references/interactive-decisions.md) — Interactive decision-making protocol.
- [overview.md](references/overview.md) — Extended design documentation.
````
