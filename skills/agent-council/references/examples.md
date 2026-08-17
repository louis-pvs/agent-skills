# Examples

## Technical Decision with Pre-Flight Alignment

Prompt:

```text
React vs Vue - which fits this project better? Summon the council
```

Steps:

1. **Pre-Flight Context Gathering**: Inspect `package.json`, existing UI components, and TypeScript configs.
2. **Pre-Flight Alignment (`ask_question`)**:
   Prompt the user:
   - `(Recommended) Query the council on React vs Vue focused on TypeScript integration and existing component migration cost`
   - `Query the council focused strictly on bundle size and runtime performance`
   - `Include Svelte in the council comparison`
3. **Execution**:
   Run `agent-skills agent-council start "<formulated_prompt>"` to launch queries to active members.
4. **Monitoring**:
   Run `agent-skills agent-council wait "$JOB_DIR"` to await process completion.
5. **Collection & Status Report**:
   Run `agent-skills agent-council results "$JOB_DIR"` to collect outputs and member status.
6. **Consensus Synthesis & Post-Flight Decision**:
   Report member availability status and synthesize consensus recommendations. Use `ask_question` to select next actions (e.g. create ADR).
7. **Cleanup**:
   Run `agent-skills agent-council clean "$JOB_DIR"`.

---

## Architecture Review

Prompt:

```text
Let's hear other AIs' opinions on this design
```

Steps:

1. Summarize the design with full self-contained context and verify alignment with user.
2. Launch via `agent-skills agent-council start "<prompt>"`.
3. Await completion and gather responses with `agent-skills agent-council results "$JOB_DIR"`.
4. Check and report member availability breakdown.
5. Synthesize common patterns, highlight tradeoffs, and note dissenting views.
6. Clean up temporary files with `agent-skills agent-council clean "$JOB_DIR"`.
