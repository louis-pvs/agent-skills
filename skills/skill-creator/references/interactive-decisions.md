# Interactive Decision-Making Protocol (`ask_question` Pattern)

## Purpose

When AI agents interact with users, open-ended or ambiguous questions in plain text often create cognitive friction and inefficient back-and-forth communication. The **Interactive Decision-Making Protocol** mandates using structured, tidy selection modals via the `ask_question` tool whenever an agent encounters design forks, ambiguous requirements, or configuration options.

---

## Core Principles

### 1. Mandatory Tool Usage for Decisions

- Whenever an agent reaches an ambiguous requirement, architectural choice, or branch in execution, it **MUST** call the `ask_question` tool.
- Do not output plain text multiple-choice lists (`1. Option A, 2. Option B...`) in markdown if a decision is required from the user.
- Trivial single-word confirmations (e.g. simple yes/no) can be asked directly in text, but all design/configuration choices MUST use `ask_question`.

### 2. Option Formatting Rules

1. **`(Recommended)` Convention**:
   - Always place the best-practice or recommended option as the very first item in `options`.
   - Prefix the string with `(Recommended)`.
2. **User-Perspective Phrasing**:
   - Phrase each option as the user's direct intent or response (e.g., `"Use PostgreSQL with connection pooling"`, `"Deploy to staging first"`).
   - Do NOT describe the agent's action in third-person (e.g., avoid *"The agent will set up PostgreSQL"*).
3. **No Redundant "Other" Option**:
   - The UI automatically provides a custom write-in field. Never add `"Other"`, `"None of the above"`, or similar fallback items.
4. **No Manual Numbering or Bullets**:
   - The UI handles indexing automatically. Do not prefix options with numbers (`1.`, `2.`) or bullet points (`-`).
5. **Selection Cardinality (`is_multi_select`)**:
   - Set `is_multi_select: false` (default) for mutually exclusive choices (radio buttons).
   - Set `is_multi_select: true` when multiple independent features or items can be selected at once (checkboxes).

---

## Question Schema & Example

```json
{
  "question": "Which database engine and migration strategy should we configure for the new microservice?",
  "options": [
    "(Recommended) PostgreSQL with automated Flyway migrations",
    "PostgreSQL with manual SQL scripts",
    "SQLite for local development and in-memory testing",
    "MySQL 8.0 with Liquibase migrations"
  ],
  "is_multi_select": false
}
```

---

## Sequential Dependency Resolution

When a workflow requires multiple interdependent decisions:

1. **Ask One Question at a Time**: Resolve the top-level architecture decision before asking about granular details.
2. **Branch Dynamically**: Use the user's response to Question 1 to determine the options presented in Question 2.
3. **Explore Codebase First**: If a question can be answered deterministically by inspecting files, checking configs, or running a search tool, explore the codebase first before bothering the user.
