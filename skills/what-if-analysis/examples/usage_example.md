# What-If Analysis Usage Examples

Practical workflow examples demonstrating how to use `what-if-analysis` before making code changes.

## Example 1: Full Pipeline via Native Rust CLI

```bash
# 1. Run blast-radius impact analysis for target function
cargo run -p agent-skills -- what-if-analysis impact --symbol calculate_score

# 2. Evaluate competing architectural scenarios
cargo run -p agent-skills -- what-if-analysis scenario --symbol calculate_score

# 3. Parse exact AST call sites in a specific source file
cargo run -p agent-skills -- what-if-analysis ast --symbol calculate_score --file src/core.rs

# 4. Auto-generate adversarial RED counterfactual unit test
cargo run -p agent-skills -- what-if-analysis counterfactual --symbol calculate_score

# 5. Intercept pre-emptive failure risks before committing
cargo run -p agent-skills -- what-if-analysis preempt --symbol calculate_score
```

## Example 2: JSON Output

```bash
# Get raw JSON metadata for downstream automation tools
cargo run -p agent-skills -- what-if-analysis impact --symbol parse_simple_yaml --json
```
