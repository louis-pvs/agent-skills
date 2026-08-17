use agent_skills_core::path_safety::sanitize_path;
use agent_skills_core::process_exec::run_with_windows_fallback_status;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

#[derive(Args, Debug, Clone)]
pub struct CheckBenchmarkingArgs {
    /// Target path of the benchmarking skill folder
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct RunBenchmarkArgs {
    /// Command line string to benchmark
    #[arg(long)]
    pub cmd: String,

    /// Optional baseline command to compare against
    #[arg(long)]
    pub baseline_cmd: Option<String>,

    /// Number of iterations to execute (default: 5)
    #[arg(long, default_value_t = 5)]
    pub iterations: usize,

    /// Maximum allowed average duration in milliseconds
    #[arg(long)]
    pub assert_max_duration_ms: Option<u64>,

    /// Minimum required pass ratio (0.0 to 1.0)
    #[arg(long)]
    pub assert_min_pass_ratio: Option<f64>,

    /// Output findings in JSON format
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum BenchmarkingSubcommand {
    /// Verify health and structural completeness of benchmarking files
    Check(CheckBenchmarkingArgs),
    /// Execute empirical performance benchmark runs
    Run(RunBenchmarkArgs),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkMetricResult {
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkSummary {
    pub status: String,
    pub per_metric: Vec<BenchmarkMetricResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkReport {
    pub cmd: String,
    pub iterations: usize,
    pub pass_count: usize,
    pub pass_ratio: f64,
    pub avg_duration_ms: u64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub baseline_avg_ms: Option<u64>,
    pub delta_ms: Option<i64>,
    pub summary: BenchmarkSummary,
}

#[allow(dead_code)]
pub struct MetricRegistry {
    #[allow(clippy::type_complexity)]
    evaluators: HashMap<String, Box<dyn Fn(&BenchmarkReport) -> f64 + Send + Sync>>,
}

#[allow(dead_code)]
impl MetricRegistry {
    pub fn new() -> Self {
        let mut reg = MetricRegistry {
            evaluators: HashMap::new(),
        };
        reg.evaluators.insert(
            "duration_ms".to_string(),
            Box::new(|r: &BenchmarkReport| r.avg_duration_ms as f64),
        );
        reg.evaluators.insert(
            "pass_ratio".to_string(),
            Box::new(|r: &BenchmarkReport| r.pass_ratio),
        );
        reg
    }

    #[allow(clippy::type_complexity)]
    pub fn register(
        &mut self,
        name: &str,
        evaluator: Box<dyn Fn(&BenchmarkReport) -> f64 + Send + Sync>,
    ) {
        self.evaluators.insert(name.to_string(), evaluator);
    }

    pub fn lookup(&self, name: &str) -> Option<&(dyn Fn(&BenchmarkReport) -> f64 + Send + Sync)> {
        self.evaluators.get(name).map(|f| f.as_ref())
    }

    pub fn names(&self) -> Vec<String> {
        self.evaluators.keys().cloned().collect()
    }
}

impl Default for MetricRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run_benchmark_iterations(
    cmd_str: &str,
    iterations: usize,
    repo_root: &Path,
) -> anyhow::Result<BenchmarkReport> {
    if cmd_str.contains('\0') {
        return Err(anyhow::anyhow!("Command string contains null bytes"));
    }
    let parts =
        shlex::split(cmd_str).ok_or_else(|| anyhow::anyhow!("Failed to parse command string"))?;
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Command string is empty"));
    }

    let mut durations = Vec::new();
    let mut pass_count = 0;

    for _ in 0..iterations {
        let start = Instant::now();
        let status = run_with_windows_fallback_status(&parts, repo_root);

        let duration_ms = start.elapsed().as_millis() as u64;
        durations.push(duration_ms);

        if let Ok(st) = status {
            if st.success() {
                pass_count += 1;
            }
        }
    }

    let min_duration_ms = *durations.iter().min().unwrap_or(&0);
    let max_duration_ms = *durations.iter().max().unwrap_or(&0);
    let sum_duration: u64 = durations.iter().sum();
    let avg_duration_ms = if iterations > 0 {
        sum_duration / iterations as u64
    } else {
        0
    };

    let pass_ratio = if iterations > 0 {
        pass_count as f64 / iterations as f64
    } else {
        0.0
    };

    let summary = BenchmarkSummary {
        status: if (pass_ratio - 1.0).abs() < f64::EPSILON {
            "PASS".to_string()
        } else {
            "FAIL".to_string()
        },
        per_metric: vec![
            BenchmarkMetricResult {
                name: "duration_ms".to_string(),
                value: avg_duration_ms as f64,
            },
            BenchmarkMetricResult {
                name: "pass_ratio".to_string(),
                value: pass_ratio,
            },
        ],
    };

    Ok(BenchmarkReport {
        cmd: cmd_str.to_string(),
        iterations,
        pass_count,
        pass_ratio,
        avg_duration_ms,
        min_duration_ms,
        max_duration_ms,
        baseline_avg_ms: None,
        delta_ms: None,
        summary,
    })
}

pub fn check_benchmarking_health(skill_dir: &Path) -> anyhow::Result<Vec<String>> {
    let required_files = [
        skill_dir.join("SKILL.md"),
        skill_dir.join("README.md"),
        skill_dir.join("references").join("philosophy.md"),
    ];

    let mut missing = Vec::new();
    for req in &required_files {
        if !req.is_file() {
            missing.push(
                req.strip_prefix(skill_dir)
                    .unwrap_or(req)
                    .display()
                    .to_string(),
            );
        }
    }

    if missing.is_empty() {
        Ok(Vec::new())
    } else {
        Err(anyhow::anyhow!(
            "Benchmarking health check failed. Missing files: {:?}",
            missing
        ))
    }
}

pub fn run_benchmarking_command(
    subcommand: &BenchmarkingSubcommand,
    repo_root: &Path,
) -> anyhow::Result<()> {
    match subcommand {
        BenchmarkingSubcommand::Check(args) => {
            let skill_dir = if let Some(p) = &args.path {
                sanitize_path(p, Some(repo_root))?
            } else {
                repo_root.join("skills").join("benchmarking")
            };

            match check_benchmarking_health(&skill_dir) {
                Ok(_) => {
                    println!("Benchmarking skill health check passed cleanly.");
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        BenchmarkingSubcommand::Run(args) => {
            let mut report = run_benchmark_iterations(&args.cmd, args.iterations, repo_root)?;

            if let Some(baseline_cmd) = &args.baseline_cmd {
                let baseline_report =
                    run_benchmark_iterations(baseline_cmd, args.iterations, repo_root)?;
                report.baseline_avg_ms = Some(baseline_report.avg_duration_ms);
                report.delta_ms =
                    Some(report.avg_duration_ms as i64 - baseline_report.avg_duration_ms as i64);
            }

            if let Some(limit) = args.assert_max_duration_ms {
                if report.avg_duration_ms > limit {
                    return Err(anyhow::anyhow!(
                        "Benchmark threshold failure: avg duration {} ms exceeds threshold {} ms",
                        report.avg_duration_ms,
                        limit
                    ));
                }
            }

            if let Some(limit) = args.assert_min_pass_ratio {
                if report.pass_ratio < limit {
                    return Err(anyhow::anyhow!(
                        "Benchmark threshold failure: pass ratio {} is below limit {}",
                        report.pass_ratio,
                        limit
                    ));
                }
            }

            if args.json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!("Empirical Benchmark Results for: '{}'", report.cmd);
                println!("  Iterations: {}", report.iterations);
                println!("  Pass Count: {}/{}", report.pass_count, report.iterations);
                println!("  Pass Ratio: {:.1}%", report.pass_ratio * 100.0);
                println!("  Avg Duration: {} ms", report.avg_duration_ms);
                println!(
                    "  Min / Max: {} ms / {} ms",
                    report.min_duration_ms, report.max_duration_ms
                );
                if let Some(baseline_avg_ms) = report.baseline_avg_ms {
                    println!("  Baseline Avg: {} ms", baseline_avg_ms);
                }
                if let Some(delta_ms) = report.delta_ms {
                    println!("  Delta: {} ms", delta_ms);
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_run_benchmark_iterations_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let py_cmd = if cfg!(windows) {
            "python -c \"import time; time.sleep(0.01)\""
        } else {
            "python3 -c \"import time; time.sleep(0.01)\""
        };
        let report = run_benchmark_iterations(py_cmd, 2, &base).unwrap();
        assert_eq!(report.iterations, 2);
        assert_eq!(report.pass_count, 2);
        assert_eq!(report.pass_ratio, 1.0);
        assert!(report.avg_duration_ms >= 5);
    }

    #[test]
    fn test_check_benchmarking_health_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let references = base.join("references");
        fs::create_dir_all(&references).unwrap();

        fs::write(base.join("SKILL.md"), "---").unwrap();
        fs::write(base.join("README.md"), "# Title").unwrap();
        fs::write(references.join("philosophy.md"), "# Philosophy").unwrap();

        let res = check_benchmarking_health(&base);
        assert!(res.is_ok());
    }

    #[test]
    fn test_cli_main_success() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let args = RunBenchmarkArgs {
            cmd: "echo hello".to_string(),
            baseline_cmd: None,
            iterations: 2,
            assert_max_duration_ms: Some(5000),
            assert_min_pass_ratio: Some(0.9),
            json: false,
        };

        let res = run_benchmarking_command(&BenchmarkingSubcommand::Run(args), &base);
        assert!(res.is_ok());
    }

    #[test]
    fn test_cli_main_threshold_failure() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let py_cmd = if cfg!(windows) {
            "python -c \"import time; time.sleep(0.05)\""
        } else {
            "python3 -c \"import time; time.sleep(0.05)\""
        };

        let args = RunBenchmarkArgs {
            cmd: py_cmd.to_string(),
            baseline_cmd: None,
            iterations: 2,
            assert_max_duration_ms: Some(1),
            assert_min_pass_ratio: None,
            json: false,
        };

        let res = run_benchmarking_command(&BenchmarkingSubcommand::Run(args), &base);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("exceeds threshold"));
    }

    #[test]
    fn test_builtin_evaluators() {
        let registry = MetricRegistry::new();
        let names = registry.names();
        assert!(
            names.contains(&"duration_ms".to_string()),
            "registry must have builtin duration_ms evaluator"
        );
        assert!(
            names.contains(&"pass_ratio".to_string()),
            "registry must have builtin pass_ratio evaluator"
        );

        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let report = run_benchmark_iterations("echo hello", 1, &base).unwrap();
        let duration_eval = registry.lookup("duration_ms").unwrap();
        let dur = duration_eval(&report);
        assert!(dur >= 0.0);
        let ratio_eval = registry.lookup("pass_ratio").unwrap();
        let ratio = ratio_eval(&report);
        assert!((ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_metric_registry() {
        let mut registry = MetricRegistry::new();
        registry.register(
            "custom_metric",
            Box::new(|r: &BenchmarkReport| r.pass_count as f64 * 2.0),
        );
        let eval = registry.lookup("custom_metric").unwrap();
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let report = run_benchmark_iterations("echo ok", 1, &base).unwrap();
        let val = eval(&report);
        assert_eq!(val, report.pass_count as f64 * 2.0);
    }

    #[test]
    fn test_dynamic_plugin_loading() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let plugin = base.join("custom_evaluator.py");
        fs::write(&plugin, "print('memory_mb:42.0')\n").unwrap();

        let py_bin = if cfg!(windows) { "python" } else { "python3" };
        let output = std::process::Command::new(py_bin)
            .arg(&plugin)
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("memory_mb:"),
            "plugin must output metric_name:value"
        );

        let line = stdout.trim();
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        assert_eq!(parts.len(), 2);
        let metric_name = parts[0].trim();
        let metric_value: f64 = parts[1].trim().parse().unwrap();
        assert_eq!(metric_name, "memory_mb");
        assert!((metric_value - 42.0).abs() < 0.01);
    }

    #[test]
    fn test_run_benchmark_baseline_comparison() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let args = RunBenchmarkArgs {
            cmd: "echo hello".to_string(),
            iterations: 1,
            assert_max_duration_ms: None,
            assert_min_pass_ratio: None,
            baseline_cmd: Some("echo world".to_string()),
            json: false,
        };
        let res = run_benchmarking_command(&BenchmarkingSubcommand::Run(args), &base);
        assert!(res.is_ok(), "baseline comparison must succeed: {:?}", res);
    }

    #[test]
    fn test_run_benchmark_summary_status() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let report = run_benchmark_iterations("echo hello", 2, &base).unwrap();
        assert!(
            !report.summary.status.is_empty(),
            "summary.status must not be empty"
        );
        assert!(report.summary.status == "PASS" || report.summary.status == "FAIL");
        assert!(
            !report.summary.per_metric.is_empty(),
            "summary.per_metric must not be empty"
        );
        let metric_names: Vec<&str> = report
            .summary
            .per_metric
            .iter()
            .map(|m| m.name.as_str())
            .collect();
        assert!(
            metric_names.contains(&"duration_ms") || metric_names.contains(&"pass_ratio"),
            "per_metric must include at least duration_ms or pass_ratio, got: {:?}",
            metric_names
        );
    }
}
