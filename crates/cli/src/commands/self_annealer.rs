use agent_skills_core::path_safety::sanitize_path;
use agent_skills_core::process_exec::run_with_windows_fallback_output;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Args, Debug, Clone)]
pub struct CheckSelfAnnealerArgs {
    /// Target path of the self-annealer skill folder
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct RunAnnealArgs {
    /// Test or verification command to run
    #[arg(long, default_value = "cargo test")]
    pub cmd: String,

    /// Maximum repair iteration limit (default: 3)
    #[arg(long, default_value_t = 3)]
    pub max_iterations: usize,

    /// Automatically rollback uncommitted changes on failure
    #[arg(long, default_value_t = true)]
    pub auto_rollback: bool,

    /// Output findings in JSON format
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SelfAnnealerSubcommand {
    /// Verify health and structural completeness of self-annealer files
    Check(CheckSelfAnnealerArgs),
    /// Run bounded self-healing repair loop
    Run(RunAnnealArgs),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnnealReport {
    pub cmd: String,
    pub max_iterations: usize,
    pub attempts: usize,
    pub converged: bool,
    pub rolled_back: bool,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_anneal_loop(
    cmd_str: &str,
    max_iterations: usize,
    auto_rollback: bool,
    repo_root: &Path,
) -> anyhow::Result<AnnealReport> {
    if cmd_str.contains('\0') {
        return Err(anyhow::anyhow!("Command string contains null bytes"));
    }

    let parts =
        shlex::split(cmd_str).ok_or_else(|| anyhow::anyhow!("Failed to parse command string"))?;
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Command string is empty"));
    }

    let mut attempts = 0;
    let mut converged = false;
    let mut last_stdout = String::new();
    let mut last_stderr = String::new();

    while attempts < max_iterations {
        attempts += 1;
        let output = run_with_windows_fallback_output(&parts, repo_root);

        if let Ok(out) = output {
            last_stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            last_stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            if out.status.success() {
                converged = true;
                break;
            }
        }
    }

    let mut rolled_back = false;
    if !converged && auto_rollback {
        let _ = Command::new("git")
            .args(["checkout", "--", "."])
            .current_dir(repo_root)
            .status();
        rolled_back = true;
    }

    Ok(AnnealReport {
        cmd: cmd_str.to_string(),
        max_iterations,
        attempts,
        converged,
        rolled_back,
        stdout: last_stdout,
        stderr: last_stderr,
    })
}

pub fn check_self_annealer_health(skill_dir: &Path) -> anyhow::Result<Vec<String>> {
    let required_files = [
        skill_dir.join("SKILL.md"),
        skill_dir.join("README.md"),
        skill_dir.join("references").join("overview.md"),
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
            "Self Annealer health check failed. Missing files: {:?}",
            missing
        ))
    }
}

pub fn run_self_annealer_command(
    subcommand: &SelfAnnealerSubcommand,
    repo_root: &Path,
) -> anyhow::Result<()> {
    match subcommand {
        SelfAnnealerSubcommand::Check(args) => {
            let skill_dir = if let Some(p) = &args.path {
                sanitize_path(p, Some(repo_root))?
            } else {
                repo_root.join("skills").join("self-annealer")
            };

            match check_self_annealer_health(&skill_dir) {
                Ok(_) => {
                    println!("Self Annealer skill health check passed cleanly.");
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        SelfAnnealerSubcommand::Run(args) => {
            let report = run_anneal_loop(
                &args.cmd,
                args.max_iterations,
                args.auto_rollback,
                repo_root,
            )?;

            if args.json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!("Self-Annealing Loop Execution Report for: '{}'", report.cmd);
                println!("  Max Iterations Cap: {}", report.max_iterations);
                println!("  Attempts Executed: {}", report.attempts);
                println!("  Converged (GREEN): {}", report.converged);
                println!("  Git Rollback Executed: {}", report.rolled_back);
            }

            if report.converged {
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Self-annealing repair loop failed to converge after {} iterations.",
                    report.attempts
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_run_anneal_loop_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        let py_cmd = if cfg!(windows) {
            "python -c \"import sys; sys.exit(0)\""
        } else {
            "python3 -c \"import sys; sys.exit(0)\""
        };
        let report = run_anneal_loop(py_cmd, 3, false, &base).unwrap();
        assert_eq!(report.cmd, py_cmd);
        assert_eq!(report.max_iterations, 3);
        assert_eq!(report.attempts, 1);
        assert!(report.converged);
        assert!(!report.rolled_back);
    }

    #[test]
    fn test_check_self_annealer_health_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let references = base.join("references");
        fs::create_dir_all(&references).unwrap();

        fs::write(base.join("SKILL.md"), "---").unwrap();
        fs::write(base.join("README.md"), "# Title").unwrap();
        fs::write(references.join("overview.md"), "# Overview").unwrap();

        let res = check_self_annealer_health(&base);
        assert!(res.is_ok());
    }

    #[test]
    fn test_run_cmd_empty_command() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let res = run_anneal_loop("", 3, false, &base);
        assert!(res.is_err());
    }

    #[test]
    fn test_run_cmd_null_bytes() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let res = run_anneal_loop("echo\0test", 3, false, &base);
        assert!(res.is_err());
    }

    #[test]
    fn test_run_cmd_success() {
        // Python: test_run_cmd_success — Rust uses .status() not .output(), so stdout is never captured
        // Real regression: AnnealReport has no stdout field
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let report = run_anneal_loop("echo UNIQUE_STDOUT_TOKEN", 3, false, &base).unwrap();

        assert!(report.converged, "echo must succeed and converge");
        // The AnnealReport must capture stdout to verify the command actually ran
        // This test fails because AnnealReport has no stdout/output field
        let report_json = serde_json::to_string(&report).unwrap();
        assert!(
            report_json.contains(r#""stdout":"UNIQUE_STDOUT_TOKEN"#),
            "AnnealReport must have a 'stdout' field capturing subprocess output. Got: {report_json}"
        );
    }

    #[test]
    fn test_run_cmd_invalid_quotes() {
        // Python: test_run_cmd_invalid_quotes — malformed shell quoting must fail
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let result = run_anneal_loop("echo 'unclosed quote", 1, false, &base);
        assert!(result.is_err(), "unclosed quote must cause a parse error");
    }

    #[test]
    fn test_parse_args_defaults() {
        // Python: test_parse_args_defaults — max_iterations=3, auto_rollback=true by default
        use clap::Parser;
        #[derive(Parser)]
        struct Wrapper {
            #[command(flatten)]
            args: RunAnnealArgs,
        }
        let parsed = Wrapper::parse_from(["self-annealer-run"]);
        assert_eq!(parsed.args.max_iterations, 3);
        assert!(parsed.args.auto_rollback);
    }
}
