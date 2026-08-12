# Git Conflict Resolver

Intelligently resolve Git rebase or merge conflicts using repository context, commit histories, AST references, and post-resolution test verification.

---

## Documentation Entry Points

- **AI Agent Protocol**: See [SKILL.md](SKILL.md) (used automatically by Antigravity, Claude Code, and Copilot).
- **3-Way Merge Architecture**: See [references/overview.md](references/overview.md).
- **Resolution Heuristics**: See [references/resolution-strategies.md](references/resolution-strategies.md).
- **Resolution Report Template**: See [templates/resolution_report.md](templates/resolution_report.md).

## Features

- **Git State Detection**: Automatically detects `merge`, `rebase`, `cherry-pick`, and `revert` states.
- **3-Way Marker Parsing**: Extracts `OURS`, `BASE`, and `THEIRS` blocks with line numbers.
- **Commit History Context**: Integrates recent commit messages and branch intent into conflict decisions.
- **Automated Verification**: CLI tool verifies zero remaining conflict markers and validates clean test runs.

## CLI Usage

```bash
# Check health of git-conflict-resolver skill files
agent-skills git-conflict-resolver check

# Deep analysis of all unmerged files
agent-skills git-conflict-resolver analyze --json

# Verify zero conflict markers remain
agent-skills git-conflict-resolver analyze --verify
```
