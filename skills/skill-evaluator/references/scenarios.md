# Benchmark Scenario Authoring Specification

Guide for creating declarative `scenario.yaml` task evaluations for agent skills.

---

## 1. File Location & Naming

Scenario files must be placed under:

```text
skills/<skill-name>/benchmarks/scenarios/<scenario_id>.yaml
```

---

## 2. YAML Schema Definition

```yaml
version: "1.0.0"
skill: "my-skill"
scenario_id: "example_task"
description: "High-level summary of what this scenario tests"
complexity: "low" # "low", "medium", or "high"

# Workspace & Execution Constraints
fixture_dir: "../fixtures/sample_repo" # Relative path to fixture directory
max_turns: 8
timeout_seconds: 60

# The exact prompt passed to both Baseline and Skill agents
prompt: |
  Perform task X on the codebase and output the verified result.

# Ground Truth Verification
eval_criteria:
  assertions:
    - type: "regex_match"
      pattern: "Expected summary output"
    - type: "max_tool_calls"
      threshold: 6.0
    - type: "command_pass"
      command: "cargo check"

  # Negative penalties
  penalties:
    - type: "file_read_count_exceeded"
      max_allowed: 10
      penalty_score: 0.2
```

---

## 3. Best Practices

1. **Deterministic Prompts**: Ensure prompts are self-contained and reproducible.
2. **Lean Fixtures**: Keep test fixture workspaces small (< 100 files) to minimize test run time.
3. **Strict Assertions**: Validate both output text and required file side-effects.
