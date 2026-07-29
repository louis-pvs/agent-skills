"""Built-in metric evaluators for Benchmarking skill."""

from typing import Any, Dict

from .base import (
    BenchmarkContext,
    MetricResult,
    MetricStatus,
)


class TimingEvaluator:
    """Evaluates wall-clock duration in milliseconds."""

    name = "timing"
    requires = ()

    def __init__(self, max_duration_ms: float = 0.0):
        self.max_duration_ms = max_duration_ms

    def configure(self, config: Dict[str, Any]) -> None:
        if "max_duration_ms" in config:
            self.max_duration_ms = float(config["max_duration_ms"])

    def evaluate(self, context: BenchmarkContext) -> MetricResult:
        status = MetricStatus.PASS
        detail = f"Wall-clock runtime {context.wall_time_ms:.2f} ms"
        threshold = self.max_duration_ms if self.max_duration_ms > 0 else None

        if threshold and context.wall_time_ms > threshold:
            status = MetricStatus.FAIL
            detail += f" exceeded threshold {threshold:.2f} ms"

        return MetricResult(
            name=self.name,
            status=status,
            value=round(context.wall_time_ms, 2),
            unit="ms",
            threshold=threshold,
            detail=detail,
            raw={"wall_time_ms": context.wall_time_ms},
        )


class MemoryEvaluator:
    """Evaluates peak RAM memory allocation in Megabytes."""

    name = "memory"
    requires = ()

    def __init__(self, max_memory_mb: float = 0.0):
        self.max_memory_mb = max_memory_mb

    def configure(self, config: Dict[str, Any]) -> None:
        if "max_memory_mb" in config:
            self.max_memory_mb = float(config["max_memory_mb"])

    def evaluate(self, context: BenchmarkContext) -> MetricResult:
        status = MetricStatus.PASS
        detail = f"Peak RSS allocation {context.peak_memory_mb:.2f} MB"
        threshold = self.max_memory_mb if self.max_memory_mb > 0 else None

        if threshold and context.peak_memory_mb > threshold:
            status = MetricStatus.FAIL
            detail += f" exceeded threshold {threshold:.2f} MB"

        return MetricResult(
            name=self.name,
            status=status,
            value=round(context.peak_memory_mb, 2),
            unit="MB",
            threshold=threshold,
            detail=detail,
            raw={"peak_memory_mb": context.peak_memory_mb},
        )


class PassRatioEvaluator:
    """Evaluates process exit status / assertion pass ratio."""

    name = "pass_ratio"
    requires = ()

    def __init__(self, min_pass_ratio: float = 1.0):
        self.min_pass_ratio = min_pass_ratio

    def configure(self, config: Dict[str, Any]) -> None:
        if "min_pass_ratio" in config:
            self.min_pass_ratio = float(config["min_pass_ratio"])

    def evaluate(self, context: BenchmarkContext) -> MetricResult:
        passed = context.exit_code == 0
        ratio = 1.0 if passed else 0.0
        status = MetricStatus.PASS if ratio >= self.min_pass_ratio else MetricStatus.FAIL
        detail = "Process completed cleanly (exit code 0)" if passed else f"Process failed with exit code {context.exit_code}"

        return MetricResult(
            name=self.name,
            status=status,
            value=ratio,
            unit="ratio",
            threshold=self.min_pass_ratio,
            detail=detail,
            raw={"exit_code": context.exit_code},
        )
