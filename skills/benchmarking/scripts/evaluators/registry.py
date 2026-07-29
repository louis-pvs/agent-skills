"""Metric Registry and dynamic plugin discovery for Benchmarking skill."""

import importlib.util
from pathlib import Path
from typing import Any, Dict, List, Optional

from .builtins import (
    MemoryEvaluator,
    PassRatioEvaluator,
    TimingEvaluator,
)


class MetricRegistry:
    """Central registry for managing built-in and user-registered metric evaluators."""

    def __init__(self):
        self._evaluators: Dict[str, Any] = {}

    def register(self, evaluator: Any) -> None:
        """Register a metric evaluator instance."""
        name = getattr(evaluator, "name", None)
        if not name:
            raise ValueError("Evaluator must define a 'name' attribute.")
        self._evaluators[name] = evaluator

    def register_defaults(self) -> None:
        """Register standard built-in metric evaluators."""
        self.register(TimingEvaluator())
        self.register(MemoryEvaluator())
        self.register(PassRatioEvaluator())

    def get(self, name: str) -> Optional[Any]:
        """Retrieve a registered evaluator by name."""
        return self._evaluators.get(name)

    def list_evaluators(self) -> List[str]:
        """List names of all registered evaluators."""
        return list(self._evaluators.keys())

    def load_from_directory(self, directory: Path) -> int:
        """Dynamically load custom metric evaluators from Python files in directory."""
        if not directory.is_dir():
            return 0

        loaded = 0
        for filepath in directory.glob("*.py"):
            if filepath.name.startswith("_") or filepath.name.startswith("test_"):
                continue

            mod_name = f"custom_metric_{filepath.stem}"
            spec = importlib.util.spec_from_file_location(mod_name, filepath)
            if spec and spec.loader:
                mod = importlib.util.module_from_spec(spec)
                try:
                    spec.loader.exec_module(mod)
                    if hasattr(mod, "METRIC"):
                        self.register(mod.METRIC)
                        loaded += 1
                    elif hasattr(mod, "get_evaluator"):
                        self.register(mod.get_evaluator())
                        loaded += 1
                except Exception:
                    pass
        return loaded
