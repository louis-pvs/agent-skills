# Antigravity Custom Agent Specification & Schema

Antigravity Custom Agents are defined as single Markdown files (`.agents/agents/<name>.md` or `~/.gemini/config/agents/<name>.md`) with a YAML frontmatter header defining runtime parameters and a Markdown body defining the agent's system prompt.

---

## Frontmatter Schema

```yaml
---
name: string # (Required) Unique identifier (lowercase alphanumeric + hyphens)
description: string # (Required) Clear summary of the agent's role and specialization
model: string # (Optional) Model tier (e.g. gemini-3.7-flash, gemini-3.7-flash-thinking)
mainAgent: boolean # (Optional, default: true) Available as a primary interactive session
subagent: boolean # (Optional, default: true) Available to be spawned as a background subagent
tools: # (Optional) Explicitly enabled built-in or MCP tools
  - view_file
  - grep_search
  - replace_file_content
  - write_to_file
  - run_command
  - ask_question
skills: # (Optional) Skills pre-attached to the agent
  - tdd
  - self-annealer
---
```

---

## Field Specifications

### 1. `name`

- Must match regex `^[a-z0-9-]+$`.
- Keep concise (e.g., `qa-engineer`, `security-auditor`, `refactoring-bot`).

### 2. `description`

- Single or multi-line summary of what the agent does and when to dispatch tasks to it.
- If containing special characters or colons, enclose in quotes.

### 3. `model`

- **`gemini-3.7-flash`**: Fast, token-efficient. Best for test runners, formatters, and documentation bots.
- **`gemini-3.7-flash-thinking`**: High-reasoning depth. Best for architecture auditors, complex refactoring, and domain modeling.

### 4. `mainAgent` & `subagent` (Symmetry)

- `mainAgent: true`: Appears in the session selector / launchable via CLI or IDE chat.
- `subagent: true`: Can be spawned concurrently by other agents for background sub-tasks.

### 5. `tools` & `skills`

- Scoped to follow the **Principle of Least Privilege**.
- Pre-attach relevant domain skills (e.g., `tdd`, `code-janitor`) to avoid prompt bloat.

---

## File System Locations

1. **Workspace Scope**: `.agents/agents/<name>.md`
   - Shared with team members via Git version control.
2. **Global Scope**: `~/.gemini/config/agents/<name>.md`
   - Available across all local projects and workspaces.
