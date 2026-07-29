"""Base metrics models and protocol definitions for Benchmarking skill."""

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, Optional, Protocol, Tuple, runtime_checkable


class MetricStatus(Enum):
    PASS = "pass"
    FAIL = "fail"
    WARN = "warn"
    ERROR = "error"
    SKIPPED = "skipped"


@dataclass
class MetricResult:
    """Individual metric evaluation result."""

    name: str
    status: MetricStatus
    value: Any
    unit: Optional[str] = None
    threshold: Optional[Any] = None
    detail: str = ""
    raw: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "status": self.status.value,
            "value": self.value,
            "unit": self.unit,
            "threshold": self.threshold,
            "detail": self.detail,
            "raw": self.raw,
        }


@dataclass
class BenchmarkContext:
    """Shared telemetry context passed to all metric evaluators."""

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


@runtime_checkable
class MetricEvaluator(Protocol):
    """Protocol contract for custom and built-in metric evaluators."""

    name: str
    requires: Tuple[str, ...]

    def configure(self, config: Dict[str, Any]) -> None:
        """Apply metric configuration settings."""
        ...

    def evaluate(self, context: BenchmarkContext) -> MetricResult:
        """Compute and return metric result based on context."""
        ...
