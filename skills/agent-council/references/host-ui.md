# Host UI Checklist Guidance

Use these steps only when a host agent UI supports native checklist updates.

## Checklist Flow

1. Run `agent-skills agent-council wait "$JOB_DIR"` to monitor job progress.
2. Update the host's native checklist UI as member outputs are processed.
3. Finish with `agent-skills agent-council results "$JOB_DIR"` and `agent-skills agent-council clean "$JOB_DIR"`.

## Behavior Notes

- Keep exactly one `in_progress` item while member queries are executing.
- Preserve existing checklist items and append the `[Council]` section.
- Ensure all configured members are listed with their status (`Responded`, `Missing CLI`, `No Response`).
- Avoid blocking while loops in a single tool call; check status after each wait return.
