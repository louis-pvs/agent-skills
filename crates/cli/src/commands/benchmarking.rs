use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
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
pub struct BenchmarkReport {
    pub cmd: String,
    pub iterations: usize,
    pub pass_count: usize,
    pub pass_ratio: f64,
    pub avg_duration_ms: u64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
}

pub fn run_benchmark_iterations(
    cmd_str: &str,
    iterations: usize,
    repo_root: &Path,
) -> Result<BenchmarkReport, String> {
    let parts =
        shlex::split(cmd_str).ok_or_else(|| "Failed to parse command string".to_string())?;
    if parts.is_empty() {
        return Err("Command string is empty".to_string());
    }

    let program = &parts[0];
    let args = &parts[1..];

    let mut durations = Vec::new();
    let mut pass_count = 0;

    for _ in 0..iterations {
        let start = Instant::now();
        let status = Command::new(program)
            .args(args)
            .current_dir(repo_root)
            .status();

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

    Ok(BenchmarkReport {
        cmd: cmd_str.to_string(),
        iterations,
        pass_count,
        pass_ratio,
        avg_duration_ms,
        min_duration_ms,
        max_duration_ms,
    })
}

pub fn check_benchmarking_health(skill_dir: &Path) -> Result<Vec<String>, String> {
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
        Err(format!(
            "Benchmarking health check failed. Missing files: {:?}",
            missing
        ))
    }
}

pub fn run_benchmarking_command(
    subcommand: &BenchmarkingSubcommand,
    repo_root: &Path,
) -> Result<(), String> {
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
            let report = run_benchmark_iterations(&args.cmd, args.iterations, repo_root)?;

            if let Some(limit) = args.assert_max_duration_ms {
                if report.avg_duration_ms > limit {
                    return Err(format!(
                        "Benchmark threshold failure: avg duration {} ms exceeds limit {} ms",
                        report.avg_duration_ms, limit
                    ));
                }
            }

            if let Some(limit) = args.assert_min_pass_ratio {
                if report.pass_ratio < limit {
                    return Err(format!(
                        "Benchmark threshold failure: pass ratio {} is below limit {}",
                        report.pass_ratio, limit
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

        let report =
            run_benchmark_iterations("python3 -c \"import time; time.sleep(0.01)\"", 2, &base)
                .unwrap();
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
}
