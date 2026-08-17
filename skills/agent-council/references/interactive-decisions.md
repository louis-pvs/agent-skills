# Interactive Decision-Making Protocol for Agent Council

The **Agent Council** uses the `ask_question` tool to eliminate ambiguity, align with the user before consuming external model tokens, and guide multi-agent consensus workflows.

---

## When to Call `ask_question`

### 1. Pre-Flight Alignment (Token Conservation)

Council queries fan out across multiple external AI models simultaneously (Claude, Copilot, Antigravity, Codex). Dispatching a vague, misaligned, or unrefined prompt wastes massive tokens and rate limits across all providers.

- **Mandatory Alignment**: When a user's question is broad or missing key architectural constraints (e.g., *"Which framework is better?"* without specifying performance constraints, ecosystem, or database requirements):
  - Explore the codebase locally first.
  - Formulate a precise, self-contained prompt with relevant inlined context.
  - Present the proposed question and target member lineup to the user via `ask_question` before launching background processes.
  - Always provide a `(Recommended)` option based on repository conventions.

### 2. Unavailable Member CLI Triage

When configured member CLIs are missing from `PATH` or fail to return responses:

- Alert the user in the report summary.
- If all or critical members are missing, invoke `ask_question` to ask the user how to proceed:
  - `(Recommended) Proceed with synthesis using responses from available members`
  - `Abort and wait until missing CLI is installed and authenticated in PATH`
  - `Update council.config.yaml to reconfigure active members`

### 3. Dissenting Opinion & Trade-Off Resolution

When council members propose competing architectures or dissenting trade-offs (e.g., Claude recommends Option A while Antigravity recommends Option B):

- Do not make arbitrary assumptions or force artificial unanimity.
- Present the core architectural trade-offs to the user via `ask_question` with a clear `(Recommended)` option.

### 4. Post-Synthesis Follow-Up Actions

After presenting the council synthesis and final recommendation, present actionable next steps:

- `(Recommended) Apply the council's recommended architecture refactoring`
- `Document this decision in an Architecture Decision Record (ADR)`
- `Run a what-if analysis simulation to evaluate blast radius`

---

## Question Formatting Rules

1. **Prefix Recommendations**: Always list the recommended option first with `(Recommended)`.
2. **User-Perspective Phrasing**: Phrase each option as the user's direct intent (e.g., `"Adopt PostgreSQL with connection pooling"`).
3. **No Redundant Fallbacks**: Never include manual "Other" or "None" options (the UI provides a write-in field by default).
4. **No Manual Numbering**: Avoid prefixing options with numbers or bullets.
