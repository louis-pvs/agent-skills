# Metric Evaluators Reference & Plugin Guide

The Benchmarking Skill features an expandable metric evaluator framework. Built-in evaluators provide core execution metrics, while custom evaluators can be added dynamically.

---

## 1. Built-in Evaluators

| Evaluator Name | Metric Measured           | Unit    | Default Threshold | Description                                                                                  |
| -------------- | ------------------------- | ------- | ----------------- | -------------------------------------------------------------------------------------------- |
| `timing`       | Wall-clock execution time | `ms`    | Configurable      | Measures execution duration using high-precision performance timers (`time.perf_counter()`). |
| `memory`       | Peak RAM heap allocation  | `MB`    | Configurable      | Tracks peak Resident Set Size / memory allocation using `tracemalloc`.                       |
| `pass_ratio`   | Process exit status       | `ratio` | `1.0`             | Asserts process return code (1.0 for exit code 0, 0.0 for failure).                          |

---

## 2. Custom Metric Evaluator Interface

Custom metric plugins must implement the `MetricEvaluator` contract.

```python
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, Optional


class MetricStatus(Enum):
    PASS = "pass"
    FAIL = "fail"
    WARN = "warn"
    ERROR = "error"
    SKIPPED = "skipped"


@dataclass
class MetricResult:
    name: str
    status: MetricStatus
    value: Any
    unit: Optional[str] = None
    threshold: Optional[Any] = None
    detail: str = ""
    raw: Dict[str, Any] = field(default_factory=dict)


@dataclass
class BenchmarkContext:
    command: str
    cwd: str
    stdout: str
    stderr: str
    exit_code: int
    wall_time_ms: float
    peak_memory_mb: float
    baseline: Optional[Dict[str, Any]] = None
    artifacts: Dict[str, str] = field(default_factory=dict)
    config: Dict[str, Any] = field(default_factory=dict)
```

---

## 3. Creating a Custom Metric Evaluator Plugin

Create a `.py` file inside your custom metric directory (e.g., `.benchmarking/metrics/lint_evaluator.py`):

```python
"""Custom Metric Plugin Example: Lint Warning Count."""

from skills.benchmarking.scripts.evaluators.base import (
    BenchmarkContext,
    MetricResult,
    MetricStatus,
)


class LintWarningEvaluator:
    name = "lint_warnings"
    requires = ()

    def configure(self, config: dict) -> None:
        self.max_warnings = config.get("max_warnings", 0)

    def evaluate(self, context: BenchmarkContext) -> MetricResult:
        # Example logic parsing warnings from stderr or stdout
        warning_count = context.stderr.count("WARNING") + context.stdout.count("warning")
        status = MetricStatus.PASS if warning_count <= self.max_warnings else MetricStatus.FAIL

        return MetricResult(
            name=self.name,
            status=status,
            value=warning_count,
            unit="warnings",
            threshold=self.max_warnings,
            detail=f"Found {warning_count} warning(s) (max allowed: {self.max_warnings})",
        )


# Module export hook for auto-discovery
METRIC = LintWarningEvaluator()
```

Run the benchmark:

```bash
agent-skills benchmarking run \
  --cmd "cargo test --workspace" \
  --metrics "timing,memory,pass_ratio"
```
