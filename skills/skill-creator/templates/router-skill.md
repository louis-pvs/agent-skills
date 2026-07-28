---
name: {{SKILL_NAME}}
description: {{SKILL_DESCRIPTION}}
disable-model-invocation: true
---

# {{SKILL_TITLE}}

Router skill for dispatching task requests to specialized sub-skills or workflows.

## Sub-Skill Directory & Routing Map

| Intent / Task | Skill / Resource | Description |
| :--- | :--- | :--- |
| Task Category A | `sub-skill-a` | Description of when to use sub-skill A. |
| Task Category B | `sub-skill-b` | Description of when to use sub-skill B. |
| Task Category C | [reference-doc.md](references/reference-doc.md) | Deep reference documentation for C. |

## Guidance for Agent

1. Inspect the user request against the routing map above.
2. Direct the workflow to the appropriate sub-skill or reference file.
3. If no matching sub-skill applies, request clarification from the user.

## Completion Criteria

- [ ] Matched user request to target sub-skill or reference document.
- [ ] Invoked or presented the relevant skill path cleanly.
