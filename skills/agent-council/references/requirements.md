# Requirements

- Python 3.8+ (Standard Library only; zero external `pip` or `npm` dependencies).
- Install and authenticate the CLIs listed under `council.members` in `council.config.yaml`.
- Missing CLIs are automatically detected via `shutil.which` pre-checks and marked as `missing_cli` in job status output without causing orchestrator crashes.
- Verify each member's base command exists (for example, `command -v <binary>` or `<binary> --version`).
- Supply explicit self-contained context (code diffs, ticket descriptions, snippets) directly in prompts to prevent headless sub-agent authentication or interactive approval failures.
