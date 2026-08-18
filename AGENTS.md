# Agent Directives & Repository Architecture

This document defines the universal operating invariants, tool execution standards, and architecture patterns for all AI coding assistants (**Antigravity**, **Gemini CLI**, **Claude Code**, **GitHub Copilot**, and **Cursor**) working inside this repository.

---

## 1. Codebase Navigation & Knowledge Graph (Graphify First)

- **Pre-Indexed Knowledge Graph**: An indexed knowledge graph is maintained in `graphify-out/`.
- **Mandatory Invariant**: When exploring codebase architecture, symbol locations, call graphs, or cross-skill dependencies, **always run `graphify query "<concept>"` first**.
- **Avoid Brute-Force Grep Loops**: Do NOT execute multiple iterative `grep_search` and whole-file reads before querying `graphify`.
- **Targeted Reading**: Once `graphify query` identifies the target file and line numbers, use targeted line-bounded reads to inspect exact implementation code.

```bash
# Query the codebase knowledge graph
graphify query "MetricRegistry"
graphify query "circular dependencies"
```

---

## 2. CLI Tool Execution & Binary Resolution

- **Canonical Binary Name**: The compiled CLI binary is **`agent-skills`** (installed in `$env:USERPROFILE\.cargo\bin` / `~/.cargo/bin` and present in `$env:PATH`).
- **Cargo Package Name**: The Cargo package located in `crates/cli` is named **`agent-skills-cli`** with explicit binary target `agent-skills`.
- **Primary Subcommand Execution**: Always invoke **`agent-skills <subcommand>`** directly for all skill operations, linting, and verifications.
- **Development Source Testing**: Use `cargo run -p agent-skills-cli -- <subcommand>` **only** when actively modifying Rust source code inside `crates/` in the current session to test uninstalled live changes.

```bash
# Direct execution (standard)
agent-skills lint-scripts
agent-skills depgraph --verify
agent-skills skill-evaluator run --skill graphify --mock

# Development execution (when modifying crates/ source)
cargo run -p agent-skills-cli -- lint-scripts
cargo run -p agent-skills-cli -- depgraph --verify
```

---

## 3. Repository Architecture

```text
agent-skills/
├── AGENTS.md                   # Universal agent directives (this file)
├── CLAUDE.md                   # Claude Code entrypoint (points to AGENTS.md)
├── Cargo.toml                  # Workspace root
├── skills.lock               # Skill dependency graph lockfile
├── .githooks/pre-commit        # Git verification hook
├── .agents/rules/              # Antigravity/Gemini workspace rules
├── crates/
│   ├── cli/                    # Package: agent-skills-cli (Binary: agent-skills)
│   └── core/                   # Package: agent_skills_core (Path safety, depgraph, config)
├── docs/adr/                   # Architectural Decision Records (MADR)
├── graphify-out/               # Pre-built architectural knowledge graph
└── skills/                     # 16 Agent Skills conforming to agentskills.io standard
```

---

## 4. Mandatory Pre-Completion Verification Gates

Before completing any task or code modification, verify that all quality gates pass cleanly:

1. `cargo fmt --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo test --workspace`
4. `agent-skills lint-scripts` (or `cargo run -p agent-skills-cli -- lint-scripts`)
5. `agent-skills depgraph --verify` (or `cargo run -p agent-skills-cli -- depgraph --verify`)
6. `npx -y markdownlint-cli "**/*.md"`
