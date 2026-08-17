# Principle of Least Privilege: Tool Scoping for Custom Agents

## Overview

A critical best practice when authoring custom agents is restricting tools to only those strictly necessary for the agent's assigned role. Unconstrained tool access increases the risk of unintended side-effects and inflates token overhead.

---

## Tool Archetype Matrix

| Agent Role | Recommended Tools | Excluded Tools | Rationale |
| :--- | :--- | :--- | :--- |
| **Code Reviewer / Auditor** | `view_file`, `grep_search`, `ask_question` | `write_to_file`, `replace_file_content`, `run_command` | Purely analytical; prevents inadvertent modifications to production files. |
| **QA / TDD Engineer** | `view_file`, `write_to_file`, `replace_file_content`, `run_command`, `ask_question` | Browser tools (unless UI testing) | Needs code authoring and terminal execution to verify unit tests. |
| **Documentation Writer** | `view_file`, `write_to_file`, `replace_file_content`, `ask_question` | `run_command` | Documentation tasks should not require arbitrary system shell execution. |
| **Security Auditor** | `view_file`, `grep_search`, `ask_question` | `write_to_file`, `replace_file_content` | Safe read-only inspection for secret leakage and vulnerability scans. |
| **Frontend / Web Specialist** | `view_file`, `write_to_file`, `replace_file_content`, `browser_subagent`, `ask_question` | Server management tools | Interacts with UI and validates rendering in browser subagent. |

---

## Guidelines for Authoring Agents

1. **Default to Read-Only**: Unless the agent's explicit mandate is code generation or refactoring, do not grant write permissions.
2. **Isolate Shell Execution**: Only grant `run_command` when automated tests, builds, or script runners must be validated directly.
3. **Mandate `ask_question`**: Always include `ask_question` so the agent can interactively clarify parameters and present options cleanly.
