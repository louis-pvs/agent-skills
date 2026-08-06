use agent_skills_core::path_safety::sanitize_path;
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
}

pub fn run_anneal_loop(
    cmd_str: &str,
    max_iterations: usize,
    auto_rollback: bool,
    repo_root: &Path,
) -> Result<AnnealReport, String> {
    let parts =
        shlex::split(cmd_str).ok_or_else(|| "Failed to parse command string".to_string())?;
    if parts.is_empty() {
        return Err("Command string is empty".to_string());
    }

    let program = &parts[0];
    let args = &parts[1..];

    let mut attempts = 0;
    let mut converged = false;

    while attempts < max_iterations {
        attempts += 1;
        let status = Command::new(program)
            .args(args)
            .current_dir(repo_root)
            .status();

        if let Ok(st) = status {
            if st.success() {
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
    })
}

pub fn check_self_annealer_health(skill_dir: &Path) -> Result<Vec<String>, String> {
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
        Err(format!(
            "Self Annealer health check failed. Missing files: {:?}",
            missing
        ))
    }
}

pub fn run_self_annealer_command(
    subcommand: &SelfAnnealerSubcommand,
    repo_root: &Path,
) -> Result<(), String> {
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
                Err(format!(
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

        let report =
            run_anneal_loop("python3 -c \"import sys; sys.exit(0)\"", 3, false, &base).unwrap();
        assert_eq!(report.cmd, "python3 -c \"import sys; sys.exit(0)\"");
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
}
