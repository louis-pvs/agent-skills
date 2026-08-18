# GitHub Copilot Instructions

Follow the universal repository directives defined in [AGENTS.md](../AGENTS.md):

- **Knowledge Graph Navigation**: Run `graphify query "<concept>"` before falling back to manual grep.
- **Canonical Binary**: Use `agent-skills <subcommand>` directly for skill execution and linting.
- **Crate & Package**: The crate in `crates/cli` is `agent-skills-cli` (binary: `agent-skills`).
- **Development**: Use `cargo run -p agent-skills-cli -- <args>` for live workspace testing.
