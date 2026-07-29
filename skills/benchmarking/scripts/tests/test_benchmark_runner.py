#!/usr/bin/env python3
"""Unit tests for the Benchmarking Skill runner and expandable metric framework."""

import sys
import tempfile
import unittest
from pathlib import Path

TESTS_DIR = Path(__file__).parent.resolve()
SCRIPTS_DIR = TESTS_DIR.parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from benchmark_runner import main, run_benchmark  # noqa: E402
from evaluators.base import (  # noqa: E402
    BenchmarkContext,
    MetricResult,
    MetricStatus,
)
from evaluators.builtins import (  # noqa: E402
    MemoryEvaluator,
    PassRatioEvaluator,
    TimingEvaluator,
)
from evaluators.registry import MetricRegistry  # noqa: E402


class CustomTestEvaluator:
    """A custom metric evaluator for testing dynamic plugin loading."""

    name = "custom_score"
    requires = ()

    def configure(self, config):
        self.target = config.get("target", 100)

    def evaluate(self, context: BenchmarkContext) -> MetricResult:
        value = 95
        status = MetricStatus.PASS if value >= getattr(self, "target", 100) else MetricStatus.FAIL
        return MetricResult(
            name=self.name,
            status=status,
            value=value,
            unit="pts",
            threshold=getattr(self, "target", 100),
            detail=f"Score {value} / {getattr(self, 'target', 100)}",
        )


class TestBenchmarkRunner(unittest.TestCase):
    def test_builtin_evaluators(self):
        ctx = BenchmarkContext(
            command="echo 'hello'",
            cwd=".",
            stdout="hello\n",
            stderr="",
            exit_code=0,
            wall_time_ms=25.5,
            peak_memory_mb=1.2,
        )

        timing_res = TimingEvaluator().evaluate(ctx)
        self.assertEqual(timing_res.name, "timing")
        self.assertEqual(timing_res.status, MetricStatus.PASS)
        self.assertEqual(timing_res.value, 25.5)
        self.assertEqual(timing_res.unit, "ms")

        memory_res = MemoryEvaluator().evaluate(ctx)
        self.assertEqual(memory_res.name, "memory")
        self.assertEqual(memory_res.status, MetricStatus.PASS)
        self.assertEqual(memory_res.value, 1.2)
        self.assertEqual(memory_res.unit, "MB")

        pass_res = PassRatioEvaluator().evaluate(ctx)
        self.assertEqual(pass_res.name, "pass_ratio")
        self.assertEqual(pass_res.status, MetricStatus.PASS)
        self.assertEqual(pass_res.value, 1.0)

    def test_metric_registry(self):
        registry = MetricRegistry()
        registry.register_defaults()

        self.assertIsNotNone(registry.get("timing"))
        self.assertIsNotNone(registry.get("memory"))
        self.assertIsNotNone(registry.get("pass_ratio"))

        custom = CustomTestEvaluator()
        registry.register(custom)
        self.assertEqual(registry.get("custom_score"), custom)

    def test_dynamic_plugin_loading(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            plugin_path = Path(tmpdir) / "sample_plugin.py"
            plugin_path.write_text(
                "from evaluators.base import MetricResult, MetricStatus\n"
                "class DummyPlugin:\n"
                "    name = 'dummy_metric'\n"
                "    def configure(self, config): pass\n"
                "    def evaluate(self, context):\n"
                "        return MetricResult(name='dummy_metric', status=MetricStatus.PASS, value=42)\n"
                "METRIC = DummyPlugin()\n"
            )

            registry = MetricRegistry()
            registry.load_from_directory(Path(tmpdir))

            evaluator = registry.get("dummy_metric")
            self.assertIsNotNone(evaluator)

            ctx = BenchmarkContext(
                command="true",
                cwd=".",
                stdout="",
                stderr="",
                exit_code=0,
                wall_time_ms=5.0,
                peak_memory_mb=0.5,
            )
            result = evaluator.evaluate(ctx)
            self.assertEqual(result.name, "dummy_metric")
            self.assertEqual(result.value, 42)

    def test_run_benchmark(self):
        report = run_benchmark(
            cmd="echo 'test benchmark'",
            iterations=3,
            metrics=["timing", "memory", "pass_ratio"],
        )
        self.assertEqual(report["command"], "echo 'test benchmark'")
        self.assertEqual(report["iterations"], 3)
        self.assertEqual(report["summary"]["status"], "pass")
        self.assertIn("metrics", report)

    def test_run_benchmark_baseline_comparison(self):
        report = run_benchmark(
            cmd="echo 'target'",
            baseline_cmd="echo 'baseline'",
            iterations=2,
        )
        self.assertIn("baseline", report)
        self.assertEqual(report["baseline"]["command"], "echo 'baseline'")

    def test_cli_main_success(self):
        exit_code = main(["--cmd", "echo 'cli test'", "--iterations", "2", "--json"])
        self.assertEqual(exit_code, 0)

    def test_cli_main_threshold_failure(self):
        exit_code = main(["--cmd", "echo 'fail test'", "--assert-max-duration-ms", "0.000001"])
        self.assertEqual(exit_code, 1)


if __name__ == "__main__":
    unittest.main()
