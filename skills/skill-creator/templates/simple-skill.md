---
name: {{SKILL_NAME}}
description: {{SKILL_DESCRIPTION}}
---

# {{SKILL_TITLE}}

{{SKILL_OVERVIEW}}

## Guidelines & Workflow

1. **Step 1: Input & Clarification**: Validate requirements. If parameters, strategy, or scope are ambiguous, call the `ask_question` tool with `(Recommended)` first-choice options rather than guessing or printing open-ended text.
2. **Step 2: Execution & Analysis**: Perform domain logic and validations.
3. **Step 3: Synthesis & Verification**: Generate final output and verify against constraints.

## Rules & Constraints

- Constraint 1: Rule to enforce.
- Constraint 2: Error prevention rule.
- **Interactive Decisions**: Whenever presenting architectural choices or configuration forks, use `ask_question` with structured options phrased as user intent.

## Completion Criteria

- [ ] Interactive decisions (if any) resolved using the `ask_question` tool protocol.
- [ ] Task execution completed without errors or unresolved exceptions.
- [ ] Output produced matches requested structure and parameters.
- [ ] Verification checks pass.
