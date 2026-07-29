# Complex Skill Template Structure

This template defines the standard layout for multi-resource or script-augmented Agent Skills.

```text
skills/{{SKILL_NAME}}/
├── SKILL.md
├── README.md                # Human-facing landing page
├── config.yaml              # Optional configuration file
├── references/
│   └── overview.md          # Mandatory architecture & reference overview
├── scripts/
│   ├── main.py              # Primary CLI orchestrator (ADR 0001 stdlib, ADR 0003 argparse)
│   └── tests/
│       └── test_main.py     # Unit test suite
├── templates/
│   └── sample_output.txt
└── examples/
    └── usage_example.md
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

1. **Initialization**: Read configuration or set up environment.
2. **Execution**: Run automation script

   ```bash
   python3 skills/{{SKILL_NAME}}/scripts/main.py --check
   ```

3. **Synthesis**: Process output and report results to user.

## Completion Criteria

- [ ] All automation script executions finish with exit code 0.
- [ ] Unit tests in `scripts/tests/` pass cleanly.
- [ ] Output files and artifacts comply with required schema.

## References

- [overview.md](references/overview.md) — Extended design documentation.
````
