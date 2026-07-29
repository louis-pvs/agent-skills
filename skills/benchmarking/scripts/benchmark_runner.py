"""Benchmarking execution runner and CLI entrypoint.

Executes target commands under controlled conditions, collects metric telemetry,
compares against baselines, and generates structured JSON benchmark reports.
"""

import argparse
import json
import os
import shlex
import subprocess
import sys
import time
import tracemalloc
from pathlib import Path
from typing import Any, Dict, List, Optional

SCRIPT_DIR = Path(__file__).parent.resolve()
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

try:
    from .evaluators.base import BenchmarkContext, MetricStatus
    from .evaluators.registry import MetricRegistry
except ImportError:
    from evaluators.base import BenchmarkContext, MetricStatus
    from evaluators.registry import MetricRegistry


def execute_single_run(cmd: str, cwd: Optional[str] = None) -> BenchmarkContext:
    """Execute command once while measuring wall-clock time and peak memory."""
    work_dir = cwd or os.getcwd()
    raw_tokens = shlex.split(cmd) if isinstance(cmd, str) else list(cmd)
    if not raw_tokens:
        raise ValueError("Command cannot be empty.")
    tokens = [str(tok) for tok in raw_tokens]

    tracemalloc.start()
    start_time = time.perf_counter()

    proc = subprocess.run(
        tokens,
        shell=False,
        cwd=work_dir,
        capture_output=True,
        text=True,
    )

    end_time = time.perf_counter()
    _, peak_bytes = tracemalloc.get_traced_memory()
    tracemalloc.stop()

    wall_time_ms = (end_time - start_time) * 1000.0
    peak_memory_mb = peak_bytes / (1024.0 * 1024.0)

    return BenchmarkContext(
        command=cmd,
        cwd=work_dir,
        stdout=proc.stdout,
        stderr=proc.stderr,
        exit_code=proc.returncode,
        wall_time_ms=wall_time_ms,
        peak_memory_mb=peak_memory_mb,
    )


def run_benchmark(
    cmd: str,
    baseline_cmd: Optional[str] = None,
    iterations: int = 5,
    warmup: int = 1,
    metrics: Optional[List[str]] = None,
    metric_dir: Optional[str] = None,
    assert_max_duration_ms: float = 0.0,
    assert_min_pass_ratio: float = 1.0,
    cwd: Optional[str] = None,
) -> Dict[str, Any]:
    """Run benchmark iterations, compute metrics, and return report."""
    registry = MetricRegistry()
    registry.register_defaults()

    if metric_dir:
        registry.load_from_directory(Path(metric_dir))

    # Configure built-in assertion thresholds
    if assert_max_duration_ms > 0:
        timing_eval = registry.get("timing")
        if timing_eval and hasattr(timing_eval, "configure"):
            timing_eval.configure({"max_duration_ms": assert_max_duration_ms})

    if assert_min_pass_ratio < 1.0 or assert_min_pass_ratio > 0.0:
        pass_eval = registry.get("pass_ratio")
        if pass_eval and hasattr(pass_eval, "configure"):
            pass_eval.configure({"min_pass_ratio": assert_min_pass_ratio})

    # Warmup runs
    for _ in range(max(0, warmup)):
        execute_single_run(cmd, cwd=cwd)

    # Benchmark measurement iterations
    contexts: List[BenchmarkContext] = []
    for _ in range(max(1, iterations)):
        ctx = execute_single_run(cmd, cwd=cwd)
        contexts.append(ctx)

    # Compute aggregate telemetry
    avg_wall_ms = sum(c.wall_time_ms for c in contexts) / len(contexts)
    avg_mem_mb = sum(c.peak_memory_mb for c in contexts) / len(contexts)
    last_ctx = contexts[-1]

    aggregate_ctx = BenchmarkContext(
        command=cmd,
        cwd=last_ctx.cwd,
        stdout=last_ctx.stdout,
        stderr=last_ctx.stderr,
        exit_code=last_ctx.exit_code,
        wall_time_ms=avg_wall_ms,
        peak_memory_mb=avg_mem_mb,
    )

    # Run requested metric evaluators
    active_metric_names = metrics or ["timing", "memory", "pass_ratio"]
    eval_results = []
    all_passed = True

    for name in active_metric_names:
        evaluator = registry.get(name)
        if evaluator:
            res = evaluator.evaluate(aggregate_ctx)
            eval_results.append(res.to_dict())
            if res.status == MetricStatus.FAIL:
                all_passed = False
        else:
            eval_results.append(
                {
                    "name": name,
                    "status": MetricStatus.SKIPPED.value,
                    "detail": f"Evaluator '{name}' not found in registry.",
                }
            )

    report: Dict[str, Any] = {
        "command": cmd,
        "iterations": len(contexts),
        "summary": {
            "status": "pass" if all_passed else "fail",
            "avg_wall_time_ms": round(avg_wall_ms, 2),
            "avg_peak_memory_mb": round(avg_mem_mb, 2),
            "exit_code": last_ctx.exit_code,
        },
        "metrics": eval_results,
    }

    # Baseline differential run if requested
    if baseline_cmd:
        baseline_ctxs = []
        for _ in range(max(1, iterations)):
            baseline_ctxs.append(execute_single_run(baseline_cmd, cwd=cwd))

        b_avg_wall_ms = sum(c.wall_time_ms for c in baseline_ctxs) / len(baseline_ctxs)
        b_avg_mem_mb = sum(c.peak_memory_mb for c in baseline_ctxs) / len(baseline_ctxs)

        wall_delta_pct = ((avg_wall_ms - b_avg_wall_ms) / b_avg_wall_ms) * 100.0 if b_avg_wall_ms > 0 else 0.0

        report["baseline"] = {
            "command": baseline_cmd,
            "avg_wall_time_ms": round(b_avg_wall_ms, 2),
            "avg_peak_memory_mb": round(b_avg_mem_mb, 2),
            "wall_time_delta_pct": round(wall_delta_pct, 2),
        }

    return report


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="Benchmarking Skill Runner")
    parser.add_argument("--cmd", type=str, required=True, help="Target command to benchmark")
    parser.add_argument(
        "--baseline-cmd",
        type=str,
        default=None,
        help="Baseline command for diff comparison",
    )
    parser.add_argument(
        "--iterations",
        type=int,
        default=5,
        help="Number of benchmark iterations",
    )
    parser.add_argument(
        "--warmup",
        type=int,
        default=1,
        help="Warmup iterations before recording",
    )
    parser.add_argument(
        "--metrics",
        type=str,
        default="timing,memory,pass_ratio",
        help="CSV list of metrics to evaluate",
    )
    parser.add_argument(
        "--metric-dir",
        type=str,
        default=None,
        help="Directory containing custom metric plugins",
    )
    parser.add_argument(
        "--assert-max-duration-ms",
        type=float,
        default=0.0,
        help="Max duration assertion in ms",
    )
    parser.add_argument(
        "--assert-min-pass-ratio",
        type=float,
        default=1.0,
        help="Min pass ratio assertion (0.0 to 1.0)",
    )
    parser.add_argument("--json", action="store_true", help="Output raw JSON report")

    args = parser.parse_args(argv)
    metric_list = [m.strip() for m in args.metrics.split(",") if m.strip()]

    report = run_benchmark(
        cmd=args.cmd,
        baseline_cmd=args.baseline_cmd,
        iterations=args.iterations,
        warmup=args.warmup,
        metrics=metric_list,
        metric_dir=args.metric_dir,
        assert_max_duration_ms=args.assert_max_duration_ms,
        assert_min_pass_ratio=args.assert_min_pass_ratio,
    )

    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(f"📊 Benchmark Report: {report['command']}")
        print(f"Status: {report['summary']['status'].upper()}")
        print(f"Avg Wall Time: {report['summary']['avg_wall_time_ms']} ms")
        print(f"Avg Peak Memory: {report['summary']['avg_peak_memory_mb']} MB")
        for m in report.get("metrics", []):
            print(f"  - [{m['status'].upper()}] {m['name']}: {m.get('value')} {m.get('unit', '')} ({m.get('detail', '')})")

    return 0 if report["summary"]["status"] == "pass" else 1


if __name__ == "__main__":
    sys.exit(main())
