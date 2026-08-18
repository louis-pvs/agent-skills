# Agent Skills

A curated, open-source collection of procedural skills, automated CLI subcommands, and workflows for AI coding assistants (**Antigravity / Gemini CLI**, **Claude Code**, **GitHub Copilot**, and **Cursor**).

Engineered with a high-performance **compiled Rust core** (`agent-skills` CLI binary), these skills provide structured procedures, architectural guidelines, shift-left simulation, empirical benchmarking, and multi-agent synthesis tools to ensure clean code execution and reliable decision-making.

---

## 🚀 Available Skills (15 Native Rust Subcommands + Graphify)

| Skill | Description | Rust CLI Subcommand | Path |
| :--- | :--- | :--- | :--- |
| **`agent-council`** | Collect and synthesize opinions from multiple AI agents into a unified recommendation. | `agent-skills agent-council` | [`skills/agent-council`](skills/agent-council/SKILL.md) |
| **`agent-creator`** | Create, scaffold, configure, and validate Antigravity Custom Agents (`.agents/agents/*.md`). | `agent-skills agent-creator` | [`skills/agent-creator`](skills/agent-creator/SKILL.md) |
| **`architecture-decision-records`** | Log, index, supersede, and audit Architectural Decision Records (ADRs) following MADR standards. | `agent-skills adr` | [`skills/architecture-decision-records`](skills/architecture-decision-records/SKILL.md) |
| **`benchmarking`** | Empirical performance benchmarking, baseline differential analysis, and metric evaluations. | `agent-skills benchmarking` | [`skills/benchmarking`](skills/benchmarking/SKILL.md) |
| **`capability-gap-analyzer`** | Measure capability distance between registered agent skills and target project domains. | `agent-skills capability-gap-analyzer` | [`skills/capability-gap-analyzer`](skills/capability-gap-analyzer/SKILL.md) |
| **`code-janitor`** | Automated code hygiene scanner for dead code, oversized functions, and design principles (SOLID/CUPID/DRY). | `agent-skills code-janitor` | [`skills/code-janitor`](skills/code-janitor/SKILL.md) |
| **`context-gatherer`** | Gather deep codebase context: git temporal coupling, symbol navigation, and AST search. | `agent-skills context-gatherer` | [`skills/context-gatherer`](skills/context-gatherer/SKILL.md) |
| **`git-conflict-resolver`** | Intelligently resolve Git rebase or merge conflicts using 3-way markers and commit context. | `agent-skills git-conflict-resolver` | [`skills/git-conflict-resolver`](skills/git-conflict-resolver/SKILL.md) |
| **`self-annealer`** | Enforce bounded self-healing repair loops with strict iteration limits and automated git rollback. | `agent-skills self-annealer` | [`skills/self-annealer`](skills/self-annealer/SKILL.md) |
| **`self-progress`** | Identify session capability gaps, user corrections, and retrospective growth opportunities. | `agent-skills self-progress` | [`skills/self-progress`](skills/self-progress/SKILL.md) |
| **`skill-creator`** | Create, scaffold, format, and audit new Agent Skills adhering to agentskills.io standard. | `agent-skills skill-creator` | [`skills/skill-creator`](skills/skill-creator/SKILL.md) |
| **`skill-evaluator`** | Empirical agent skill evaluation, benchmark token savings, and ROI scorecard measurement. | `agent-skills skill-evaluator` | [`skills/skill-evaluator`](skills/skill-evaluator/SKILL.md) |
| **`tdd`** | Enforce Test-Driven Development (TDD) workflows: verify RED gate, write code, verify GREEN gate. | `agent-skills tdd` | [`skills/tdd`](skills/tdd/SKILL.md) |
| **`tech-doc-writer`** | Generate, refactor, and audit human-facing technical documentation and GFM compliance. | `agent-skills tech-doc-writer` | [`skills/tech-doc-writer`](skills/tech-doc-writer/SKILL.md) |
| **`what-if-analysis`** | Prospective blast-radius simulation, AST call-graph parsing, and counterfactual test generation. | `agent-skills what-if-analysis` | [`skills/what-if-analysis`](skills/what-if-analysis/SKILL.md) |

---

## 📦 Quick Onboarding & Installation

### Step 1: Install Rust & Build `agent-skills` CLI

Ensure you have [Rust & Cargo](https://rustup.rs/) installed, then build and install the binary globally:

```bash
# Clone the repository
git clone https://github.com/louis-pvs/agent-skills.git
cd agent-skills

# Option A: Install globally via Cargo
cargo install --path crates/cli

# Option B: Build local release binary
cargo build --release
```

---

### Step 2: Global Skill Linking (`agent-skills install`)

To automatically make all 17 skills available across your supported AI Coding Assistants:

```bash
# Execute the native installer (recommended)
agent-skills install

# Or run from source during local development
cargo run -p agent-skills-cli -- install
```

`agent-skills install` automatically validates the skill dependency graph and creates global symlinks across:

- 🧠 **Claude Code**: `~/.claude/skills/`
- 💎 **Gemini / Antigravity**: `~/.gemini/config/skills/`
- ✈️ **GitHub Copilot**: `~/.copilot/skills/`

---

## 💻 CLI Subcommand Usage Reference

```bash
# Verify skill dependency graph and lockfile integrity
agent-skills depgraph --verify

# Run Test-Driven Development (TDD) verification gates
agent-skills tdd --cmd "cargo test" --verify-red
agent-skills tdd --cmd "cargo test" --verify-green

# Scaffold & validate new agent skills
agent-skills skill-creator scaffold --name "my-skill" --description "My skill description"
agent-skills skill-creator validate --path skills/my-skill

# Scaffold & validate custom Antigravity agents
agent-skills agent-creator scaffold --name "code-reviewer"
agent-skills agent-creator validate --path .agents/agents/code-reviewer.md

# Evaluate agent skill ROI and token savings
agent-skills skill-evaluator check --skill graphify
agent-skills skill-evaluator run --skill graphify --mock

# Manage Architectural Decision Records (ADRs)
agent-skills adr new "Use Rust for CLI Automation"
agent-skills adr reindex

# Calculate blast radius for a symbol before refactoring
agent-skills what-if-analysis impact --symbol my_function

# Execute empirical performance benchmarks
agent-skills benchmarking run --cmd "cargo test" --iterations 5
```

---

## 🛠 Repository Architecture

```text
agent-skills/
├── README.md
├── Cargo.toml
├── skills.lock               # Skill dependency graph lockfile
├── .githooks/
│   └── pre-commit            # Git verification pipeline
├── crates/
│   ├── cli/                  # Unified agent-skills Rust CLI application
│   └── core/                 # Path safety (CWE-22), 4-tier config merger, depgraph
├── docs/
│   └── adr/                  # Architectural Decision Records
└── skills/                   # 17 Agent Skill folders (SKILL.md, README.md, references/)
```

---

## ✍️ Creating New Skills

To scaffold and register a new Agent Skill adhering to the `agentskills.io` standard:

```bash
agent-skills skill-creator scaffold --name "my-new-skill" --description "Description of trigger criteria"
```

After scaffolding:

1. Edit `skills/my-new-skill/SKILL.md` (keep main instructions under 500 lines for context load optimization).
2. Validate compliance: `agent-skills skill-creator validate --path skills/my-new-skill`.
3. Re-link globally: `agent-skills install`.

---

## 🧹 Development & Contribution Guidelines

Enable pre-commit verification hook once per clone:

```bash
git config core.hooksPath .githooks
```

The pre-commit hook enforces strict Rust code quality (ADR 0006):

- `cargo check --workspace`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `agent-skills depgraph --verify`
- `agent-skills lint-scripts`
- `npx markdownlint-cli "**/*.md"`

---

## 📄 License

MIT License. Free for use in personal and open-source projects.
