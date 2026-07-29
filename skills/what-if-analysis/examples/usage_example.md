# What-If Analysis Usage Examples

Practical workflow examples demonstrating how to use `what-if-analysis` before making code changes.

## Example 1: Full Pipeline via Unified `main.py` CLI

```bash
# 1. Run blast-radius impact analysis for target function
python3 skills/what-if-analysis/scripts/main.py impact --symbol calculate_score --dir .

# 2. Evaluate competing architectural scenarios
python3 skills/what-if-analysis/scripts/main.py scenario \
  --symbol calculate_score \
  --scenarios "Option A: Async Caching, Option B: In-Memory Calculation"

# 3. Parse exact AST call sites in a specific source file
python3 skills/what-if-analysis/scripts/main.py ast --symbol calculate_score --file src/core.py

# 4. Auto-generate adversarial RED counterfactual unit test
python3 skills/what-if-analysis/scripts/main.py counterfactual \
  --symbol calculate_score \
  --module src.core \
  --params "val,multiplier" \
  --out tests/test_counterfactual_calculate_score.py

# 5. Intercept pre-emptive failure risks before committing
python3 skills/what-if-analysis/scripts/main.py preempt --symbol calculate_score --dir .
```

## Example 2: Individual Script Execution & JSON Output

```bash
# Get raw JSON metadata for downstream automation tools
python3 skills/what-if-analysis/scripts/impact_analyzer.py --symbol parse_simple_yaml --json
```
