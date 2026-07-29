# Empirical Benchmarking Overview

The Benchmarking skill provides empirical verification, baseline differential analysis, and expandable metric evaluations to prevent quality regressions and performance hallucinations.

---

## The Problem & Friction

Optimizing software without empirical measurement leads to speculative refactoring. Developers and AI agents often make architectural changes assuming they improve latency or memory usage, only to introduce hidden bottlenecks.

Benchmarking grounds code changes in empirical data by capturing baseline execution metrics and measuring differential impact after code modifications.

---

## Benchmark Execution Pipeline

```mermaid
sequenceDiagram
    participant Agent as Agent / Engineer
    participant Script as Benchmark Harness
    participant Code as Target Codebase
    participant Report as Differential Report

    Agent->>Script: Run Baseline Benchmark
    Script->>Code: Measure Latency & Resource Footprint
    Code-->>Script: Baseline Metrics
    Script-->>Report: Store Baseline Snapshot

    Note over Agent,Code: Code Changes / Refactoring Applied

    Agent->>Script: Run Differential Benchmark
    Script->>Code: Measure Post-Patch Execution
    Code-->>Script: Post-Patch Metrics
    Script-->>Report: Calculate Variance (Δ %)
    Report-->>Agent: Output Differential Analysis
```

---

## Metric Evaluation Invariants

> [!NOTE]
> Benchmarks must run under isolated conditions to minimize OS context-switch noise and variance.

> [!IMPORTANT]
> A performance claim (e.g., "10x faster") MUST be supported by repeatable timing samples and percentage variance output.

---

## Key Metrics Evaluated

- **Wall-clock Execution Time**: Mean, median, and p95 latency.
- **Memory Allocation Footprint**: Peak resident set size (RSS) and allocation rate.
- **Throughput / Assertion Ratios**: Executed ops per second or test assertions per second.
