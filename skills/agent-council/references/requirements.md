# Requirements

- Native compiled `agent-skills` CLI binary (located at `~/.cargo/bin/agent-skills` or `$env:USERPROFILE\.cargo\bin\agent-skills.exe`). If `agent-skills` is not recognized in the current shell, prepend `~/.cargo/bin` to `PATH` or invoke the binary by its full path.
- Install and authenticate the CLIs listed under `council.members` in `council.config.yaml`.
- Missing CLIs are automatically detected and marked as `missing_cli` in job status output without causing orchestrator crashes.
- Verify each member's base command exists (for example, `command -v <binary>` or `<binary> --version`).
- Supply explicit self-contained context (code diffs, ticket descriptions, snippets) directly in prompts to prevent headless sub-agent authentication or interactive approval failures.
