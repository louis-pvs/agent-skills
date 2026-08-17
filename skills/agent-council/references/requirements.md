# Requirements

## Environment & Preparation

- Native compiled `agent-skills` CLI binary (located at `~/.cargo/bin/agent-skills` or `$env:USERPROFILE\.cargo\bin\agent-skills.exe`).
- Shell session preparation:
  - **PowerShell (Windows)**: `$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:APPDATA\npm;$env:PATH"`
  - **Bash / Zsh (Unix)**: `export PATH="$HOME/.cargo/bin:$PATH"`
  - If `agent-skills` is not recognized directly, invoke by full path.
- Run `agent-skills agent-council doctor` to verify detected binary locations and versions across configured members.

## Member CLIs & Execution

- Install and authenticate the CLIs listed under `council.members` in `council.config.yaml`.
- Missing CLIs are automatically detected and marked as `missing_cli` in job metadata without causing orchestrator crashes.
- The host agent must explicitly report any missing, failed, or non-responding member CLIs to the user in the final council report.
- Verify each member's base command exists via `agent-skills agent-council doctor`.
- Supply explicit self-contained context (code diffs, ticket descriptions, snippets) directly in prompts to prevent headless sub-agent authentication or interactive approval failures.
