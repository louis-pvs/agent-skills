use agent_skills_core::path_safety::sanitize_path;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args, Debug, Clone)]
pub struct CheckWhatIfArgs {
    /// Target path of the what-if-analysis skill folder
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct ImpactAnalysisArgs {
    /// Target symbol name to calculate blast radius for
    #[arg(long)]
    pub symbol: String,

    /// Output findings in JSON format
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum WhatIfAnalysisSubcommand {
    /// Verify health and structural completeness of what-if-analysis files
    Check(CheckWhatIfArgs),
    /// Calculate symbol blast radius and prospective risk
    Impact(ImpactAnalysisArgs),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlastRadiusReport {
    pub symbol: String,
    pub risk_level: String,
    pub caller_count: usize,
    pub test_files: Vec<PathBuf>,
}

pub fn analyze_symbol_blast_radius(
    symbol: &str,
    workspace_dir: &Path,
) -> Result<BlastRadiusReport, String> {
    let mut caller_count = 0;
    let mut test_files = Vec::new();

    let mut walk = vec![workspace_dir.to_path_buf()];
    while let Some(curr) = walk.pop() {
        if let Ok(entries) = fs::read_dir(curr) {
            for entry in entries.flatten() {
                let p = entry.path();
                let fname = p.file_name().unwrap_or_default().to_string_lossy();
                if fname.starts_with('.') || fname == "target" || fname == "node_modules" {
                    continue;
                }
                if p.is_dir() {
                    walk.push(p.clone());
                } else if p.is_file() {
                    if let Ok(content) = fs::read_to_string(&p) {
                        if content.contains(symbol) {
                            caller_count += 1;
                            let fname_lower = fname.to_lowercase();
                            if fname_lower.contains("test") || fname_lower.contains("spec") {
                                test_files.push(p);
                            }
                        }
                    }
                }
            }
        }
    }

    let risk_level = if caller_count > 10 {
        "HIGH".to_string()
    } else if caller_count > 3 {
        "MEDIUM".to_string()
    } else {
        "LOW".to_string()
    };

    Ok(BlastRadiusReport {
        symbol: symbol.to_string(),
        risk_level,
        caller_count,
        test_files,
    })
}

pub fn check_what_if_analysis_health(skill_dir: &Path) -> Result<Vec<String>, String> {
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
            "What-If Analysis health check failed. Missing files: {:?}",
            missing
        ))
    }
}

pub fn run_what_if_analysis_command(
    subcommand: &WhatIfAnalysisSubcommand,
    repo_root: &Path,
) -> Result<(), String> {
    match subcommand {
        WhatIfAnalysisSubcommand::Check(args) => {
            let skill_dir = if let Some(p) = &args.path {
                sanitize_path(p, Some(repo_root))?
            } else {
                repo_root.join("skills").join("what-if-analysis")
            };

            match check_what_if_analysis_health(&skill_dir) {
                Ok(_) => {
                    println!("What-If Analysis skill health check passed cleanly.");
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        WhatIfAnalysisSubcommand::Impact(args) => {
            let report = analyze_symbol_blast_radius(&args.symbol, repo_root)?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!(
                    "What-If Blast Radius Analysis for symbol: '{}'",
                    report.symbol
                );
                println!("  Risk Level: {}", report.risk_level);
                println!("  Call Sites Found: {}", report.caller_count);
                println!("  Impacted Test Suites: {}", report.test_files.len());
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_analyze_symbol_blast_radius_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();

        fs::write(
            base.join("main.rs"),
            "fn main() { target_function(); }\nfn target_function() {}\n",
        )
        .unwrap();

        let report = analyze_symbol_blast_radius("target_function", &base).unwrap();
        assert_eq!(report.symbol, "target_function");
        assert_eq!(report.caller_count, 1);
        assert_eq!(report.risk_level, "LOW");
    }

    #[test]
    fn test_check_what_if_analysis_health_strict_assertions() {
        let dir = tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let references = base.join("references");
        fs::create_dir_all(&references).unwrap();

        fs::write(base.join("SKILL.md"), "---").unwrap();
        fs::write(base.join("README.md"), "# Title").unwrap();
        fs::write(references.join("overview.md"), "# Overview").unwrap();

        let res = check_what_if_analysis_health(&base);
        assert!(res.is_ok());
    }
}
